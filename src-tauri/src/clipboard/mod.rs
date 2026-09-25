pub mod exclusion;
pub mod foreground;
pub mod notify;
pub mod watcher;
pub mod writer;

pub use watcher::start_watcher;
pub use writer::write_to_clipboard;
