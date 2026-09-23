# Architecture

## Scope of this foundation

This repository intentionally contains no clipboard monitor or provider yet.
The first implementation must validate the operating-system capabilities
described below before choosing an API or promising automatic capture.

The intended dependency direction is:

```text
ui -> application -> domain
                 -> storage
                 -> clipboard
                 -> platform
```

The domain and application layers must not depend on GTK, SQLite, X11, or
Wayland types. Platform adapters translate OS-specific behavior into explicit
application results and errors.

## Planned modules

- `domain`: immutable clipboard entries, version relationships, favorites,
  Current selection, content kinds, and settings.
- `storage`: active SQLite database, migrations, retention, and archive
  coordination.
- `clipboard`: provider interface and capture/write operations.
- `platform`: capability detection and X11/Wayland adapters.
- `search`: active-database and archive search orchestration.
- `ui`: GTK4/libadwaita views and user actions.

The initial implementation may add these modules incrementally; empty
abstractions must not conceal unsupported platform behavior.

The first clipboard implementation adds only the domain-level
`ClipboardProvider` contract, clipboard value types, and runtime display
backend detection through GDK. The X11, Wayland, and unsupported-platform
modules are provider slots with no monitoring implementation yet. Domain types
remain independent of GTK, GDK, X11, and Wayland types.

The X11 provider now occupies the X11 slot. It uses `x11rb` and XFixes for
selection-owner notifications and reads UTF-8/text clipboard data through the
ICCCM selection protocol, including incremental transfers. It is deliberately
text-read/monitoring-only; Wayland monitoring and all persistence remain
deferred.

## Data model requirements

Editing an item creates a new immutable item. The original remains unchanged.
Each derived item may reference its origin with `parent_id`, allowing version
navigation in both directions. A separate Current marker identifies the
version the user currently prefers and must be preserved when entries move to
archive storage.

An entry should carry a stable ID, content kind, content payload or archive
reference, creation and usage timestamps, favorite state, Current state, and
version relationship metadata. The model should leave room for image and other
binary content kinds without requiring image support in the MVP.

## Active storage and archives

Recent items live in SQLite. Archiving is a user-configurable policy:

- item-count limit;
- weekly schedule;
- monthly schedule;
- never archive;
- archive retention;
- optional compression.

Archive files must preserve IDs, timestamps, favorites, Current state, and
version relationships. Search must eventually query active SQLite data and
archives through one application-level interface; users should not need to
open archive files manually.

Archive operations must be transactional from the user's perspective: an
item is not removed from active storage until its archive copy has been
validated.

## Clipboard platform constraints

X11 and Wayland are not equivalent. X11 commonly permits clipboard ownership
and observation patterns that are not generally available to arbitrary
Wayland clients. Wayland intentionally restricts global observation, and
support may depend on compositor behavior, portals, desktop integration, or a
foreground application.

Before implementing `ClipboardProvider`, validate on Ubuntu 24.04 with GNOME:

1. Which read/write APIs are available to a regular Wayland client.
2. Whether any portal or GNOME integration supports the required event flow.
3. How behavior differs when the application is backgrounded or inactive.
4. Whether X11 fallback behavior can be detected and documented reliably.

If automatic global monitoring is structurally unavailable on Wayland, the
product must document that limitation and expose a truthful capability state.
It must not provide a fake adapter that silently loses clipboard events.

## Privacy and observability

No network service, account, telemetry, or content-bearing logs are part of the
MVP. Logs may include operation names and sanitized error categories, never
clipboard payloads, search terms, tokens, or database contents. Deletion must
cover active data and the applicable archive copies.
