# Contributing to Copy&Vault

Thank you for helping improve Copy&Vault. Please keep changes focused and
preserve the project's local-first and privacy-first design.

## Before opening a pull request

- Explain the user-visible behavior and platform assumptions.
- Add or update tests for domain, storage, search, or privacy behavior.
- Do not add network calls, telemetry, or content-bearing logs.
- Do not claim Wayland support without a reproducible Ubuntu 24.04/GNOME test.
- Run `cargo fmt --check`, `cargo check`, and the relevant test commands.

Clipboard contents may be sensitive. Never include real clipboard data,
credentials, tokens, or personal information in issues, logs, fixtures, or
screenshots.
