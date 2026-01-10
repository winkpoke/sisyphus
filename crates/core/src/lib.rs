#[macro_use]
extern crate rust_i18n;

i18n!("../common/locales");

pub mod agent;
pub mod command;
pub mod service;
pub mod session;
pub mod template;
