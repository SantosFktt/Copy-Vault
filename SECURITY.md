# Security policy

## Scope

Copy&Vault is local-first. Security-sensitive areas include clipboard
contents, SQLite and archive files, deletion, permissions, logs, and any
future platform integration.

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting for this repository when
available. If it is unavailable, open a private contact with the maintainers
before publishing details. Do not include real secrets or clipboard contents in
the report.

Reports should include the affected version or commit, platform/session type,
reproduction steps using synthetic data, and the impact.

## Design expectations

- No clipboard payloads in logs or telemetry.
- Explicit errors instead of silent data loss.
- Active and archived copies must be covered by deletion behavior.
- Wayland limitations must be reported rather than bypassed unsafely.
