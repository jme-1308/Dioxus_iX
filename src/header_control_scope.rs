// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Explicit Header composition groups for already mounted Dioxus controls.
//! The overlay runtime owns element handles; shared native focus facts supply
//! eligibility and document order. This metadata never discovers DOM controls.

use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::{Rc, Weak},
};

#[derive(Clone)]
struct Control {
    focus_id: String,
    group: u8,
    live: Rc<Cell<bool>>,
}

#[derive(Clone)]
pub(crate) struct HeaderControls {
    controls: Rc<RefCell<BTreeMap<String, Control>>>,
    onchange: Rc<dyn Fn()>,
    owner: ScopeId,
    owner_live: CopyValue<()>,
    renderer: Weak<dioxus::core::Runtime>,
}
impl PartialEq for HeaderControls {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.controls, &other.controls)
    }
}
impl HeaderControls {
    pub(crate) fn new(onchange: Rc<dyn Fn()>) -> Self {
        Self {
            controls: Rc::new(RefCell::new(BTreeMap::new())),
            onchange,
            owner: dioxus::core::current_scope_id(),
            owner_live: CopyValue::new(()),
            renderer: Rc::downgrade(&dioxus::core::Runtime::current()),
        }
    }

    pub(crate) fn groups(&self) -> BTreeMap<String, u8> {
        self.controls
            .borrow()
            .values()
            .filter(|control| !control.focus_id.is_empty())
            .map(|control| (control.focus_id.clone(), control.group))
            .collect()
    }

    fn changed(&self) {
        // Pinned Runtime exposes no public scope-exists query. Its scope-owned
        // CopyValue is invalidated on teardown, including reused numeric IDs.
        if self.owner_live.try_read().is_err() {
            return;
        }
        let Some(renderer) = self.renderer.upgrade() else {
            return;
        };
        let onchange = self.onchange.clone();
        // Child teardown must invalidate the surviving Header, not enqueue an
        // effect owned by the child scope that is currently being removed.
        renderer.in_scope(self.owner, || {
            dioxus::core::queue_effect(move || onchange());
        });
    }
}

#[derive(Clone)]
pub(crate) struct HeaderControlGroup {
    pub(crate) controls: HeaderControls,
    pub(crate) group: u8,
}

#[derive(Clone)]
pub(crate) struct HeaderControlRegistration {
    controls: HeaderControls,
    key: String,
    group: u8,
    live: Rc<Cell<bool>>,
}
impl HeaderControlRegistration {
    pub(crate) fn update(&self, focus_id: &str) {
        if !self.live.get() {
            return;
        }
        // An explicit empty ID has no programmatic focus destination. Remove
        // its metadata without retiring the mounted registration itself.
        if focus_id.is_empty() {
            self.remove_current();
            return;
        }
        let changed = {
            let mut controls = self.controls.controls.borrow_mut();
            let changed = controls
                .get(&self.key)
                .is_none_or(|control| control.focus_id != focus_id || control.group != self.group);
            controls.insert(
                self.key.clone(),
                Control {
                    focus_id: focus_id.into(),
                    group: self.group,
                    live: self.live.clone(),
                },
            );
            changed
        };
        if changed {
            self.controls.changed();
        }
    }

    fn unregister(&self) {
        self.live.set(false);
        self.remove_current();
    }

    fn remove_current(&self) {
        let changed = {
            let mut controls = self.controls.controls.borrow_mut();
            if controls
                .get(&self.key)
                .is_some_and(|control| Rc::ptr_eq(&control.live, &self.live))
            {
                controls.remove(&self.key);
                true
            } else {
                false
            }
        };
        if changed {
            self.controls.changed();
        }
    }
}

