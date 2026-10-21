// SPDX-FileCopyrightText: 2023 Siemens AG
// SPDX-License-Identifier: MIT

//! Viewport queries adapted from pinned iX utils/breakpoints.ts and
//! utils/application-layout/service.ts. Rust resolves raw matchMedia facts.
//! Owners are isolated: this does not reproduce the upstream global service or
//! Menu.pinned's global override. Applications pass Some(supported), while
//! descendant Headers pass (None, None) to reuse their nearest owner. SSR starts
//! Large unless forced. Empty supported lists safely fall back to Large.

use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ApplicationBreakpoint;
use crate::document_viewport::{DocumentMediaFacts, DocumentViewportFacts, use_document_viewport};

const DEFAULT_SUPPORTED: [ApplicationBreakpoint; 3] = [
    ApplicationBreakpoint::Small,
    ApplicationBreakpoint::Medium,
    ApplicationBreakpoint::Large,
];

fn resolve(
    force: Option<ApplicationBreakpoint>,
    supported: Option<&[ApplicationBreakpoint]>,
    facts: Option<DocumentMediaFacts>,
) -> ApplicationBreakpoint {
    if let Some(force) = force {
        return force;
    }
    let Some(facts) = facts else {
        return ApplicationBreakpoint::Large;
    };
    let supported = supported.unwrap_or(&DEFAULT_SUPPORTED);
    if let Some(found) = supported.iter().rev().find(|&&item| match item {
        ApplicationBreakpoint::Small => facts.sm,
        ApplicationBreakpoint::Medium => facts.md,
        ApplicationBreakpoint::Large => facts.lg,
    }) {
        return *found;
    }
    if supported.contains(&ApplicationBreakpoint::Large) {
        supported.first().copied()
    } else {
        supported.last().copied()
    }
    .unwrap_or(ApplicationBreakpoint::Large)
}

struct ViewportModel {
    alive: bool,
    observed: bool,
    force: Option<ApplicationBreakpoint>,
    supported: Option<Vec<ApplicationBreakpoint>>,
    facts: Option<DocumentMediaFacts>,
    viewport_size: Option<(f64, f64)>,
}
impl ViewportModel {
    fn new(
        force: Option<ApplicationBreakpoint>,
        supported: Option<Vec<ApplicationBreakpoint>>,
    ) -> Self {
        Self {
            alive: true,
            observed: false,
            force,
            supported,
            facts: None,
            viewport_size: None,
        }
    }

    fn breakpoint(&self) -> ApplicationBreakpoint {
        resolve(self.force, self.supported.as_deref(), self.facts)
    }

    // Only plain model data changes during render. Signal notification is queued
    // until after the render, so existing descendant readers are also updated.
    fn sync_inputs(
        &mut self,
        force: Option<ApplicationBreakpoint>,
        supported: Option<Vec<ApplicationBreakpoint>>,
    ) -> bool {
        if !self.alive || (self.force == force && self.supported == supported) {
            return false;
        }
        self.force = force;
        self.supported = supported;
        true
    }

    fn observe(&mut self, facts: DocumentViewportFacts) -> bool {
        if !self.alive {
            return false;
        }
        // Geometry remains useful even when the native matchMedia capability
        // is unavailable. Raw media matches never replace logical owner inputs.
        let viewport_size = (facts.width.is_finite()
            && facts.width > 0.0
            && facts.height.is_finite()
            && facts.height > 0.0)
            .then_some((facts.width, facts.height));
        let changed =
            !self.observed || self.facts != facts.media || self.viewport_size != viewport_size;
        self.observed = true;
        self.facts = facts.media;
        self.viewport_size = viewport_size;
        changed
    }

    fn invalidate(&mut self) {
        self.alive = false;
        self.viewport_size = None;
    }
}

#[derive(Clone)]
pub(crate) struct ApplicationViewport {
    model: Rc<RefCell<ViewportModel>>,
    revision: Signal<u64>,
}
impl ApplicationViewport {
    pub(crate) fn is_ready(&self) -> bool {
        let _ = (self.revision)();
        let model = self.model.borrow();
        model.force.is_some() || model.observed
    }

    pub(crate) fn breakpoint(&self) -> ApplicationBreakpoint {
        let _ = (self.revision)();
        self.model.borrow().breakpoint()
    }

    /// Raw document viewport dimensions, unavailable before native observation
    /// (including SSR). Element-based layouts should use mounted measurements.
    pub(crate) fn viewport_size(&self) -> Option<(f64, f64)> {
        let _ = (self.revision)();
        self.model.borrow().viewport_size
    }
}

/// Some(supported) establishes an Application owner even below another owner.
/// (None, None) reuses an ancestor; without one it owns a standalone policy.
/// Logical owners share the root document observation rather than native streams.
/// This ownership role stays fixed for the lifetime of this hook invocation.
pub(crate) fn use_application_viewport(
    force: Option<ApplicationBreakpoint>,
    supported: Option<Vec<ApplicationBreakpoint>>,
) -> ApplicationViewport {
    let inherited = use_hook(try_consume_context::<ApplicationViewport>);
    let owns = use_hook(|| inherited.is_none() || force.is_some() || supported.is_some());
    let document = use_document_viewport(owns);
    let revision = use_signal(|| 0_u64);
    let model = use_hook(|| {
        if owns {
            Rc::new(RefCell::new(ViewportModel::new(force, supported.clone())))
        } else {
            inherited
                .as_ref()
                .expect("inherited viewport")
                .model
                .clone()
        }
    });
    let revision = if owns {
        revision
    } else {
        inherited.as_ref().expect("inherited viewport").revision
    };
    let viewport = ApplicationViewport {
        model: model.clone(),
        revision,
    };
    use_hook(|| provide_context(viewport.clone()));

    if owns && model.borrow_mut().sync_inputs(force, supported) {
        let current = model.clone();
        dioxus::core::queue_effect(move || {
            let alive = current.borrow().alive;
            if alive {
                let mut revision = revision;
                let next = revision.peek().wrapping_add(1);
                revision.set(next);
            }
        });
    }

    use_effect({
        let model = model.clone();
        move || {
            if !owns {
                return;
            }
            if let Some(facts) = document.facts() {
                let changed = model.borrow_mut().observe(facts);
                if changed {
                    let mut revision = revision;
                    let next = revision.peek().wrapping_add(1);
                    revision.set(next);
                }
            }
        }
    });
    use_drop({
        let model = model.clone();
        move || {
            if owns {
                model.borrow_mut().invalidate();
            }
        }
    });
    viewport
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod pane_layout_tests;
