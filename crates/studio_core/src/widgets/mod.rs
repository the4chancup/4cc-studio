//! The widgets several tools' views share: `BottomDock`, the collapsible bottom
//! panel's frame, `LogPanel`,
//! the run log drawn in it, the settings rows (`settings_row`) the menu's
//! sections share, and the checkbox and radio button every view draws
//! (`choice`). Where they sit in a tool's view is that tool's choice;
//! the widgets themselves know no tool.

mod choice;
mod dock;
mod log_panel;
mod menu_button;
mod settings_row;

pub use choice::{checkbox, checkbox_without_text, radio, radio_value};
pub use dock::BottomDock;
pub use log_panel::{LogLine, LogPanel};
pub use menu_button::menu_button;
pub(crate) use settings_row::{PathCommit, path_row_commit, row_label};
pub use settings_row::{PathKind, path_row};
