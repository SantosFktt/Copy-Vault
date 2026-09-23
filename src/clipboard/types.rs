use std::fmt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MimeType(String);

impl MimeType {
    pub fn new(value: impl Into<String>) -> Result<Self, ClipboardError> {
        let value = value.into();
        if value.is_empty() || !value.contains('/') || value.chars().any(char::is_whitespace) {
            return Err(ClipboardError::InvalidMimeType(value));
        }

        Ok(Self(value))
    }

    pub fn text_plain() -> Self {
        Self("text/plain".to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MimeType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClipboardData {
    Text(String),
    Binary { mime_type: MimeType, bytes: Vec<u8> },
}

impl ClipboardData {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn binary(mime_type: MimeType, bytes: Vec<u8>) -> Self {
        Self::Binary { mime_type, bytes }
    }

    pub fn mime_type(&self) -> MimeType {
        match self {
            Self::Text(_) => MimeType::text_plain(),
            Self::Binary { mime_type, .. } => mime_type.clone(),
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Text(_) => None,
            Self::Binary { bytes, .. } => Some(bytes),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardEvent {
    pub formats: Vec<MimeType>,
    pub data: Option<ClipboardData>,
}

impl ClipboardEvent {
    pub fn new(formats: Vec<MimeType>) -> Self {
        Self {
            formats,
            data: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardCapabilities {
    pub can_read: bool,
    pub can_write: bool,
    pub can_discover_formats: bool,
    pub can_monitor_changes: bool,
    pub can_monitor_in_background: bool,
    pub can_read_rich_text: bool,
}

impl ClipboardCapabilities {
    pub const fn unsupported() -> Self {
        Self {
            can_read: false,
            can_write: false,
            can_discover_formats: false,
            can_monitor_changes: false,
            can_monitor_in_background: false,
            can_read_rich_text: false,
        }
    }
}

impl Default for ClipboardCapabilities {
    fn default() -> Self {
        Self::unsupported()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClipboardError {
    Unsupported { operation: &'static str },
    DisplayUnavailable,
    TransferFailed(String),
    Cancelled,
    InvalidMimeType(String),
}

pub type ClipboardResult<T> = Result<T, ClipboardError>;
