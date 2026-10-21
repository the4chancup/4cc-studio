//! The widgets several tools' views share (core plan, "Shell layout", "Crate
//! structure"): `BottomDock`, the collapsible bottom panel's frame, and `LogPanel`,
//! the run log drawn in it. Where they sit in a tool's view is that tool's choice;
//! the widgets themselves know no tool.

mod dock;
mod log_panel;

pub use dock::BottomDock;
pub use log_panel::{LogLine, LogPanel};
