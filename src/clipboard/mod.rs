mod backend;
mod types;

pub mod platform;

pub use backend::{detect_backend, DisplayBackend};
pub use types::{
    ClipboardCapabilities, ClipboardData, ClipboardError, ClipboardEvent, ClipboardResult, MimeType,
};

pub trait ClipboardProvider {
    fn capabilities(&self) -> ClipboardCapabilities;

    fn start(&mut self, events: Box<dyn Fn(ClipboardEvent) + Send>) -> ClipboardResult<()>;

    fn stop(&mut self) -> ClipboardResult<()>;

    fn read(&self, format: &MimeType) -> ClipboardResult<ClipboardData>;

    fn write(&self, data: ClipboardData) -> ClipboardResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_data_reports_text_mime_type() {
        let data = ClipboardData::text("hello");

        assert_eq!(data.mime_type(), MimeType::text_plain());
    }

    #[test]
    fn binary_data_preserves_mime_type_and_bytes() {
        let mime_type = MimeType::new("text/html").expect("valid MIME type");
        let data = ClipboardData::binary(mime_type.clone(), vec![1, 2, 3]);

        assert_eq!(data.mime_type(), mime_type);
        assert_eq!(data.as_bytes(), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn capabilities_default_to_unsupported() {
        assert_eq!(
            ClipboardCapabilities::default(),
            ClipboardCapabilities::unsupported()
        );
    }
}
