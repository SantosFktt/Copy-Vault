//! Platform provider slots.
//!
//! Implementations are intentionally absent until the platform-specific
//! clipboard behavior has been approved and validated.

pub mod unsupported;
pub mod wayland;
pub mod x11;
