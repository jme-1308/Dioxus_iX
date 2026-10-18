//! Read-only authorization for Rust-current native focus-cue projection.
//!
//! Registration and raw capture binding happen outside projection queries.
//! Reconciliation asks current truth repeatedly; it never consumes a focus job
//! or replays a cached JavaScript winner.

use crate::dropdown_inventory::{HostFacts, HostPartKind, ItemKind};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct CueOffer {
    pub(crate) key: String,
    pub(crate) capture: String,
    #[serde(default)]
    pub(crate) part: Option<String>,
    pub(crate) active: bool,
}

struct HostBinding {
    kind: ItemKind,
    facts: Option<HostFacts>,
}

#[derive(Default)]
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

    /// Pure and repeatable. `known` distinguishes an authoritative empty
    /// projection from pre-observation bootstrap. Owning registration lifetime
    /// is checked by the runtime; no open/revision/connected policy belongs here.
    pub(crate) fn permits(
        &self,
        offer: &CueOffer,
        current_host: Option<&HostFacts>,
        known: bool,
    ) -> bool {
        if !known || offer.key.is_empty() || offer.capture.is_empty() {
            return false;
        }
        let current = current_host
            .filter(|host| host.registration_key == offer.key && host.capture == offer.capture);
        let facts = match self.hosts.get(&offer.key) {
            Some(binding) => {
                let Some(facts) = &binding.facts else {
                    return false;
                };
                if facts.capture != offer.capture {
                    return false;
                }
                facts
            }
            // An independently admitted operation can still hold its original
            // actual host after that item's scope detached. Never find a new
            // object by public ID or weaken a current registered binding.
            None => match current {
                Some(host) => host,
                None => return false,
            },
        };
        // Observation binds current object/part identities. Only a delivered
        // focus attempt changes focus policy; a later class/property update
        // must not erase that attempt or transfer it to a replacement part.
        let attempted = current.filter(|host| host.kind == facts.kind);
        let desired = match offer.part.as_deref() {
            None => match facts.kind {
                ItemKind::Dropdown | ItemKind::Select | ItemKind::Menu => {
                    attempted.is_some_and(|host| host.source_focusable)
                }
                ItemKind::Category | ItemKind::NativeOptIn => return false,
            },
            Some(capture) => {
                if capture.is_empty() || capture == facts.capture {
                    return false;
                }
                let Some(part) = facts.parts.iter().find(|part| part.capture == capture) else {
                    return false;
                };
                match (facts.kind, part.kind) {
                    // This is the outer Select's rendered cue, not an
                    // independent inner focus/selection decision.
                    (ItemKind::Select, HostPartKind::SelectOption) => {
                        attempted.is_some_and(|host| host.source_focusable)
                    }
                    (ItemKind::Category, HostPartKind::CategoryParent) => {
                        attempted.is_some_and(|host| {
                            host.parts.iter().any(|attempted_part| {
                                attempted_part.kind == HostPartKind::CategoryParent
                                    && attempted_part.capture == capture
                                    && attempted_part.source_focusable
                            })
                        })
                    }
                    _ => return false,
                }
            }
        };
        offer.active == desired
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dropdown_inventory::{HostPartFacts, ItemRef};

    fn host(key: &str, capture: &str, kind: ItemKind) -> HostFacts {
        HostFacts {
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
    fn offer(host: &HostFacts, part: Option<&str>, active: bool) -> CueOffer {
        CueOffer {
            key: host.registration_key.clone(),
            capture: host.capture.clone(),
            part: part.map(str::to_string),
            active,
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
        assert!(!registry.permits(&offer(&original, None, true), Some(&original), true));
        assert!(registry.observe(&original));
        assert!(registry.permits(&offer(&original, None, true), Some(&original), true));
        let copied = host("source-a", "object-b", ItemKind::Dropdown);
        assert_eq!(original.target, copied.target);
        assert!(!registry.observe(&copied));
        assert!(!registry.permits(&offer(&copied, None, true), Some(&copied), true));
        let foreign = host("other-runtime-key", "object-a", ItemKind::Dropdown);
        assert!(!registry.observe(&foreign));
        assert!(!registry.permits(&offer(&foreign, None, false), None, true));
        let mut wrong_kind = original.clone();
        wrong_kind.kind = ItemKind::Select;
        assert!(!registry.observe(&wrong_kind));
        assert!(!registry.register(original.registration_key.clone(), ItemKind::Select));
    }

    #[test]
    fn current_projection_queries_are_repeatable_and_clear_without_epochs() {
        let mut registry = CueHosts::default();
        let a = host("a", "object-a", ItemKind::Dropdown);
        let b = host("b", "object-b", ItemKind::Menu);
        bind(&mut registry, &a);
        bind(&mut registry, &b);
        for _ in 0..3 {
            assert!(registry.permits(&offer(&a, None, true), Some(&a), true));
            assert!(registry.permits(&offer(&b, None, false), Some(&a), true));
        }
        assert!(!registry.permits(&offer(&a, None, true), Some(&b), true));
        assert!(registry.permits(&offer(&a, None, false), Some(&b), true));
        assert!(registry.permits(&offer(&b, None, true), Some(&b), true));
        assert!(registry.permits(&offer(&a, None, false), None, true));
        assert!(registry.permits(&offer(&b, None, false), None, true));
        assert!(!registry.permits(&offer(&a, None, false), None, false));
        assert!(!registry.permits(&offer(&a, None, true), Some(&a), false));
        assert_eq!(registry.hosts.len(), 2);
    }

    #[test]
    fn typed_parts_keep_select_projection_and_category_parent_separate() {
        let mut registry = CueHosts::default();
        let mut select = host("select", "select-object", ItemKind::Select);
        select.parts.push(HostPartFacts {
            kind: HostPartKind::SelectOption,
            capture: "select-inner".into(),
            source_focusable: false,
        });
        let mut category = host("category", "category-object", ItemKind::Category);
        category.source_focusable = false;
        category.parts.push(HostPartFacts {
            kind: HostPartKind::CategoryParent,
            capture: "category-parent".into(),
            source_focusable: true,
        });
        bind(&mut registry, &select);
        bind(&mut registry, &category);
        assert!(registry.permits(&offer(&select, None, true), Some(&select), true));
        assert!(registry.permits(
            &offer(&select, Some("select-inner"), true),
            Some(&select),
            true
        ));
        assert!(registry.permits(&offer(&select, Some("select-inner"), false), None, true));
        let attempted_select = select.clone();
        select.source_focusable = false;
        assert!(registry.observe(&select));
        for part in [None, Some("select-inner")] {
            assert!(registry.permits(&offer(&select, part, true), Some(&attempted_select), true));
            assert!(!registry.permits(&offer(&select, part, false), Some(&attempted_select), true));
            assert!(registry.permits(&offer(&select, part, false), Some(&select), true));
            assert!(!registry.permits(&offer(&select, part, true), Some(&select), true));
        }
        select.source_focusable = true;
        assert!(registry.observe(&select));
        for active in [false, true] {
            assert!(!registry.permits(&offer(&category, None, active), Some(&category), true));
            assert!(!registry.permits(
                &offer(&select, Some("category-parent"), active),
                Some(&select),
                true
            ));
            assert!(!registry.permits(
                &offer(&category, Some("select-inner"), active),
                Some(&category),
                true
            ));
        }
        assert!(registry.permits(
            &offer(&category, Some("category-parent"), true),
            Some(&category),
            true
        ));
        assert!(registry.permits(
            &offer(&category, Some("category-parent"), false),
            Some(&select),
            true
        ));
        let attempted_category = category.clone();
        category.parts[0].source_focusable = false;
        assert!(registry.observe(&category));
        assert!(registry.permits(
            &offer(&category, Some("category-parent"), true),
            Some(&attempted_category),
            true
        ));
        assert!(!registry.permits(
            &offer(&category, Some("category-parent"), false),
            Some(&attempted_category),
            true
        ));
        assert!(!registry.permits(
            &offer(&category, Some("category-parent"), true),
            Some(&category),
            true
        ));
        assert!(registry.permits(
            &offer(&category, Some("category-parent"), false),
            Some(&category),
            true
        ));
        category.parts[0].capture = "replacement-parent".into();
        category.parts[0].source_focusable = true;
        assert!(registry.observe(&category));
        assert!(!registry.permits(
            &offer(&category, Some("category-parent"), false),
            Some(&category),
            true
        ));
        assert!(!registry.permits(
            &offer(&category, Some("replacement-parent"), true),
            Some(&attempted_category),
            true
        ));
        assert!(registry.permits(
            &offer(&category, Some("replacement-parent"), false),
            Some(&attempted_category),
            true
        ));
        assert!(registry.permits(
            &offer(&category, Some("replacement-parent"), true),
            Some(&category),
            true
        ));
        select.parts.push(HostPartFacts {
            kind: HostPartKind::CategoryParent,
            capture: "wrong-kind-part".into(),
            source_focusable: true,
        });
        assert!(registry.observe(&select));
        for active in [false, true] {
            assert!(!registry.permits(
                &offer(&select, Some("wrong-kind-part"), active),
                Some(&select),
                true
            ));
        }
    }

    #[test]
    fn only_new_attempt_updates_source_awareness_and_native_opt_in_has_no_fabricated_cue() {
        let mut registry = CueHosts::default();
        let mut menu = host("menu", "menu-object", ItemKind::Menu);
        let native = host("native", "native-object", ItemKind::NativeOptIn);
        bind(&mut registry, &menu);
        bind(&mut registry, &native);
        let attempted_menu = menu.clone();
        menu.source_focusable = false;
        assert!(registry.observe(&menu));
        assert!(registry.permits(&offer(&menu, None, true), Some(&attempted_menu), true));
        assert!(!registry.permits(&offer(&menu, None, false), Some(&attempted_menu), true));
        assert!(registry.permits(&offer(&menu, None, false), Some(&menu), true));
        assert!(!registry.permits(&offer(&menu, None, true), Some(&menu), true));
        // An unrelated positive observation likewise cannot activate an
        // originally non-focus-aware attempt until another attempt occurs.
        let non_focusable_attempt = menu.clone();
        menu.source_focusable = true;
        assert!(registry.observe(&menu));
        assert!(registry.permits(
            &offer(&menu, None, false),
            Some(&non_focusable_attempt),
            true
        ));
        assert!(!registry.permits(
            &offer(&menu, None, true),
            Some(&non_focusable_attempt),
            true
        ));
        assert!(registry.permits(&offer(&menu, None, true), Some(&menu), true));
        for active in [false, true] {
            assert!(!registry.permits(&offer(&native, None, active), Some(&native), true));
        }
    }

    #[test]
    fn detached_current_capture_survives_item_unregister_without_retargeting_replacement() {
        let mut registry = CueHosts::default();
        let mut old = host("mount-a", "old-object", ItemKind::Category);
        old.parts.push(HostPartFacts {
            kind: HostPartKind::CategoryParent,
            capture: "old-parent".into(),
            source_focusable: true,
        });
        bind(&mut registry, &old);
        registry.unregister(&old.registration_key);
        assert!(registry.hosts.is_empty());
        assert!(registry.permits(&offer(&old, Some("old-parent"), true), Some(&old), true));
        assert!(!registry.permits(&offer(&old, Some("old-parent"), true), None, true));
        let replacement = host("mount-b", "new-object", ItemKind::Category);
        bind(&mut registry, &replacement);
        assert_eq!(old.target, replacement.target);
        assert!(registry.permits(&offer(&old, Some("old-parent"), true), Some(&old), true));
        assert!(!registry.permits(
            &offer(&replacement, Some("old-parent"), true),
            Some(&old),
            true
        ));
        // Even explicit reuse of a key must not weaken the new live binding.
        let reused = host("mount-a", "reused-object", ItemKind::Category);
        bind(&mut registry, &reused);
        assert!(!registry.permits(&offer(&old, Some("old-parent"), true), Some(&old), true));
    }

    #[test]
    fn new_capture_namespace_requires_fresh_evidence_without_losing_live_registration() {
        let mut registry = CueHosts::default();
        let old = host("live-host", "old-document-object", ItemKind::Select);
        bind(&mut registry, &old);
        registry.reset_captures();
        assert!(!registry.permits(&offer(&old, None, true), Some(&old), true));
        let fresh = host("live-host", "new-document-object", ItemKind::Select);
        assert!(registry.observe(&fresh));
        assert!(registry.permits(&offer(&fresh, None, true), Some(&fresh), true));
        assert!(!registry.permits(&offer(&old, None, true), Some(&old), true));
        registry.unregister(&fresh.registration_key);
        assert!(registry.hosts.is_empty());
    }
}
