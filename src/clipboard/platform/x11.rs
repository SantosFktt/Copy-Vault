use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use x11rb::{
    connection::Connection,
    protocol::{
        xfixes::{ConnectionExt as XFixesConnectionExt, SelectionEventMask},
        xproto::{
            self, Atom, AtomEnum, ConnectionExt as XprotoConnectionExt, CreateWindowAux, EventMask,
            SelectionNotifyEvent, Window, WindowClass,
        },
        Event,
    },
    rust_connection::RustConnection,
    COPY_FROM_PARENT, CURRENT_TIME,
};

use crate::clipboard::{
    detect_backend, ClipboardCapabilities, ClipboardData, ClipboardError, ClipboardEvent,
    ClipboardProvider, ClipboardResult, DisplayBackend, MimeType,
};

const UTF8_STRING: &[u8] = b"UTF8_STRING";
const TEXT: &[u8] = b"TEXT";
const STRING: &[u8] = b"STRING";
const CLIPBOARD: &[u8] = b"CLIPBOARD";
const TARGETS: &[u8] = b"TARGETS";
const INCR: &[u8] = b"INCR";
const REQUEST_PROPERTY: &[u8] = b"COPY_VAULT_SELECTION";

pub struct X11ClipboardProvider {
    stop: Option<Arc<AtomicBool>>,
    monitor: Option<JoinHandle<()>>,
}

impl X11ClipboardProvider {
    pub fn new() -> Self {
        Self {
            stop: None,
            monitor: None,
        }
    }

    fn monitor(
        stop: Arc<AtomicBool>,
        events: Box<dyn Fn(ClipboardEvent) + Send + 'static>,
    ) -> ClipboardResult<()> {
        let (connection, screen, atoms) = connect()?;
        let root = connection.setup().roots[screen].root;
        let requestor = create_requestor(&connection, root)?;

        connection
            .xfixes_select_selection_input(
                root,
                atoms.clipboard,
                SelectionEventMask::SET_SELECTION_OWNER,
            )
            .map_err(connection_error)?
            .check()
            .map_err(connection_error)?;
        connection.flush().map_err(connection_error)?;

        let mut last_text = None;
        while !stop.load(Ordering::Acquire) {
            let Some(event) = connection.poll_for_event().map_err(connection_error)? else {
                thread::sleep(Duration::from_millis(20));
                continue;
            };

            let Event::XfixesSelectionNotify(selection) = event else {
                continue;
            };
            if selection.selection != atoms.clipboard || selection.owner == x11rb::NONE {
                continue;
            }

            let Ok(text) = read_text(&connection, requestor, &atoms) else {
                continue;
            };
            if is_duplicate(last_text.as_deref(), &text) {
                continue;
            }
            last_text = Some(text.clone());

            events(ClipboardEvent {
                formats: vec![MimeType::text_plain()],
                data: Some(ClipboardData::text(text)),
            });
        }

        connection
            .destroy_window(requestor)
            .map_err(connection_error)?
            .check()
            .map_err(connection_error)?;
        Ok(())
    }
}

impl Default for X11ClipboardProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardProvider for X11ClipboardProvider {
    fn capabilities(&self) -> ClipboardCapabilities {
        ClipboardCapabilities {
            can_read: true,
            can_write: false,
            can_discover_formats: true,
            can_monitor_changes: true,
            can_monitor_in_background: true,
            can_read_rich_text: false,
        }
    }

    fn start(
        &mut self,
        events: Box<dyn Fn(ClipboardEvent) + Send + 'static>,
    ) -> ClipboardResult<()> {
        if detect_backend() != DisplayBackend::X11 {
            return Err(ClipboardError::Unsupported {
                operation: "X11 clipboard outside an X11 display",
            });
        }
        if self.monitor.is_some() {
            return Err(ClipboardError::Unsupported {
                operation: "monitor already started",
            });
        }

        let (_, _, _) = connect()?;
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        self.monitor = Some(thread::spawn(move || {
            let _ = Self::monitor(thread_stop, events);
        }));
        self.stop = Some(stop);
        Ok(())
    }

