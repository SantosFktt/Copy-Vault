# Copy&Vault

Copy&Vault is a local-first clipboard manager for Ubuntu/Linux. It is designed
to keep clipboard history useful without requiring an account, a server, or a
network connection.

> **Status:** foundation and documentation only. Clipboard monitoring is not
> implemented yet.

## Goals

- Native Rust and GTK4/libadwaita desktop application.
- Local-only storage with no telemetry or remote service.
- Text clipboard history, search, reuse, pinning, and explicit deletion.
- Clear behavior across X11 and Wayland, without promising capabilities the
  platform does not provide.

The MVP targets Ubuntu 24.04 LTS and is intended to support both Wayland and
X11 where the platform permits. See [the architecture document](docs/ARCHITECTURE.md)
for the current boundaries and known platform constraints.

## Development status

The repository currently contains the project foundation and implementation
plan. The next implementation phase must validate clipboard APIs on Ubuntu
24.04/GNOME/Wayland before adding a provider or monitor.

## Privacy

Copy&Vault is intended to run offline and store data only on the user's
computer. Clipboard contents can contain passwords, tokens, and personal data;
the application must make persistence and deletion behavior explicit. See
[SECURITY.md](SECURITY.md) for reporting security issues and
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for privacy constraints.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a change. The project
uses the MIT license; see [LICENSE](LICENSE).
