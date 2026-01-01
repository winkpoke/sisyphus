use clap::{Parser, Subcommand};
use common::{config::Config, logging, bus::EventBus, llm::LLMProvider, path::SandboxedPath};
use provider::{openai::OpenAIProvider, mock::MockProvider};
use core::agent::Agent;
use core::session::manager::SessionManager;
use tools::{cmd::CommandTool, fs::{ReadFileTool, WriteFileTool}};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use rust_i18n::t;

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

    match cli.command.unwrap_or(Commands::Chat) {
        Commands::Chat => {
            run_chat(config).await?;
        }
        Commands::Serve { port } => {
            run_serve(config, port).await?;
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

    let mut agent = Agent::new(provider, bus.clone());

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

async fn run_chat(config: Config) -> anyhow::Result<()> {
    println!("{}", t!("starting_agent"));
    
    // 4. Init Components
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

    let mut agent = Agent::new(provider, bus);

    // Register tools
    agent.register_tool(Box::new(CommandTool));
    
    let cwd = std::env::current_dir()?;
    let sandbox = Arc::new(SandboxedPath::new(cwd)?);
    
    agent.register_tool(Box::new(ReadFileTool::new(sandbox.clone())));
    agent.register_tool(Box::new(WriteFileTool::new(sandbox)));

    // 6. Init Session
    let mut session_manager = SessionManager::new();
    let session = session_manager.create_session();
    println!("Session ID: {}", session.id);

    // 5. Chat Loop
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    println!("{}", t!("type_exit"));
    loop {
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush()?;
        
        line.clear();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            break;
        }

        let input = line.trim();
        if input == "exit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        match agent.chat(session, input.to_string()).await {
            Ok(response) => {
                println!("{}", t!("assistant_prefix", msg = response));
            }
            Err(e) => {
                eprintln!("{}", t!("error_prefix", err = e));
            }
        }
    }

    Ok(())
}