/// The calling component's mounted lifecycle enrolls its current physical ID.
/// The nearest Header group owns only the composition metadata, never a second
/// element handle or a fallback for unregistered consumer content.
pub(crate) fn use_header_control_registration(key: String) -> Option<HeaderControlRegistration> {
    let registration = use_hook(move || {
        try_consume_context::<HeaderControlGroup>().map(|group| HeaderControlRegistration {
            controls: group.controls,
            key,
            group: group.group,
            live: Rc::new(Cell::new(true)),
        })
    });
    let cleanup = registration.clone();
    use_drop(move || {
        if let Some(registration) = cleanup {
            registration.unregister();
        }
    });
    registration
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Fixture {
        present: Rc<Cell<bool>>,
        focus_id: Rc<RefCell<String>>,
        changes: Rc<Cell<usize>>,
        controls: Rc<RefCell<Option<HeaderControls>>>,
        registration: Rc<RefCell<Option<HeaderControlRegistration>>>,
    }
    impl PartialEq for Fixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.present, &other.present)
        }
    }

    #[component]
    fn RegisteredControl(focus_id: String, fixture: Fixture) -> Element {
        let current = use_header_control_registration("header-control".into()).unwrap();
        current.update(&focus_id);
        *fixture.registration.borrow_mut() = Some(current);
        rsx! { button { id: focus_id } }
    }

    fn fixture(fixture: Fixture) -> Element {
        let changes = fixture.changes.clone();
        let controls =
            use_hook(|| HeaderControls::new(Rc::new(move || changes.set(changes.get() + 1))));
        *fixture.controls.borrow_mut() = Some(controls.clone());
        use_context_provider(|| HeaderControlGroup { controls, group: 2 });
        let focus_id = fixture.focus_id.borrow().clone();
        rsx! {
            if fixture.present.get() {
                RegisteredControl { focus_id, fixture: fixture.clone() }
            }
        }
    }

    fn settle(dom: &mut VirtualDom) {
        for _ in 0..4 {
            dom.render_immediate_to_vec();
            dom.process_events();
        }
    }

    #[test]
    fn current_control_id_and_group_follow_scope_lifetime_without_duplicate_notifications() {
        let state = Fixture {
            present: Rc::new(Cell::new(true)),
            focus_id: Rc::new(RefCell::new("first-control".into())),
            changes: Default::default(),
            controls: Default::default(),
            registration: Default::default(),
        };
        let mut dom = VirtualDom::new_with_props(fixture, state.clone());
        dom.rebuild_in_place();
        settle(&mut dom);
        let controls = state.controls.borrow().clone().unwrap();
        let registration = state.registration.borrow().clone().unwrap();
        assert_eq!(
            controls.groups(),
            BTreeMap::from([("first-control".into(), 2)])
        );
        assert_eq!(state.changes.get(), 1);
        dom.in_scope(ScopeId::APP, || registration.update("first-control"));
        settle(&mut dom);
        assert_eq!(state.changes.get(), 1);

        dom.in_scope(ScopeId::APP, || registration.update(""));
        settle(&mut dom);
        assert!(controls.groups().is_empty());
        assert_eq!(state.changes.get(), 2);
        dom.in_scope(ScopeId::APP, || registration.update(""));
        settle(&mut dom);
        assert_eq!(state.changes.get(), 2);
        dom.in_scope(ScopeId::APP, || registration.update("first-control"));
        settle(&mut dom);
        assert_eq!(
            controls.groups(),
            BTreeMap::from([("first-control".into(), 2)])
        );
        assert_eq!(state.changes.get(), 3);

        *state.focus_id.borrow_mut() = "current-control".into();
        dom.mark_dirty(ScopeId::APP);
        settle(&mut dom);
        assert_eq!(
            controls.groups(),
            BTreeMap::from([("current-control".into(), 2)])
        );
        assert_eq!(state.changes.get(), 4);

        state.present.set(false);
        dom.mark_dirty(ScopeId::APP);
        settle(&mut dom);
        assert!(controls.groups().is_empty());
        assert_eq!(
            state.changes.get(),
            5,
            "child cleanup notifies the surviving Header"
        );
        dom.in_scope(ScopeId::APP, || registration.update("stale-control"));
        settle(&mut dom);
        assert!(controls.groups().is_empty());
        assert_eq!(state.changes.get(), 5);
        drop(dom);
        controls.changed();
        assert_eq!(state.changes.get(), 5);
    }
}
