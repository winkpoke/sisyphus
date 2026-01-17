use clap::{Parser, Subcommand};
use common::{config::Config, logging};
use std::path::Path;

use sisyphus_cli_core::commands;

mod tui_cmd;

#[macro_use]
extern crate rust_i18n;

// Initialize i18n once in the binary, shared with all crates
i18n!("../common/locales");

#[derive(Parser)]
#[command(name = "sisyphus")]
#[command(about = "AI Agent CLI", long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, global = true)]
    config: Option<String>,
    /// Path to log file (instead of stderr)
    #[arg(long, global = true)]
    log_file: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a REPL session (interactive command-line interface)
    Repl {
        /// URL of remote server to connect to
        #[arg(long)]
        attach: Option<String>,
    },
    /// Start a TUI session (rich terminal interface, requires --features tui)
    Tui {
        /// URL of remote server to connect to
        #[arg(long)]
        attach: Option<String>,
    },
    /// Send a one-shot message and exit
    Msg {
        /// URL of remote server to connect to
        #[arg(long)]
        attach: Option<String>,
        /// Message to send
        message: String,
    },
    /// Start a server (dedicated server mode)
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Print the bound port to stdout (useful when port is 0)
        #[arg(long)]
        print_port: bool,
    },
}
fn init_interactive_logging(log_file: Option<String>) -> anyhow::Result<()> {
    let (default_level, output) = if let Some(path) = log_file {
        ("info", logging::LogOutput::File(path))
    } else {
        ("off", logging::LogOutput::Null)
    };

    logging::init(logging::LogConfig {
        default_level,
        output,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    let config = Config::load(cli.config.as_deref().map(Path::new)).unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
        std::process::exit(1);
    });

    rust_i18n::set_locale(&config.language);

    let config_path = cli.config.clone();
    match cli.command.unwrap_or(Commands::Repl { attach: None }) {
        Commands::Repl { attach } => {
            init_interactive_logging(cli.log_file.clone())?;
            sisyphus_cli_core::ui::banner::print_startup_info(&config);
            commands::repl::run(attach, config_path, cli.log_file).await?;
        }
        Commands::Tui { attach } => {
            init_interactive_logging(cli.log_file.clone())?;
            sisyphus_cli_core::ui::banner::print_startup_info(&config);
            tui_cmd::run(attach, config_path, cli.log_file).await?;
        }
        Commands::Msg { attach, message } => {
            init_interactive_logging(cli.log_file.clone())?;
            commands::msg::run(attach, message, config_path, cli.log_file).await?;
        }
        Commands::Serve { port, print_port } => {
            logging::init(logging::LogConfig {
                default_level: "info",
                output: cli
                    .log_file
                    .clone()
                    .map(logging::LogOutput::File)
                    .unwrap_or(logging::LogOutput::Stderr),
            })?;
            commands::serve::run(config, port, print_port, cli.log_file).await?;
        }
    }

    Ok(())
}
