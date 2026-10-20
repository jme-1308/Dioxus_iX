//! Dioxus composition for visible grid cells, without foreign renderer objects
//! or a second DOM lifecycle. Keep persistent per-row state in the consumer:
//! virtualized rows and built-in editing can unmount custom content normally.
use crate::{DataGridColumn, DataGridRow};
use dioxus::prelude::Element;

/// Current data supplied to `DataGrid::render_cell` on a visible cell render.
#[derive(Clone, Debug, PartialEq)]
pub struct DataGridCellContext {
    pub row: DataGridRow,
    pub column: DataGridColumn,
    pub value: String,
    /// Zero-based position in the displayed, sorted/filtered rows.
    pub row_index: usize,
    pub selected: bool,
    /// Whether grid-managed editing is enabled for this cell.
    /// Custom inputs can bind their read-only state to this value.
    pub editable: bool,
    /// The host's whole-grid read-only setting. Custom actions decide whether
    /// they mutate data and should disable themselves when this is true.
    pub grid_read_only: bool,
}

/// Ordinary Dioxus content for a cell. Return `None` from `render_cell` to keep
/// the built-in text presentation. Active native or composed editors replace
/// this content while the cell is being edited; sorting/filtering/autosizing still use the
/// row's stored text. Configure widths for custom layouts when necessary.
#[derive(Clone, PartialEq)]
pub struct DataGridCellRender {
    pub(crate) content: Element,
    pub(crate) interactive: bool,
}

impl DataGridCellRender {
    /// Visual content such as a status Pill, icon or formatted value. Normal
    /// cell arrow/Tab navigation and double-click editing remain available.
    pub fn display(content: Element) -> Self {
        Self {
            content,
            interactive: false,
        }
    }

    /// Embedded controls such as inputs, links, Buttons or Dropdowns. Tab from
    /// the cell reaches its mounted controls; their keys and native defaults
    /// are isolated from grid navigation/editing. An unhandled Escape returns
    /// to the cell through Dioxus mounted focus. A child may consume Escape.
    ///
    /// Native Tab visits mounted controls. Use cell navigation for offscreen
    /// virtualized rows. Compose components here and keep hooks inside those
    /// components rather than calling hooks directly in a rendering callback.
    pub fn interactive(content: Element) -> Self {
        Self {
            content,
            interactive: true,
        }
    }
}
