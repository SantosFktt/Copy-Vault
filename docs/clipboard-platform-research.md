# Clipboard platform research

**Scope:** Ubuntu 24.04 LTS, with GNOME 46 on Wayland as the primary target,
and an X11 session as the compatibility target. This is a research record, not
an implementation. No clipboard dependency or provider has been added.

## Executive summary

GTK4/GDK is the correct application-facing API for ordinary clipboard reads,
writes, MIME discovery, and asynchronous transfers. It maps to the active
display backend and keeps GTK integration independent from X11 and Wayland
protocol details.

X11 provides the primitives required by a traditional background clipboard
manager: `CLIPBOARD` selection ownership, XFixes selection-owner change
notifications, and the ICCCM clipboard transfer protocol. A native X11
provider can therefore support continuous monitoring, subject to normal X11
selection-owner and transfer limitations.

Wayland core provides clipboard selection transfer, but not a universal
desktop-independent background clipboard-manager API. A normal GTK/GDK client
can read and write clipboard data when the compositor exposes it through its
data-device integration, and can observe changes delivered to that client.
That is not a guarantee of global history capture while the application is
backgrounded. Robust background capture requires a compositor-specific manager
protocol (for example, the unstable `zwlr_data_control_v1` protocol on some
wlroots compositors) or desktop-specific integration. GNOME/Ubuntu must not be
treated as if it provided that wlroots protocol.

The MVP can guarantee explicit clipboard operations initiated while the
application has a working GDK display connection, especially text read/write.
It cannot guarantee global, lossless, background clipboard history on Ubuntu
GNOME Wayland without validating and adopting a GNOME-supported mechanism.

## APIs and libraries evaluated

### GTK4/GDK (`GdkClipboard`)

GDK exposes the portable application API:

- `gdk_display_get_clipboard()` obtains the standard clipboard.
- `gdk_display_get_primary_clipboard()` obtains the primary selection where
  supported.
- `gdk_clipboard_get_formats()` reports currently advertised formats as
  `GdkContentFormats`.
- `gdk_clipboard_read_text_async()` reads text asynchronously.
- `gdk_clipboard_read_async()` reads another advertised format through a
  `GInputStream`.
- `gdk_clipboard_set_text()` writes plain text.
- `gdk_clipboard_set_content()` publishes arbitrary data through a
  `GdkContentProvider`.
- The `GdkClipboard::changed` signal reports a clipboard ownership change; it
  is not proof that a universal background-monitoring facility exists.

The GDK API is the recommended first layer for both display backends. Clipboard
I/O is asynchronous and must not block the GTK main loop.

References:

