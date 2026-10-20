//! Ordinary Dioxus composition for an active grid editing session.
use crate::DataGridCellContext;
use dioxus::prelude::*;
use dioxus::signals::{Owner, UnsyncStorage};
use std::{cell::Cell, rc::Rc};

// Dioxus callbacks are scope-owned. Retained contexts keep their existing
// allocation owner alive so the guarded callback can safely no-op after the
// editor scope is removed; this allocates no parallel callback registry.
#[derive(Clone)]
pub(crate) struct EditorCallbackOwner {
    _owner: Owner<UnsyncStorage>,
}
impl EditorCallbackOwner {
    pub(crate) fn capture() -> Self {
        Self {
            _owner: dioxus::core::Runtime::current().current_owner(),
        }
    }
}
impl PartialEq for EditorCallbackOwner {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Current editing data supplied to `DataGrid::render_editor`.
///
/// Return `Some(Element)` to compose an editor, or `None` for the column's native
/// editor. Put hooks inside composed components, not in the renderer callback.
/// Attach `onmounted` to the actual primary native control for Dioxus focus;
/// missing registration never triggers DOM discovery or a JavaScript fallback.
/// Custom text selection remains under the composed control's ownership.
///
/// `active` describes this rendered snapshot. Retiring controls should become
/// read-only and leave the Tab order. Keep a retiring focused button physically
/// enabled with `aria-disabled` and guarded actions until the grid completes
/// its mounted focus handoff; native `disabled` would blur it prematurely.
/// Methods also guard the session identity,
/// so retaining an old context cannot affect a later edit of the same cell.
/// The grid supplies required/type validation and its shared error message.
#[derive(Clone, PartialEq)]
pub struct DataGridEditorContext {
    pub cell: DataGridCellContext,
    pub draft: String,
    pub error: Option<String>,
    pub label: String,
    pub error_id: Option<String>,
    pub active: bool,
    pub onmounted: EventHandler<MountedEvent>,
    pub(crate) callback_owner: EditorCallbackOwner,
    pub(crate) alive: Rc<Cell<bool>>,
    pub(crate) set_value_handler: EventHandler<String>,
    pub(crate) commit_handler: EventHandler<()>,
    pub(crate) cancel_handler: EventHandler<()>,
    pub(crate) advance_handler: EventHandler<bool>,
}
impl DataGridEditorContext {
    /// Update the draft and clear its previous validation error.
    pub fn set_value(&self, value: String) {
        if self.alive.get() {
            self.set_value_handler.call(value);
        }
    }
    /// Validate and request a save, returning focus to the cell on success.
    pub fn commit(&self) {
        if self.alive.get() {
            self.commit_handler.call(());
        }
    }
    /// Discard this edit and return to the cell.
    pub fn cancel(&self) {
        if self.alive.get() {
            self.cancel_handler.call(());
        }
    }
    /// Validate/save and edit the adjacent editable cell, respecting row order.
    pub fn advance(&self, backwards: bool) {
        if self.alive.get() {
            self.advance_handler.call(backwards);
        }
    }
}
