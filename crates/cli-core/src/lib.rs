#[macro_use]
extern crate rust_i18n;

mod bootstrap;
pub mod commands;
mod server_manager;
pub mod ui;

i18n!("../common/locales");

pub use bootstrap::{build_agent_registry, build_builtins, BuiltInAgents};
pub use commands::connection::{setup_connection, ConnectionContext};
pub use commands::msg;
pub use commands::repl;
pub use commands::serve;
pub use server_manager::ServerManager;
