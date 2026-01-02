use clap::{Parser, Subcommand};
use common::{config::Config, logging, bus::EventBus, llm::LLMProvider, path::SandboxedPath};
use provider::{openai::OpenAIProvider, mock::MockProvider};
use sisyphus_core::agent::{Agent, config::AgentConfig};
use sisyphus_core::session::manager::SessionManager;
use tools::{cmd::CommandTool, fs::{ReadFileTool, WriteFileTool}};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use rust_i18n::t;
use futures::StreamExt;
use reqwest_eventsource::Event;

mod server_manager;
mod banner;
use server_manager::ServerManager;

#[macro_use]
extern crate rust_i18n;

i18n!("../common/locales");

#[derive(Parser)]
#[command(name = "sisyphus")]
#[command(about = "AI Agent CLI", long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a chat session
    Chat,
    /// Start the server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Attach to a running server
    Attach {
        /// URL of the server
        url: String,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenv::dotenv().ok();

    // 1. Parse Args
    let cli = Cli::parse();

    // 2. Load Config
    let config = Config::load(cli.config.as_deref().map(Path::new)).unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
        std::process::exit(1);
    });

    // 3. Init Logging
    logging::init();

    // 4. Init Locale
    rust_i18n::set_locale(&config.language);

    // 5. Print Startup Banner
    banner::print_startup_info(&config);

    let config_path = cli.config.clone();
    match cli.command.unwrap_or(Commands::Chat) {
        Commands::Chat => {
            run_chat(config, config_path).await?;
        }
        Commands::Serve { port } => {
            run_serve(config, port).await?;
        }
        Commands::Attach { url } => {
            run_attach(url).await?;
        }
    }

    Ok(())
}

async fn run_serve(config: Config, port: u16) -> anyhow::Result<()> {
    println!("{}", t!("starting_server", port = port));

    let bus = Arc::new(EventBus::new(100));
    
    // Subscribe to bus for logging
    logging::start_event_logger(&bus).await;

    let provider: Box<dyn LLMProvider> = match config.llm.provider.as_str() {
        "mock" => Box::new(MockProvider::new()),
        _ => {
            println!("Initializing provider: {} (model: {})", config.llm.provider, config.llm.model);
            if let Some(ref url) = config.llm.base_url {
                println!("Base URL: {}", url);
            } else if config.llm.provider != "openai" {
                println!("Warning: No base_url specified for custom provider. Defaulting to OpenAI.");
            }

            let api_key = config.llm.api_key.clone().or_else(|| std::env::var("OPENAI_API_KEY").ok())
                .expect("API Key must be set");
            Box::new(OpenAIProvider::new(
                api_key, 
                config.llm.base_url.clone(), 
                config.llm.model.clone()
            ))
        }
    };

    let mut agent = Agent::new(provider, bus.clone(), AgentConfig::default());

    // Register tools
    agent.register_tool(Box::new(CommandTool));
    
    let cwd = std::env::current_dir()?;
    let sandbox = Arc::new(SandboxedPath::new(cwd)?);
    
    agent.register_tool(Box::new(ReadFileTool::new(sandbox.clone())));
    agent.register_tool(Box::new(WriteFileTool::new(sandbox)));

    let session_manager = Arc::new(tokio::sync::Mutex::new(SessionManager::new()));
    let agent = Arc::new(agent);
    
    server::Server::new(port, agent, session_manager, bus).run().await?;
    
    Ok(())
}

async fn run_chat(_config: Config, config_path: Option<String>) -> anyhow::Result<()> {
    println!("{}", t!("starting_agent"));

    // 1. Start Server
    let port = std::net::TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
    let mut server_manager = ServerManager::start(port, config_path).await?;
    let client = server_manager.client();

    // 2. Create Session
    let session = client.create_session().await?;
    println!("Session ID: {}", session.id);

    // 3. Subscribe to Events
    let mut events = client.subscribe_events()?;
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);
    
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            match event {
                Ok(Event::Message(msg)) => {
                     tracing::debug!("Event: {:?}", msg);
                     if let Ok(event) = serde_json::from_str::<common::bus::SystemEvent>(&msg.data) {
                         if let common::bus::SystemEvent::Shutdown = event {
                             let _ = shutdown_tx.send(()).await;
                             break;
                         }
                     }
                }
                _ => {}
            }
        }
    });

    // 4. Chat Loop
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    println!("{}", t!("type_exit"));
    loop {
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush()?;
        
        line.clear();
        
        let bytes = tokio::select! {
            _ = shutdown_rx.recv() => {
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutting down...");
                break;
            }
            res = reader.read_line(&mut line) => {
                res?
            }
        };

        if bytes == 0 {
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("/quit") || input.eq_ignore_ascii_case("/exit") {
            break;
        }

        match client.chat(&session.id, input.to_string()).await {
            Ok(response) => {
                if !response.is_empty() {
                    println!("{}", t!("assistant_prefix", msg = response));
                }
            }
            Err(e) => {
                eprintln!("{}", t!("error_prefix", err = e));
            }
        }
    }

    server_manager.stop().await?;
    Ok(())
}

async fn run_attach(url: String) -> anyhow::Result<()> {
    let url = url::Url::parse(&url)?;
    let server_manager = ServerManager::connect(url).await?;
    let client = server_manager.client();

    let session = client.create_session().await?;
    println!("Session ID: {}", session.id);

    // Similar chat loop as run_chat, but without server management (cleanup)
    
    let mut events = client.subscribe_events()?;
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            match event {
                Ok(Event::Message(msg)) => {
                     tracing::debug!("Event: {:?}", msg);
                }
                _ => {}
            }
        }
    });

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    println!("{}", t!("type_exit"));
    loop {
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush()?;
        
        line.clear();
        
        let bytes = tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\nExiting...");
                break;
            }
            res = reader.read_line(&mut line) => {
                res?
            }
        };

        if bytes == 0 {
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("/quit") || input.eq_ignore_ascii_case("/exit") {
            break;
        }

        match client.chat(&session.id, input.to_string()).await {
            Ok(response) => {
                if !response.is_empty() {
                    println!("{}", t!("assistant_prefix", msg = response));
                }
            }
            Err(e) => {
                eprintln!("{}", t!("error_prefix", err = e));
            }
        }
    }

    Ok(())
}
