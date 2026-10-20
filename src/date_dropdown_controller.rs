//! Read-only access to a DateDropdown's current Rust selection.

use crate::datetime::{DateRangeChange, DateSelection};
use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_BINDING_ID: AtomicUsize = AtomicUsize::new(1);

/// A range query requires exactly one live DateDropdown component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateDropdownError {
    /// No live DateDropdown is attached to this controller.
    NotMounted,
    /// More than one live DateDropdown is attached to this controller.
    MultipleMounted,
}

impl std::fmt::Display for DateDropdownError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::NotMounted => "DateDropdown controller has no mounted dropdown",
            Self::MultipleMounted => "DateDropdown controller has multiple mounted dropdowns",
        })
    }
}

impl std::error::Error for DateDropdownError {}

#[derive(Clone)]
struct DateDropdownBinding {
    alive: Rc<Cell<bool>>,
    read: Rc<dyn Fn() -> Option<DateRangeChange>>,
}

/// Read the current selection of an attached [`crate::DateDropdown`].
///
/// Clones share one handle. Attach the handle through the component's
/// `controller` prop. A query includes unpublished calendar edits and works
/// while the dropdown is closed. It does not publish or dismiss the selection.
/// Attaching the same handle to multiple live components makes queries
/// ambiguous and returns [`DateDropdownError::MultipleMounted`].
#[derive(Clone, Default)]
pub struct DateDropdownController {
    bindings: Rc<RefCell<BTreeMap<usize, DateDropdownBinding>>>,
}

impl PartialEq for DateDropdownController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.bindings, &other.bindings)
    }
}

impl DateDropdownController {
    /// Return an owned snapshot of the current range ID and typed dates.
    ///
    /// Reading does not subscribe the caller to changes, emit `onrangechange`,
    /// or change focus, opening, closing, or component state. The component's
    /// Rust scope must still be live; no native DOM handle is required.
    pub fn get_date_range(&self) -> Result<DateRangeChange, DateDropdownError> {
        let binding = {
            let bindings = self.bindings.borrow();
            let mut mounted = bindings.values().filter(|binding| binding.alive.get());
            let Some(binding) = mounted.next() else {
                return Err(DateDropdownError::NotMounted);
            };
            if mounted.next().is_some() {
                return Err(DateDropdownError::MultipleMounted);
            }
            binding.clone()
        };
        (binding.read)().ok_or(DateDropdownError::NotMounted)
    }
}

/// Create a stable DateDropdown controller for this component.
pub fn use_date_dropdown() -> DateDropdownController {
    use_hook(DateDropdownController::default)
}

pub(crate) fn use_date_dropdown_binding(
    requested: Option<&DateDropdownController>,
    draft: Signal<DateSelection>,
    draft_id: Signal<String>,
) {
    let binding_id = use_hook(|| NEXT_BINDING_ID.fetch_add(1, Ordering::Relaxed));
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let current = use_hook(|| Rc::new(RefCell::new(None::<DateDropdownController>)));
    let previous = current.borrow().clone();
    if previous.as_ref() != requested {
        if let Some(previous) = previous {
            previous.bindings.borrow_mut().remove(&binding_id);
        }
        *current.borrow_mut() = requested.cloned();
    }
    if let Some(requested) = requested {
        requested.bindings.borrow_mut().insert(
            binding_id,
            DateDropdownBinding {
                alive: alive.clone(),
                read: Rc::new(move || {
                    let dates = *draft.try_peek().ok()?;
                    let id = draft_id.try_peek().ok()?.clone();
                    Some(DateRangeChange { id, dates })
                }),
            },
        );
    }
    use_drop(move || {
        alive.set(false);
        if let Some(controller) = current.borrow_mut().take() {
            controller.bindings.borrow_mut().remove(&binding_id);
        }
    });
}