    fn stop(&mut self) -> ClipboardResult<()> {
        let Some(stop) = self.stop.take() else {
            return Ok(());
        };
        stop.store(true, Ordering::Release);
        if let Some(monitor) = self.monitor.take() {
            monitor.join().map_err(|_| ClipboardError::Cancelled)?;
        }
        Ok(())
    }

    fn read(&self, format: &MimeType) -> ClipboardResult<ClipboardData> {
        if format != &MimeType::text_plain() {
            return Err(ClipboardError::Unsupported {
                operation: "X11 text provider format",
            });
        }
        if detect_backend() != DisplayBackend::X11 {
            return Err(ClipboardError::Unsupported {
                operation: "X11 clipboard outside an X11 display",
            });
        }
        let (connection, screen, atoms) = connect()?;
        let root = connection.setup().roots[screen].root;
        let requestor = create_requestor(&connection, root)?;
        let result = read_text(&connection, requestor, &atoms).map(ClipboardData::text);
        connection
            .destroy_window(requestor)
            .map_err(connection_error)?
            .check()
            .map_err(connection_error)?;
        result
    }

    fn write(&self, _data: ClipboardData) -> ClipboardResult<()> {
        Err(ClipboardError::Unsupported {
            operation: "X11 clipboard write",
        })
    }
}

struct Atoms {
    clipboard: Atom,
    targets: Atom,
    utf8_string: Atom,
    text: Atom,
    string: Atom,
    incr: Atom,
    request_property: Atom,
}

fn connect() -> ClipboardResult<(RustConnection, usize, Atoms)> {
    let (connection, screen) = x11rb::connect(None).map_err(connection_error)?;
    connection
        .xfixes_query_version(5, 0)
        .map_err(connection_error)?
        .reply()
        .map_err(connection_error)?;

    let atoms = Atoms {
        clipboard: intern(&connection, CLIPBOARD)?,
        targets: intern(&connection, TARGETS)?,
        utf8_string: intern(&connection, UTF8_STRING)?,
        text: intern(&connection, TEXT)?,
        string: intern(&connection, STRING)?,
        incr: intern(&connection, INCR)?,
        request_property: intern(&connection, REQUEST_PROPERTY)?,
    };
    Ok((connection, screen, atoms))
}

fn create_requestor(connection: &RustConnection, root: Window) -> ClipboardResult<Window> {
    let requestor = connection.generate_id().map_err(connection_error)?;
    connection
        .create_window(
            COPY_FROM_PARENT as u8,
            requestor,
            root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
        )
        .map_err(connection_error)?
        .check()
        .map_err(connection_error)?;
    connection.flush().map_err(connection_error)?;
    Ok(requestor)
}

fn intern(connection: &RustConnection, name: &[u8]) -> ClipboardResult<Atom> {
    connection
        .intern_atom(false, name)
        .map_err(connection_error)?
        .reply()
        .map(|reply| reply.atom)
        .map_err(connection_error)
}

fn read_text(
    connection: &RustConnection,
    requestor: Window,
    atoms: &Atoms,
) -> ClipboardResult<String> {
    let target = select_text_target(connection, requestor, atoms)?;
    request_selection(connection, requestor, atoms, target)
}

fn select_text_target(
    connection: &RustConnection,
    requestor: Window,
    atoms: &Atoms,
) -> ClipboardResult<Atom> {
    let reply = request_selection_property(connection, requestor, atoms, atoms.targets)?;
    if reply.type_ == atoms.incr {
        return Err(ClipboardError::TransferFailed(
            "TARGETS transfer unexpectedly used INCR".to_owned(),
        ));
    }
    let available = reply
        .value32()
        .ok_or_else(|| ClipboardError::TransferFailed("invalid TARGETS reply".to_owned()))?;
    let available: Vec<Atom> = available.collect();

    preferred_text_target(&available, atoms.utf8_string, atoms.text, atoms.string).ok_or_else(
        || ClipboardError::Unsupported {
            operation: "X11 clipboard text target",
        },
    )
}

fn preferred_text_target(
    available: &[Atom],
    utf8_string: Atom,
    text: Atom,
    string: Atom,
) -> Option<Atom> {
    [utf8_string, text, string]
        .into_iter()
        .find(|target| available.contains(target))
}

