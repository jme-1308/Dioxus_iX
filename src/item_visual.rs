//! Validates registered typed hosts and native focus observations.
//! Rust selection is rendered by the owning Dioxus components.

use crate::dropdown_inventory::{HostFacts, HostPartKind, ItemKind};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub(crate) struct FocusObservation {
    #[serde(default)]
    pub(crate) hosts: Vec<HostFacts>,
    #[serde(default)]
    pub(crate) path: Vec<String>,
}

#[derive(Clone)]
struct HostBinding {
    kind: ItemKind,
    facts: Option<HostFacts>,
}

#[derive(Clone, Default)]
pub(crate) struct CueHosts {
    hosts: BTreeMap<String, HostBinding>,
}

impl CueHosts {
    /// Repeated registration of one live component does not replace its
    /// already-bound object. A new lifetime must unregister first.
    pub(crate) fn register(&mut self, key: String, kind: ItemKind) -> bool {
        if key.is_empty() {
            return false;
        }
        if let Some(binding) = self.hosts.get(&key) {
            return binding.kind == kind;
        }
        self.hosts.insert(key, HostBinding { kind, facts: None });
        true
    }

    pub(crate) fn unregister(&mut self, key: &str) {
        self.hosts.remove(key);
    }

    /// A new native capture namespace invalidates object evidence, while the
    /// same mounted Rust components retain their descriptive registrations.
    pub(crate) fn reset_captures(&mut self) {
        for binding in self.hosts.values_mut() {
            binding.facts = None;
        }
    }

    /// The runtime validates owner/document evidence before supplying these
    /// raw facts. A copied key cannot rebind a live host to another object.
    pub(crate) fn observe(&mut self, facts: &HostFacts) -> bool {
        let Some(binding) = self.hosts.get_mut(&facts.registration_key) else {
            return false;
        };
        if facts.capture.is_empty()
            || binding.kind != facts.kind
            || binding
                .facts
                .as_ref()
                .is_some_and(|previous| previous.capture != facts.capture)
        {
            return false;
        }
        binding.facts = Some(facts.clone());
        true
    }

