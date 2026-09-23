use gtk4::gdk;
use gtk4::gdk::prelude::DisplayExtManual;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayBackend {
    X11,
    Wayland,
    Unknown,
}

pub fn detect_backend() -> DisplayBackend {
    gdk::Display::default()
        .map(|display| detect_backend_for_display(&display))
        .unwrap_or(DisplayBackend::Unknown)
}

fn detect_backend_for_display(display: &gdk::Display) -> DisplayBackend {
    let backend = display.backend();

    if backend.is_x11() {
        DisplayBackend::X11
    } else if backend.is_wayland() {
        DisplayBackend::Wayland
    } else {
        DisplayBackend::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_variants_are_distinct() {
        assert_ne!(DisplayBackend::X11, DisplayBackend::Wayland);
        assert_ne!(DisplayBackend::Wayland, DisplayBackend::Unknown);
    }
}