fn is_duplicate(last: Option<&str>, current: &str) -> bool {
    last == Some(current)
}

fn request_selection(
    connection: &RustConnection,
    requestor: Window,
    atoms: &Atoms,
    target: Atom,
) -> ClipboardResult<String> {
    let reply = request_selection_property(connection, requestor, atoms, target)?;
    let bytes = if reply.type_ == atoms.incr {
        read_incremental(connection, requestor, atoms)?
    } else {
        reply.value
    };
    String::from_utf8(bytes)
        .map_err(|_| ClipboardError::TransferFailed("clipboard text is not UTF-8".to_owned()))
}

fn request_selection_property(
    connection: &RustConnection,
    requestor: Window,
    atoms: &Atoms,
    target: Atom,
) -> ClipboardResult<xproto::GetPropertyReply> {
    connection
        .convert_selection(
            requestor,
            atoms.clipboard,
            target,
            atoms.request_property,
            CURRENT_TIME,
        )
        .map_err(connection_error)?
        .check()
        .map_err(connection_error)?;
    connection.flush().map_err(connection_error)?;

    loop {
        let event = connection.wait_for_event().map_err(connection_error)?;
        if let Event::SelectionNotify(SelectionNotifyEvent {
            requestor: event_requestor,
            selection,
            property,
            ..
        }) = event
        {
            if event_requestor != requestor || selection != atoms.clipboard {
                continue;
            }
            if property == x11rb::NONE {
                return Err(ClipboardError::TransferFailed(
                    "clipboard owner rejected the requested target".to_owned(),
                ));
            }
            return connection
                .get_property(false, requestor, property, AtomEnum::ANY, 0, u32::MAX)
                .map_err(connection_error)?
                .reply()
                .map_err(connection_error);
        }
    }
}

fn read_incremental(
    connection: &RustConnection,
    requestor: Window,
    atoms: &Atoms,
) -> ClipboardResult<Vec<u8>> {
    connection
        .delete_property(requestor, atoms.request_property)
        .map_err(connection_error)?
        .check()
        .map_err(connection_error)?;
    connection.flush().map_err(connection_error)?;

    let mut bytes = Vec::new();
    loop {
        let event = connection.wait_for_event().map_err(connection_error)?;
        let Event::PropertyNotify(property) = event else {
            continue;
        };
        if property.window != requestor
            || property.atom != atoms.request_property
            || property.state != xproto::Property::NEW_VALUE
        {
            continue;
        }

        let reply = connection
            .get_property(
                true,
                requestor,
                atoms.request_property,
                AtomEnum::ANY,
                0,
                u32::MAX,
            )
            .map_err(connection_error)?
            .reply()
            .map_err(connection_error)?;
        if reply.value.is_empty() {
            return Ok(bytes);
        }
        bytes.extend_from_slice(&reply.value);
    }
}

fn connection_error(error: impl std::fmt::Debug) -> ClipboardError {
    ClipboardError::TransferFailed(format!("X11 clipboard operation failed: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_are_text_read_and_background_monitoring_only() {
        let capabilities = X11ClipboardProvider::new().capabilities();

        assert!(capabilities.can_read);
        assert!(capabilities.can_discover_formats);
        assert!(capabilities.can_monitor_changes);
        assert!(capabilities.can_monitor_in_background);
        assert!(!capabilities.can_write);
        assert!(!capabilities.can_read_rich_text);
    }

    #[test]
    fn prefers_utf8_then_text_then_string() {
        assert_eq!(preferred_text_target(&[3, 2, 1], 1, 2, 3), Some(1));
        assert_eq!(preferred_text_target(&[3, 2], 1, 2, 3), Some(2));
        assert_eq!(preferred_text_target(&[3], 1, 2, 3), Some(3));
        assert_eq!(preferred_text_target(&[4], 1, 2, 3), None);
    }

    #[test]
    fn duplicate_text_is_not_reemitted() {
        assert!(!is_duplicate(None, "same"));
        assert!(is_duplicate(Some("same"), "same"));
        assert!(!is_duplicate(Some("old"), "same"));
    }
}
