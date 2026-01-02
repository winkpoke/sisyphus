use clap::{Parser, Subcommand};
use common::{config::Config, logging};
use std::path::Path;

mod bootstrap;
mod commands;
mod server_manager;
mod ui;

use ui::banner;

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

    // 5. Print Startup Banner
    banner::print_startup_info(&config);

    let config_path = cli.config.clone();
    match cli.command.unwrap_or(Commands::Chat) {
        Commands::Chat => {
            commands::chat::run(config, config_path).await?;
        }
        Commands::Serve { port } => {
            commands::serve::run(config, port).await?;
        }
        Commands::Attach { url } => {
            commands::chat::attach(url).await?;
        }
    }

    Ok(())
}
