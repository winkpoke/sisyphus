# i18n Implementation Design

## Architecture
- **Library**: `rust-i18n` will be used for loading YAML translation files and providing the `t!` macro.
- **Location**: `crates/common` will host the setup and locale files (`locales/*.yml`).
- **Configuration**: The `Config` struct will have a `language` field (default: "en").
- **Runtime**: The CLI entry point will initialize the locale based on config.

## Directory Structure
crates/common/
  ├── locales/
  │   ├── en.yml
  │   └── zh-CN.yml
  ├── src/
  │   └── lib.rs (export i18n macro)

## Migration
1.  Add dependency.
2.  Create locale files.
3.  Replace hardcoded strings with keys.
