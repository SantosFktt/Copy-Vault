# Implementation plan

## Phase 0: foundation (current)

- Establish the Rust package and repository conventions.
- Document immutable versioning, Current, archiving, federated search, privacy,
  extensible content kinds, and platform boundaries.
- Do not implement clipboard monitoring.

## Phase 1: platform investigation

- Build small, disposable experiments for Ubuntu 24.04/GNOME on Wayland and
  X11.
- Record supported clipboard read, write, ownership, and change-notification
  behavior.
- Define capability reporting and the supported MVP workflow for each session
  type before creating the provider trait.

## Phase 2: domain and storage

- Define immutable clipboard entries and `parent_id` version navigation.
- Add Current and favorite invariants.
- Add SQLite migrations and retention settings.
- Add archive export/import with integrity checks and preserved metadata.
- Add deletion tests covering active and archived data.

## Phase 3: search and clipboard integration

- Add active plus archive search behind one interface.
- Implement provider adapters only for validated platform capabilities.
- Add explicit pause/resume and clear error states for unsupported behavior.

## Phase 4: GTK application

- Add the GTK4/libadwaita window, searchable history, reuse, pin, Current,
  edit-as-new-version, and delete actions.
- Add preferences for retention, archive schedule, compression, and privacy.
- Keep tray/indicator integration optional because desktop support varies.

## Phase 5: packaging and release

- Add unit and integration coverage for domain, storage, search, privacy, and
  platform capability reporting.
- Add Flatpak packaging first and `.deb` packaging as a complement.
- Add reproducible CI checks and document Ubuntu installation and limitations.

## MVP acceptance criteria

The MVP must install on Ubuntu 24.04, operate without network access, preserve
immutable versions, support Current and deletion, avoid content in logs, and
truthfully document the difference between Wayland and X11.
