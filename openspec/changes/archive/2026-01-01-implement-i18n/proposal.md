# Implement Internationalization (i18n)

## Summary
Add internationalization (i18n) support to the Sisyphus agent, enabling multi-language output with initial support for English and Chinese.

## Motivation
To make Sisyphus accessible to a wider audience, specifically Chinese users, we need to abstract user-facing strings and support locale switching.

## Proposed Changes
- Introduce `rust-i18n` library to `common` crate.
- Create a centralized locale storage in `crates/common/locales`.
- Update `Config` to support language selection.
- Refactor `cli` and `core` to use the i18n macro for user output.
