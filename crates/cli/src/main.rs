use clap::{Parser, Subcommand};
use common::{config::Config, logging};
use std::path::Path;

use sisyphus_cli_lib::commands;
use sisyphus_cli_lib::ui::repl::Repl;

#[macro_use]
extern crate rust_i18n;

i18n!("../cli-lib/src/locales");

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
    Chat {
        /// Use TUI frontend
        #[arg(long)]
        tui: bool,
    },
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
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    let config = Config::load(cli.config.as_deref().map(Path::new)).unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
        std::process::exit(1);
    });

    logging::init();

    rust_i18n::set_locale(&config.language);

    sisyphus_cli_lib::ui::banner::print_startup_info(&config);

    let config_path = cli.config.clone();
    match cli.command.unwrap_or(Commands::Chat { tui: false }) {
        Commands::Chat { tui } => {
            let mut ctx = commands::chat::setup_chat(config_path).await?;

            if tui {
                #[cfg(feature = "tui")]
                {
                    let mut tui_app =
                        tui::Tui::new(ctx.client.clone(), ctx.session_id.clone(), ctx.shutdown_rx);
                    tui_app.run().await?;
                }
                #[cfg(not(feature = "tui"))]
                {
                    eprintln!("Error: TUI feature not enabled. Build with --features tui");
                    std::process::exit(1);
                }
            } else {
                let mut repl = Repl::new(ctx.client, ctx.session_id, ctx.shutdown_rx);
                repl.run().await?;
            }

            ctx.server_manager.stop().await?;
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