- [GdkClipboard](https://docs.gtk.org/gdk4/class.Clipboard.html)
- [GdkClipboard::changed](https://docs.gtk.org/gdk4/signal.Clipboard.changed.html)
- [GdkContentFormats](https://docs.gtk.org/gdk4/struct.ContentFormats.html)
- [GDK Wayland interaction](https://docs.gtk.org/gdk4/wayland.html)

### Wayland core data-device protocol

The core protocol uses a `wl_data_device` associated with a `wl_seat`.
Clipboard data is offered through `wl_data_offer`, with one or more MIME types;
the receiving client requests a selected type and reads it from a file
descriptor. A client offering data uses `wl_data_source` and advertises MIME
types before becoming the seat's selection.

This is transfer and ownership infrastructure, not a global clipboard database.
The protocol does not define a cross-compositor "clipboard manager daemon"
permission or a stable event stream intended for third-party history managers.
GDK handles the normal data-device interaction for an application.

Reference: [Wayland protocol appendix](https://wayland.freedesktop.org/docs/html/apa.html).

### Wayland data-control extensions

Some wlroots-based compositors expose the unstable
`zwlr_data_control_manager_v1` protocol. It is specifically designed to let a
clipboard manager observe and take over selections without being the focused
application. It is not part of Wayland core, is not a stable cross-desktop
contract, and must be discovered at runtime.

It must not be assumed to exist in Ubuntu GNOME. Supporting it would be a
separate compositor-specific provider with a separate test matrix.

### X11 selections and XFixes

X11 represents the clipboard as the `CLIPBOARD` selection. The owner provides
data when a request arrives rather than writing into a central buffer. The
ICCCM defines target negotiation, including `TARGETS`, `UTF8_STRING`, `TEXT`,
and the incremental `INCR` transfer for large payloads.

The XFixes extension supplies selection-owner notifications. A clipboard
manager can select for `CLIPBOARD` changes, read the offered targets, and
cache content before the original owner exits. The freedesktop clipboard
manager extension also defines cooperation such as `SAVE_TARGETS`.

References:

- [freedesktop Clipboard Specification](https://specifications.freedesktop.org/clipboard/latest/)
- [freedesktop Clipboard Extensions](https://specifications.freedesktop.org/clipboard-extensions/latest/)
- [XFixes protocol](https://xorg.freedesktop.org/archive/current/doc/fixesproto/fixesproto.txt)

### XDG Desktop Portal Clipboard interface

The portal interface is not a general background clipboard-history API. Its
`org.freedesktop.portal.Clipboard` methods attach clipboard access to an
existing portal session, such as Remote Desktop or Input Capture. The session
must be created and started by the owning portal flow. It supports MIME
advertisement and file-descriptor transfers through methods such as
`RequestClipboard`, `SetSelection`, `SelectionRead`, and `SelectionWrite`.

This is useful for sandboxed or user-mediated integration, but it does not
authorize a standalone Copy&Vault daemon to monitor every clipboard change in
the background.

Reference: [XDG Desktop Portal Clipboard](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Clipboard.html).

## Capability matrix

| Capability | X11 | Wayland |
| --- | --- | --- |
| Read clipboard | **Yes**, through `CLIPBOARD` selection requests and GDK | **Yes**, when a data offer is exposed to the client; asynchronous and compositor-mediated |
| Write clipboard | **Yes**, by owning `CLIPBOARD`; GDK provides the normal API | **Yes**, by publishing a `wl_data_source`; GDK provides the normal API |
| Monitor changes | **Yes**, XFixes selection-owner notifications | **Not universally** through a stable global manager API; GDK can report changes visible to its backend/client |
| Background monitoring | **Yes**, with an X11 event connection and selection-transfer handling | **Not guaranteed on Ubuntu GNOME**; requires compositor/desktop-specific support such as a data-control extension |
| MIME types | **Yes**, via ICCCM targets (`TARGETS`, `UTF8_STRING`, `TEXT`, etc.) | **Yes**, via `wl_data_offer` MIME offers and GDK `GdkContentFormats` |
| Rich text | **Usually possible** when the owner offers HTML/RTF or another target; conversion and lifetime are responsibilities of the manager | **Possible per advertised MIME type**, but not guaranteed; the source may offer only `text/plain` and background access remains the limiting factor |

“Yes” means the protocol/API supports the operation, not that every
application offers every format. “Not guaranteed” is intentional: a provider
must report capability and transfer failures rather than silently treating a
partial Wayland implementation as complete.

## Recommended solution

### X11

Use GDK for normal reads and writes, and an X11-specific monitoring adapter
when the runtime display is X11:

1. Connect to the X display and select XFixes selection-owner notifications
   for `CLIPBOARD` (and optionally `PRIMARY` as a separate user setting).
2. On a notification, request `TARGETS`, prefer UTF-8 text, and retain other
   supported formats only when their transfer is bounded and understood.
3. Handle ICCCM `INCR` transfers and owner disappearance explicitly.
4. Use the X11 event source through the GLib main context; do not poll.

This is the only target in this research where continuous background capture
can be a normal MVP guarantee.

### Wayland on Ubuntu GNOME

Use GDK for foreground/user-initiated read and write operations and expose a
runtime capability state for monitoring. Before implementing automatic capture,
perform a GNOME 46/Ubuntu 24.04 experiment that verifies whether the running
compositor delivers the required selection events and data offers to a
background GTK client.

Do not add a direct Wayland protocol dependency yet. If GNOME does not provide
a supported background mechanism, the MVP should offer manual capture or
foreground capture and state plainly that global history is unavailable in that
session. A later GNOME-specific integration may be evaluated separately; a
wlroots `zwlr_data_control_v1` adapter must not be presented as GNOME support.

## Dependencies and permissions

No new dependency is justified by this research alone. The eventual portable
layer will require GTK4/libadwaita and their Ubuntu development/runtime
packages. An X11-specific provider would additionally need access to X11/XFixes
bindings, selected only when implementation starts and the binding choice is
documented. A compositor-specific Wayland provider would need generated
protocol bindings and the compositor's advertised protocol.

No special operating-system permission is normally required for ordinary
clipboard use by an unsandboxed GTK application:

- X11 requires a usable `DISPLAY` connection and access to the X server.
- Wayland requires a usable `WAYLAND_DISPLAY`/desktop display connection and
  seat integration supplied by the compositor.
- A sandboxed Flatpak may need the relevant desktop socket and portal access;
  portal clipboard access also requires an applicable, started portal session.

These are connection and session conditions, not a blanket permission to
monitor all clipboard contents. Packaging must be tested separately for
unsandboxed and Flatpak deployments.

## GTK4/libadwaita integration

Create the provider on the GTK/GDK display's GLib main context. Use
`GdkClipboard` for async reads/writes and `GdkContentFormats` for advertised
MIME types. Convert completed transfers into domain events, then hand them to
the application/storage layer without passing GTK objects into the domain.

The UI should show backend and capability state, support pause/resume, and
surface unsupported or failed transfers. Clipboard callbacks must not log
payloads. The provider must suppress duplicate events caused by Copy&Vault's
own write operation.

## Proposed Rust abstraction

This is an interface proposal only; it is deliberately not implemented in this
PR:

```rust
pub trait ClipboardProvider {
    fn capabilities(&self) -> ClipboardCapabilities;
    fn start(&mut self, events: Box<dyn Fn(ClipboardEvent) + Send>);
    fn stop(&mut self);
    fn read(&self, format: &MimeType) -> ClipboardResult<ClipboardData>;
    fn write(&self, data: ClipboardData) -> ClipboardResult<()>;
}
```

The final trait must account for asynchronous GTK operations, so the exact
signatures may instead use futures or channels owned by the application layer.
The important boundaries are:

- `ClipboardCapabilities` distinguishes read, write, MIME discovery,
  change-events, background operation, and rich-text support.
- `ClipboardEvent` contains metadata and offered MIME types, not logging
  content by default.
- `ClipboardData` supports text now and leaves room for binary/image data.
- Errors distinguish unsupported capability, unavailable display, transfer
  failure, and cancellation.
- Implementations are selected by runtime backend and advertised protocol,
  never by assuming Wayland equals X11.

Candidate adapters are `GdkClipboardProvider` for ordinary operations,
`X11ClipboardMonitor` for XFixes-backed monitoring, and a future
compositor-specific provider only after validation.

## Risks and known differences

- X11 selection data is supplied on demand; a slow or exiting owner can make a
  read fail. Large data requires `INCR`.
- X11 exposes more global observation than Wayland, which increases privacy
  risk. Copy&Vault must never store or log content without the user's chosen
  policy.
- Wayland behavior depends on compositor protocols, seat state, and the source
  application's advertised offers. Core protocol support is not a global
  history guarantee.
- GNOME Shell, KDE Plasma, wlroots compositors, and other Linux desktops may
  expose different protocols and policy. Ubuntu GNOME results must not be
  generalized to all Linux environments.
- Portal support is session-oriented and does not replace a universal
  clipboard-monitoring API.
- Rich text is an offered MIME representation, not a guarantee that a
  canonical format or lossless conversion exists.
- GTK/GDK emits ownership-change notifications; it does not by itself promise
  that an application can retain every historical clipboard item while
  backgrounded.

## MVP guarantees and non-guarantees

### We can guarantee after implementation

- Explicit text read and write through GTK4/GDK when the display connection and
  advertised clipboard capability are available.
- MIME inspection for formats exposed by the current clipboard offer.
- Asynchronous GTK integration without blocking the UI thread.
- X11 background monitoring when XFixes and the X11 selection transfer path are
  available and tested.
- Truthful capability/error reporting and no clipboard payloads in logs.

### We cannot guarantee in the initial MVP

- Global, lossless, background clipboard history on Ubuntu GNOME Wayland.
- Access to clipboard data after the source application has withdrawn it when
  no supported manager protocol preserves it.
- Rich text for sources that advertise only plain text.
- Identical behavior across GNOME, KDE, wlroots, remote sessions, and all
  other Linux desktops.
- A portal-based standalone background monitor without an applicable portal
  session.

## Sources consulted

- GTK 4 API reference: [`GdkClipboard`](https://docs.gtk.org/gdk4/class.Clipboard.html),
  [`changed`](https://docs.gtk.org/gdk4/signal.Clipboard.changed.html), and
  [`GdkContentFormats`](https://docs.gtk.org/gdk4/struct.ContentFormats.html).
- Wayland project protocol documentation:
  [protocol appendix](https://wayland.freedesktop.org/docs/html/apa.html).
- freedesktop.org:
  [Clipboard Specification](https://specifications.freedesktop.org/clipboard/latest/),
  [Clipboard Extensions](https://specifications.freedesktop.org/clipboard-extensions/latest/),
  and [XDG Desktop Portal Clipboard](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Clipboard.html).
- X.Org:
  [XFixes protocol](https://xorg.freedesktop.org/archive/current/doc/fixesproto/fixesproto.txt).
- Rust binding reference:
  [`gdk4::Clipboard`](https://docs.rs/gdk4/latest/gdk4/struct.Clipboard.html).

These sources describe protocol/API capabilities. Ubuntu 24.04/GNOME runtime
behavior still requires an on-device validation experiment before committing to
a Wayland background-monitoring design.
