//! The widgets several tools' views share (core plan, "Shell layout", "Crate
//! structure"): `BottomDock`, the collapsible bottom panel's frame, `LogPanel`,
//! the run log drawn in it, and the settings rows (`settings_row`) the menu's
//! sections share. Where they sit in a tool's view is that tool's choice;
//! the widgets themselves know no tool.

mod dock;
mod log_panel;
mod settings_row;

pub use dock::BottomDock;
pub use log_panel::{LogLine, LogPanel};
pub(crate) use settings_row::row_label;
pub use settings_row::{PathKind, path_row};
