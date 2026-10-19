//! Source submenu assignment, captured trigger markers and explicit return.
use super::*;

impl RuntimeHandle {
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
}

impl NodeHandle {
    pub(crate) fn set_submenu_access(
        &self,
        handle: crate::dropdown::DropdownHandle,
        query_root: Rc<dyn Fn() -> Option<String>>,
        reset: Rc<dyn Fn()>,
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
    pub(crate) fn linked_submenu(
        &self,
        host: &HostFacts,
    ) -> Option<crate::dropdown::DropdownHandle> {
        let runtime = self.runtime()?;
        let owner = runtime
            .entry(&self.id)
            .filter(|entry| entry.generation == self.generation)?;
        runtime
            .with_owner(&self.id, self.generation, || {
                runtime.read_current(&self.id, &owner)
            })
            .flatten()?;
        if host.disabled_attribute || host.hidden_attribute || !self.is_item_registered(host) {
            return None;
        }
        let linked = host.linked_trigger.as_deref()?;
        let child = runtime
            .entry(linked)
            .filter(|entry| entry.kind == OverlayKind::Dropdown)?;
        runtime
            .with_owner(linked, child.generation, || {
                runtime.read_current(linked, &child)
            })
            .flatten()?;
        let parent = if let Some(domain) = runtime.domain() {
            if domain.node_kind(&self.id, self.generation).is_none()
                || domain.node_kind(linked, child.generation) != Some(OverlayKind::Dropdown)
            {
                return None;
            }
            domain.cached_parent(linked)
        } else {
            runtime.0.borrow().registry.cached_parent(linked)
        };
        if parent.as_deref() != Some(&self.id) {
            return None;
        }
        let current = runtime
            .0
            .borrow()
            .submenu_own
            .get(&self.id)
            .filter(|(generation, _)| *generation == self.generation)
            .is_some_and(|(_, facts)| {
                facts.hosts.iter().any(|current| {
                    current.item == host.item
                        && current.target == host.target
                        && current.registration_key == host.registration_key
                        && current.linked_trigger == host.linked_trigger
                })
            });
        if !current {
            return None;
        }
        child.submenu.map(|access| access.handle)
    }
    pub(crate) fn mark_submenu_trigger(&self, host: &HostFacts) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if host.capture.is_empty() {
            return;
        }
        let _ = self.with_owner(|| {
            if !runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
            {
                return;
            }
            runtime
                .0
                .borrow_mut()
                .submenu_marks
                .insert(host.capture.clone(), (self.id.clone(), self.generation));
        });
    }
}