    /// Validates the entire focus observation before committing any host
    /// bindings, then selects the first focus-aware typed host on the path.
    pub(crate) fn focus_host(
        &mut self,
        observation: &FocusObservation,
    ) -> Result<Option<HostFacts>, ()> {
        let mut staged = self.clone();
        for facts in &observation.hosts {
            if facts
                .parts
                .iter()
                .any(|part| part.capture.is_empty() || part.capture == facts.capture)
            {
                return Err(());
            }
            if staged.hosts.iter().any(|(key, binding)| {
                key != &facts.registration_key
                    && binding.facts.as_ref().is_some_and(|other| {
                        other.capture == facts.capture
                            || other.parts.iter().any(|part| part.capture == facts.capture)
                            || facts.parts.iter().any(|part| {
                                part.capture == other.capture
                                    || other
                                        .parts
                                        .iter()
                                        .any(|other_part| other_part.capture == part.capture)
                            })
                    })
            }) {
                return Err(());
            }
            if !staged.observe(facts) {
                return Err(());
            }
        }

        let mut selected = None;
        for capture in &observation.path {
            if capture.is_empty() {
                return Err(());
            }
            let mut matched = false;
            for facts in &observation.hosts {
                if &facts.capture == capture {
                    matched = true;
                    if selected.is_none()
                        && matches!(
                            facts.kind,
                            ItemKind::Dropdown | ItemKind::Select | ItemKind::Menu
                        )
                        && facts.source_focusable
                    {
                        selected = Some(facts.clone());
                    }
                    break;
                }
                if let Some(part) = facts.parts.iter().find(|part| &part.capture == capture) {
                    matched = true;
                    if selected.is_none() {
                        match (facts.kind, part.kind) {
                            (ItemKind::Select, HostPartKind::SelectOption)
                                if facts.source_focusable =>
                            {
                                selected = Some(facts.clone());
                            }
                            (ItemKind::Category, HostPartKind::CategoryParent)
                                if part.source_focusable =>
                            {
                                let mut host = facts.clone();
                                for candidate in &mut host.parts {
                                    if candidate.kind == HostPartKind::CategoryParent
                                        && &candidate.capture != capture
                                    {
                                        candidate.source_focusable = false;
                                    }
                                }
                                selected = Some(host);
                            }
                            _ => {}
                        }
                    }
                    break;
                }
            }
            if !matched {
                return Err(());
            }
        }

        self.hosts = staged.hosts;
        Ok(selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dropdown_inventory::{HostPartFacts, ItemRef};

    fn host(key: &str, capture: &str, kind: ItemKind) -> HostFacts {
        HostFacts {
            root_path: Vec::new(),
            item: ItemRef(capture.into()),
            kind,
            target: "same-public-id".into(),
            capture: capture.into(),
            registration_key: key.into(),
            source_focusable: true,
            disabled_attribute: false,
            hidden_attribute: false,
            tabindex_attribute: None,
            checked_attribute: false,
            visible_focus: false,
            roving_opt_in: false,
            linked_trigger: None,
            parts: Vec::new(),
        }
    }
    fn bind(registry: &mut CueHosts, host: &HostFacts) {
        assert!(registry.register(host.registration_key.clone(), host.kind));
        assert!(registry.observe(host));
    }

    #[test]
    fn exact_bound_identity_is_required_and_observation_cannot_retarget_same_id() {
        let mut registry = CueHosts::default();
        let original = host("source-a", "object-a", ItemKind::Dropdown);
        assert!(!registry.observe(&original));
        assert!(registry.register(original.registration_key.clone(), original.kind));
        assert!(registry.observe(&original));
        let copied = host("source-a", "object-b", ItemKind::Dropdown);
        assert_eq!(original.target, copied.target);
        assert!(!registry.observe(&copied));
        let foreign = host("other-runtime-key", "object-a", ItemKind::Dropdown);
        assert!(!registry.observe(&foreign));
        let mut wrong_kind = original.clone();
        wrong_kind.kind = ItemKind::Select;
        assert!(!registry.observe(&wrong_kind));
        assert!(!registry.register(original.registration_key.clone(), ItemKind::Select));
    }

    #[test]
    fn select_option_focus_selects_its_host_and_category_parent_stays_separate() {
        let mut registry = CueHosts::default();
        let mut select = host("select", "select-object", ItemKind::Select);
        select.parts.push(HostPartFacts {
            kind: HostPartKind::SelectOption,
            capture: "select-inner".into(),
            source_focusable: false,
        });
        bind(&mut registry, &select);
        assert_eq!(
            registry.focus_host(&FocusObservation {
                hosts: vec![select.clone()],
                path: vec!["select-inner".into(), select.capture.clone()],
            }),
            Ok(Some(select.clone()))
        );
        select.source_focusable = false;
        assert_eq!(
            registry.focus_host(&FocusObservation {
                hosts: vec![select.clone()],
                path: vec!["select-inner".into(), select.capture.clone()],
            }),
            Ok(None)
        );
        select.parts[0].kind = HostPartKind::CategoryParent;
        select.parts[0].source_focusable = true;
        assert_eq!(
            registry.focus_host(&FocusObservation {
                hosts: vec![select.clone()],
                path: vec!["select-inner".into()],
            }),
            Ok(None),
            "a part of another host kind cannot manufacture focus awareness"
        );
    }

    #[test]
    fn new_capture_namespace_requires_fresh_evidence_without_losing_live_registration() {
        let mut registry = CueHosts::default();
        let old = host("live-host", "old-document-object", ItemKind::Select);
        bind(&mut registry, &old);
        registry.reset_captures();
        assert!(
            registry
                .hosts
                .get("live-host")
                .is_some_and(|binding| binding.facts.is_none())
        );
        let fresh = host("live-host", "new-document-object", ItemKind::Select);
        assert!(registry.observe(&fresh));
        assert!(!registry.observe(&old));
        registry.unregister(&fresh.registration_key);
        assert!(registry.hosts.is_empty());
    }

    #[test]
    fn retargeted_category_host_clears_while_explicit_private_parent_attempt_marks() {
        let mut registry = CueHosts::default();
        let mut category = host("category", "category-object", ItemKind::Category);
        category.source_focusable = true;
        category.parts.push(HostPartFacts {
            kind: HostPartKind::CategoryParent,
            capture: "category-parent".into(),
            source_focusable: true,
        });
        assert!(registry.register(category.registration_key.clone(), category.kind));

        let parent_attempt = FocusObservation {
            hosts: vec![category.clone()],
            path: vec!["category-parent".into(), category.capture.clone()],
        };
        let focused = registry.focus_host(&parent_attempt).unwrap();
        assert_eq!(focused, Some(category.clone()));

        let retargeted_outer = FocusObservation {
            hosts: vec![category.clone()],
            path: vec![category.capture.clone()],
        };
        let cleared = registry.focus_host(&retargeted_outer).unwrap();
        assert_eq!(cleared, None);

        category.parts[0].source_focusable = false;
        let non_focusable_parent = FocusObservation {
            hosts: vec![category.clone()],
            path: vec!["category-parent".into(), category.capture.clone()],
        };
        assert_eq!(registry.focus_host(&non_focusable_parent), Ok(None));
    }

    #[test]
    fn plain_focus_target_clears_and_path_picks_first_focus_aware_host() {
        let mut registry = CueHosts::default();
        let mut category = host("category", "category-object", ItemKind::Category);
        category.source_focusable = true;
        category.parts.push(HostPartFacts {
            kind: HostPartKind::CategoryParent,
            capture: "category-parent".into(),
            source_focusable: true,
        });
        let dropdown = host("dropdown", "dropdown-object", ItemKind::Dropdown);
        assert!(registry.register(category.registration_key.clone(), category.kind));
        assert!(registry.register(dropdown.registration_key.clone(), dropdown.kind));

        let nested = FocusObservation {
            hosts: vec![category.clone(), dropdown.clone()],
            path: vec![category.capture.clone(), dropdown.capture.clone()],
        };
        let focused = registry.focus_host(&nested).unwrap();
        assert_eq!(focused, Some(dropdown.clone()));

        let plain = FocusObservation::default();
        let cleared = registry.focus_host(&plain).unwrap();
        assert_eq!(cleared, None);
    }

    #[test]
    fn menu_attempted_focus_marks_without_native_success_flag() {
        let mut registry = CueHosts::default();
        let mut menu = host("menu", "menu-object", ItemKind::Menu);
        menu.visible_focus = false;
        menu.disabled_attribute = true;
        menu.hidden_attribute = true;
        menu.tabindex_attribute = Some("-1".into());
        let native = host("native", "native-object", ItemKind::NativeOptIn);
        assert!(registry.register(menu.registration_key.clone(), menu.kind));
        assert!(registry.register(native.registration_key.clone(), native.kind));

        let attempt = FocusObservation {
            hosts: vec![menu.clone(), native.clone()],
            path: vec![native.capture.clone(), menu.capture.clone()],
        };
        let focused = registry.focus_host(&attempt).unwrap();
        assert_eq!(focused, Some(menu.clone()));

        menu.source_focusable = false;
        let disabled_cue = FocusObservation {
            hosts: vec![menu.clone()],
            path: vec![menu.capture.clone()],
        };
        let cleared = registry.focus_host(&disabled_cue).unwrap();
        assert_eq!(cleared, None);
    }

    #[test]
    fn stale_replacement_and_foreign_observations_reject_atomically() {
        let mut registry = CueHosts::default();
        let dropdown = host("source-a", "object-a", ItemKind::Dropdown);
        let menu = host("source-b", "object-b", ItemKind::Menu);
        assert!(registry.register(dropdown.registration_key.clone(), dropdown.kind));
        assert!(registry.register(menu.registration_key.clone(), menu.kind));

        let live = registry
            .focus_host(&FocusObservation {
                hosts: vec![dropdown.clone()],
                path: vec![dropdown.capture.clone()],
            })
            .unwrap();
        assert_eq!(live, Some(dropdown.clone()));

        let mut mutated_dropdown = dropdown.clone();
        mutated_dropdown.source_focusable = false;
        let foreign = host("foreign-key", "object-c", ItemKind::Dropdown);
        assert_eq!(
            registry.focus_host(&FocusObservation {
                hosts: vec![mutated_dropdown, foreign],
                path: vec![dropdown.capture.clone()],
            }),
            Err(())
        );
        assert_eq!(
            registry
                .hosts
                .get("source-a")
                .and_then(|b| b.facts.as_ref()),
            Some(&dropdown)
        );

        let replacement = host("source-a", "object-replacement", ItemKind::Dropdown);
        assert_eq!(
            registry.focus_host(&FocusObservation {
                hosts: vec![menu.clone(), replacement],
                path: vec![menu.capture.clone()],
            }),
            Err(())
        );
        assert!(
            registry
                .hosts
                .get("source-b")
                .is_some_and(|b| b.facts.is_none())
        );

        let hijacked = host("source-b", "object-a", ItemKind::Menu);
        assert_eq!(
            registry.focus_host(&FocusObservation {
                hosts: vec![hijacked],
                path: vec![dropdown.capture.clone()],
            }),
            Err(())
        );
        assert_eq!(
            registry.focus_host(&FocusObservation {
                hosts: vec![menu.clone()],
                path: vec!["stale-or-unknown-capture".into()],
            }),
            Err(())
        );
        assert!(
            registry
                .hosts
                .get("source-b")
                .is_some_and(|b| b.facts.is_none())
        );
        assert_eq!(
            registry.focus_host(&FocusObservation {
                hosts: Vec::new(),
                path: vec![dropdown.capture.clone()],
            }),
            Err(()),
            "a known capture needs fresh host facts in this observation"
        );
    }
}
