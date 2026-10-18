//! Source submenu assignment, captured trigger markers and explicit return.
use super::*;

impl RuntimeHandle {
    pub(super) fn wake_submenu_markers(&self) {
        self.0.borrow_mut().item_cues_dirty = true;
        if let Some(bridge) = self.0.borrow().bridge {
            let _ = bridge.send(serde_json::json!({"item_cues":true}));
        }
    }
    pub(super) fn permits_submenu_marker(&self, offer: &SubmenuMarkerOffer) -> bool {
        let Some(entry) = self.entry(&offer.root) else {
            return false;
        };
        if entry.generation != offer.generation || self.read_current(&offer.root, &entry).is_none()
        {
            return false;
        }
        let state = self.0.borrow();
        if offer.capture.is_empty()
            || !state
                .item_root_captures
                .get(&offer.root)
                .is_some_and(|(generation, capture, _)| {
                    *generation == offer.generation
                        && !capture.is_empty()
                        && *capture == offer.root_capture
                })
        {
            return false;
        }
        let active = state
            .submenu_marks
            .get(&offer.capture)
            .is_some_and(|(root, generation)| {
                state
                    .nodes
                    .get(root)
                    .is_some_and(|entry| entry.generation == *generation)
            });
        offer.active == active
    }
    pub(super) fn first_submenu_marker(&self, root: &str, generation: u64) -> Option<String> {
        let state = self.0.borrow();
        let (_, facts) = state
            .submenu_own
            .get(root)
            .filter(|(version, _)| *version == generation)?;
        let matching = |order: &[ItemRef]| -> Vec<ItemRef> {
            order
                .iter()
                .filter(|item| {
                    facts.hosts.iter().any(|host| {
                        host.item == **item && state.submenu_marks.contains_key(&host.capture)
                    })
                })
                .cloned()
                .collect()
        };
        let mut order = matching(&facts.query_order);
        if let Some(slots) = &facts.slots {
            for slot in slots.iter().rev() {
                let mut assigned = Vec::new();
                for assignment in &slot.assignments {
                    if let Some(root) = &assignment.root
                        && !matching(std::slice::from_ref(root)).is_empty()
                    {
                        assigned.push(root.clone());
                    } else {
                        assigned.extend(matching(&assignment.descendants));
                    }
                }
                let at = order
                    .iter()
                    .position(|item| slot.following.contains(item))
                    .unwrap_or(order.len());
                order.splice(at..at, assigned);
            }
        }
        order
            .first()
            .and_then(|item| facts.hosts.iter().find(|host| host.item == *item))
            .map(|host| host.capture.clone())
    }
    #[cfg(test)]
    pub(super) fn queue_submenu_return(&self, root: &str, generation: u64, capture: String) {
        self.queue_submenu_return_after_hide(root, generation, capture, Vec::new());
    }
    pub(super) fn queue_submenu_return_after_hide(
        &self,
        root: &str,
        generation: u64,
        capture: String,
        hides: Vec<crate::overlay_descendant_focus::DescendantHide>,
    ) {
        let admitted = self
            .with_owner(root, generation, || {
                let Some(entry) = self.entry(root) else {
                    return false;
                };
                if self.read_current(root, &entry).is_none() {
                    return false;
                }
                let mut state = self.0.borrow_mut();
                let reference = state
                    .item_root_captures
                    .get(root)
                    .filter(|(version, _, _)| *version == generation)
                    .map(|(_, capture, _)| capture.clone());
                if hides.is_empty() {
                    state.descendant_focus.admit_return(
                        root.into(),
                        generation,
                        0,
                        reference,
                        capture,
                    );
                } else {
                    state.descendant_focus.admit_return_after_hide(
                        root.into(),
                        generation,
                        0,
                        reference,
                        capture,
                        hides,
                    );
                }
                true
            })
            .unwrap_or(false);
        if admitted {
            self.flush_descendant_focus();
        }
    }
}

impl NodeHandle {
    pub(crate) fn set_submenu_access(
        &self,
        handle: crate::dropdown::DropdownHandle,
        query_root: Rc<dyn Fn() -> Option<String>>,
        reset: Rc<dyn Fn()>,
        trigger_click: Rc<dyn Fn()>,
    ) {
        if let Some(runtime) = self.runtime()
            && let Some(entry) = runtime
                .0
                .borrow_mut()
                .nodes
                .get_mut(&self.id)
                .filter(|entry| entry.generation == self.generation)
        {
            entry.submenu = Some(SubmenuAccess {
                handle,
                query_root,
                reset,
                trigger_click,
            });
        }
    }
    pub(crate) fn submenu_receiver(
        &self,
        host: &HostFacts,
    ) -> Option<crate::dropdown::DropdownHandle> {
        let runtime = self.runtime()?;
        if host.root_path.is_empty() {
            return runtime
                .entry(&self.id)
                .filter(|entry| entry.generation == self.generation)?
                .submenu
                .map(|access| access.handle);
        }
        host.root_path
            .iter()
            .filter(|root| !(root.is_host && host.kind == ItemKind::Category))
            .find_map(|root| {
                runtime
                    .entry(&root.id)
                    .filter(|entry| entry.generation == root.generation && entry.kind.is_dropdown())
                    .map(|entry| entry.submenu.map(|access| access.handle))
            })
            .flatten()
    }
    pub(crate) fn first_assigned_submenu(&self) -> Option<crate::dropdown::DropdownHandle> {
        let runtime = self.runtime()?;
        runtime
            .entry(&self.id)
            .filter(|entry| entry.generation == self.generation)?;
        let first = runtime
            .0
            .borrow()
            .registry
            .assigned_children(&self.id)
            .first()?
            .clone();
        runtime.entry(&first)?.submenu.map(|access| access.handle)
    }
    pub(crate) fn mark_submenu_trigger(&self, host: &HostFacts) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if host.capture.is_empty() {
            return;
        }
        let admitted = self
            .with_owner(|| {
                if !runtime
                    .entry(&self.id)
                    .is_some_and(|entry| entry.generation == self.generation)
                {
                    return false;
                }
                runtime
                    .0
                    .borrow_mut()
                    .submenu_marks
                    .insert(host.capture.clone(), (self.id.clone(), self.generation));
                true
            })
            .unwrap_or(false);
        if admitted {
            runtime.wake_submenu_markers();
        }
    }
    pub(crate) fn queue_submenu_entry(&self, focus_version: u64) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        let admitted = runtime
            .with_owner(&self.id, self.generation, || {
                let Some(entry) = runtime.entry(&self.id) else {
                    return false;
                };
                if runtime.read_current(&self.id, &entry).is_none() {
                    return false;
                }
                let mut state = runtime.0.borrow_mut();
                let reference = state
                    .item_root_captures
                    .get(&self.id)
                    .filter(|(version, _, _)| *version == self.generation)
                    .map(|(_, capture, _)| capture.clone());
                state.descendant_focus.admit_submenu(
                    self.id.clone(),
                    self.generation,
                    focus_version,
                    reference,
                );
                true
            })
            .unwrap_or(false);
        if admitted {
            runtime.flush_descendant_focus();
        }
    }
}
