use clap::{Parser, Subcommand};
use common::{config::Config, logging, bus::EventBus, llm::LLMProvider};
use provider::{openai::OpenAIProvider, mock::MockProvider};
use core::agent::Agent;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[derive(Parser)]
#[command(name = "sisyphus")]
#[command(about = "AI Agent CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a chat session
    Chat,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenv::dotenv().ok();

    // 1. Load Config
    let config = Config::new().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
        std::process::exit(1);
    });

    // 2. Init Logging
    logging::init();

    // 3. Parse Args
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Chat) {
        Commands::Chat => {
            run_chat(config).await?;
        }
    }

    Ok(())
}

async fn run_chat(config: Config) -> anyhow::Result<()> {
    println!("Starting Sisyphus Agent...");
    
    // 4. Init Components
    let bus = Arc::new(EventBus::new(100));
    
    // Subscribe to bus for logging
    let mut rx = bus.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                 common::bus::SystemEvent::MessageReceived { role: _, content: _ } => {
                     // We print here to show the event bus is working
                     // In production, this might be handled differently
                     // println!("[BUS] {}: {}", role, content);
                 }
                 _ => {
                     println!("[BUS] {:?}", event);
                 }
            }
        }
    });

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

    // 5. Chat Loop
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    println!("Type 'exit' to quit.");
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

        match agent.chat(input.to_string()).await {
            Ok(response) => {
                println!("Assistant: {}", response);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
