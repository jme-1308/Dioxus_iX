//! Literal broad descendant selection for Dropdown opening focus.
//!
//! Native plumbing reports actual objects, own attributes and source topology.
//! Rust applies the pinned selector, slot ordering and one-shadow projection.

use crate::dropdown_inventory::ItemKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DescendantNode {
    pub(crate) capture: String,
    pub(crate) tag: String,
    pub(crate) is_html: bool,
    #[serde(default)]
    pub(crate) source_kind: Option<ItemKind>,
    #[serde(default)]
    pub(crate) tabindex: Option<String>,
    #[serde(default)]
    pub(crate) href: bool,
    #[serde(default)]
    pub(crate) contenteditable: bool,
    #[serde(default)]
    pub(crate) hidden: bool,
    #[serde(default)]
    pub(crate) disabled: bool,
    #[serde(default)]
    pub(crate) checked: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DescendantAssignment {
    pub(crate) root: String,
    pub(crate) descendants: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DescendantSlot {
    pub(crate) assignments: Vec<DescendantAssignment>,
    pub(crate) following: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DescendantShadow {
    pub(crate) host: String,
    pub(crate) descendants: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DescendantFacts {
    pub(crate) nodes: Vec<DescendantNode>,
    pub(crate) light: Vec<String>,
    pub(crate) slots: Vec<DescendantSlot>,
    pub(crate) shadows: Vec<DescendantShadow>,
    pub(crate) fallback: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DescendantChoice {
    pub(crate) candidate: Option<String>,
    pub(crate) target: String,
    pub(crate) is_html: bool,
}

impl DescendantNode {
    fn matches(&self, checked: bool) -> bool {
        if self.hidden
            || self.disabled
            || self
                .tabindex
                .as_deref()
                .is_some_and(|value| value.starts_with('-'))
            || checked && !self.checked
        {
            return false;
        }
        if self.tabindex.is_some() || self.contenteditable {
            return true;
        }
        // Flattened component controls belong to source shadow topology. A
        // marked source Menu/Category host is not a native button selector.
        let tag = match self.source_kind {
            Some(ItemKind::Dropdown) => "ix-dropdown-item",
            Some(ItemKind::Select) => "ix-select-item",
            Some(ItemKind::Menu) => "ix-menu-item",
            Some(ItemKind::Category) => "ix-menu-category",
            Some(ItemKind::NativeOptIn) | None => self.tag.as_str(),
        };
        match tag {
            "ix-dropdown-item" | "ix-select-item" | "button" | "audio" | "input" | "select"
            | "textarea" | "iframe" | "object" | "embed" | "video" => true,
            "a" | "area" => self.href,
            _ => false,
        }
    }
}

fn query_order<'a>(
    facts: &'a DescendantFacts,
    nodes: &BTreeMap<&str, &'a DescendantNode>,
    checked: bool,
) -> Option<Vec<&'a str>> {
    let matching = |order: &'a [String]| -> Option<Vec<&'a str>> {
        let mut result = Vec::new();
        for capture in order {
            let node = nodes.get(capture.as_str())?;
            if node.matches(checked) {
                result.push(capture.as_str());
            }
        }
        Some(result)
    };
    let mut result = matching(&facts.light)?;
    for slot in facts.slots.iter().rev() {
        let mut assigned = Vec::new();
        for assignment in &slot.assignments {
            let root = nodes.get(assignment.root.as_str())?;
            if root.matches(checked) {
                assigned.push(assignment.root.as_str());
            } else {
                assigned.extend(matching(&assignment.descendants)?);
            }
        }
        let at = result
            .iter()
            .position(|capture| {
                slot.following
                    .iter()
                    .any(|following| following.as_str() == *capture)
            })
            .unwrap_or(result.len());
        result.splice(at..at, assigned);
    }
    Some(result)
}

/// Select once; native failure/no-op never retries another candidate/fallback.
/// `is_html` is execution metadata, not a selector exclusion.
pub(crate) fn choose(
    facts: &DescendantFacts,
    last: bool,
    focus_checked: bool,
) -> Option<DescendantChoice> {
    let mut nodes = BTreeMap::new();
    for node in &facts.nodes {
        if node.capture.is_empty() {
            return None;
        }
        nodes.entry(node.capture.as_str()).or_insert(node);
    }
    let checked = if focus_checked && !last {
        query_order(facts, &nodes, true)?
    } else {
        Vec::new()
    };
    let order = if checked.is_empty() {
        query_order(facts, &nodes, false)?
    } else {
        checked
    };
    let candidate = if last { order.last() } else { order.first() }.copied();
    let Some(candidate) = candidate else {
        // Source passes the fallback/ref directly to tryFocusElement. Its own
        // shadow root is not searched when the outer query has no candidate.
        let fallback = nodes.get(facts.fallback.as_str())?;
        return Some(DescendantChoice {
            candidate: None,
            target: fallback.capture.clone(),
            is_html: fallback.is_html,
        });
    };
    let mut target = *nodes.get(candidate)?;
    if let Some(shadow) = facts.shadows.iter().find(|shadow| shadow.host == candidate) {
        for capture in &shadow.descendants {
            let inner = nodes.get(capture.as_str())?;
            if inner.matches(false) {
                target = inner;
                break;
            }
        }
    }
    Some(DescendantChoice {
        candidate: Some(candidate.to_string()),
        target: target.capture.clone(),
        is_html: target.is_html,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(capture: &str, tag: &str) -> DescendantNode {
        DescendantNode {
            capture: capture.into(),
            tag: tag.into(),
            is_html: true,
            source_kind: None,
            tabindex: None,
            href: false,
            contenteditable: false,
            hidden: false,
            disabled: false,
            checked: false,
        }
    }
    fn facts(nodes: Vec<DescendantNode>, light: &[&str]) -> DescendantFacts {
        let mut all = vec![node("fallback", "div")];
        all.extend(nodes);
        DescendantFacts {
            nodes: all,
            light: light.iter().map(|capture| (*capture).into()).collect(),
            slots: Vec::new(),
            shadows: Vec::new(),
            fallback: "fallback".into(),
        }
    }
    fn target(facts: &DescendantFacts, last: bool, checked: bool) -> DescendantChoice {
        choose(facts, last, checked).unwrap()
    }

    #[test]
    fn broad_selector_uses_literal_own_attributes_and_all_native_tag_branches() {
        for tag in [
            "ix-dropdown-item",
            "ix-select-item",
            "button",
            "audio",
            "input",
            "select",
            "textarea",
            "iframe",
            "object",
            "embed",
            "video",
        ] {
            let raw = facts(vec![node("first", tag)], &["first"]);
            assert_eq!(
                target(&raw, false, false).candidate.as_deref(),
                Some("first"),
                "{tag}"
            );
        }
        for tag in ["a", "area"] {
            let mut candidate = node("candidate", tag);
            assert_eq!(
                target(
                    &facts(vec![candidate.clone()], &["candidate"]),
                    false,
                    false
                )
                .candidate,
                None
            );
            candidate.href = true;
            assert_eq!(
                target(&facts(vec![candidate], &["candidate"]), false, false).target,
                "candidate"
            );
        }
        for value in ["", "0", "12", "invalid", " -1", "+0"] {
            let mut candidate = node("literal", "span");
            candidate.tabindex = Some(value.into());
            assert_eq!(
                target(&facts(vec![candidate], &["literal"]), false, false).target,
                "literal",
                "{value:?}"
            );
        }
        for value in ["-1", "-0", "-invalid", "--"] {
            let mut candidate = node("negative", "button");
            candidate.tabindex = Some(value.into());
            assert_eq!(
                target(&facts(vec![candidate], &["negative"]), false, false).candidate,
                None
            );
        }
        let mut editable = node("editable", "span");
        editable.contenteditable = true;
        assert_eq!(
            target(&facts(vec![editable.clone()], &["editable"]), false, false).target,
            "editable"
        );
        editable.hidden = true;
        assert_eq!(
            target(&facts(vec![editable.clone()], &["editable"]), false, false).candidate,
            None
        );
        editable.hidden = false;
        editable.disabled = true;
        assert_eq!(
            target(&facts(vec![editable], &["editable"]), false, false).candidate,
            None
        );
        // No input type, controls, inert, ancestor or computed style field is
        // consumed: those facts cannot replace the source literal selector.
    }

    #[test]
    fn document_order_is_not_tab_order_and_source_host_kinds_keep_shadow_controls_separate() {
        let mut a = node("a", "button");
        a.tabindex = Some("12".into());
        let mut b = node("b", "button");
        b.tabindex = Some("1".into());
        let raw = facts(vec![a, b], &["a", "b"]);
        assert_eq!(target(&raw, false, false).target, "a");
        assert_eq!(target(&raw, true, false).target, "b");
        for kind in [
            ItemKind::Dropdown,
            ItemKind::Select,
            ItemKind::Menu,
            ItemKind::Category,
        ] {
            let mut host = node("host", "div");
            host.source_kind = Some(kind);
            let raw = facts(vec![host.clone()], &["host"]);
            assert_eq!(
                target(&raw, false, false).candidate.is_some(),
                matches!(kind, ItemKind::Dropdown | ItemKind::Select)
            );
            host.tabindex = Some("0".into());
            assert_eq!(
                target(&facts(vec![host], &["host"]), false, false).target,
                "host"
            );
        }
    }

    #[test]
    fn first_checked_is_broad_attribute_priority_and_last_ignores_it() {
        let a = node("a", "button");
        let mut b = node("b", "input");
        b.checked = true;
        let c = node("c", "ix-select-item");
        let raw = facts(vec![a, b, c], &["a", "b", "c"]);
        assert_eq!(target(&raw, false, false).target, "a");
        assert_eq!(target(&raw, false, true).target, "b");
        assert_eq!(target(&raw, true, true).target, "c");
        let none = facts(vec![node("a", "button")], &["a"]);
        assert_eq!(target(&none, false, true).target, "a");
    }

    #[test]
    fn checked_slots_run_their_own_root_match_and_descendant_expansion() {
        let assigned = node("assigned", "button");
        let mut child = node("child", "input");
        child.checked = true;
        let end = node("end", "button");
        let mut raw = facts(vec![assigned, child, end], &["end"]);
        raw.slots.push(DescendantSlot {
            assignments: vec![DescendantAssignment {
                root: "assigned".into(),
                descendants: vec!["child".into()],
            }],
            following: vec!["end".into()],
        });
        assert_eq!(target(&raw, false, false).target, "assigned");
        assert_eq!(target(&raw, false, true).target, "child");
        assert_eq!(target(&raw, true, true).target, "end");
    }

    #[test]
    fn slots_preserve_duplicate_objects_and_reverse_insertion_relations() {
        let mut raw = facts(
            vec![
                node("a", "button"),
                node("z", "button"),
                node("early", "button"),
                node("late", "button"),
                node("wrapper", "div"),
            ],
            &["a", "z"],
        );
        raw.slots = vec![
            DescendantSlot {
                assignments: vec![DescendantAssignment {
                    root: "wrapper".into(),
                    descendants: vec!["early".into(), "a".into()],
                }],
                following: vec!["late".into(), "z".into()],
            },
            DescendantSlot {
                assignments: vec![DescendantAssignment {
                    root: "late".into(),
                    descendants: vec!["early".into()],
                }],
                following: vec!["z".into()],
            },
        ];
        let nodes = raw
            .nodes
            .iter()
            .map(|node| (node.capture.as_str(), node))
            .collect();
        assert_eq!(
            query_order(&raw, &nodes, false).unwrap(),
            ["a", "early", "a", "late", "z"]
        );
        raw.slots[0].following.clear();
        let nodes = raw
            .nodes
            .iter()
            .map(|node| (node.capture.as_str(), node))
            .collect();
        assert_eq!(
            query_order(&raw, &nodes, false).unwrap(),
            ["a", "late", "z", "early", "a"]
        );
        assert_eq!(target(&raw, true, false).target, "a");
    }

    #[test]
    fn shadow_projection_is_first_unqualified_match_without_slot_expansion_or_recursion() {
        let mut outer = node("outer", "ix-select-item");
        outer.checked = true;
        let hidden = {
            let mut node = node("hidden", "button");
            node.hidden = true;
            node
        };
        let nested = node("nested", "ix-dropdown-item");
        let later = node("later", "button");
        let deep = node("deep", "input");
        let assigned = node("assigned", "button");
        let mut raw = facts(
            vec![outer, hidden, nested, later, deep, assigned],
            &["outer"],
        );
        raw.shadows = vec![
            DescendantShadow {
                host: "outer".into(),
                descendants: vec!["hidden".into(), "nested".into(), "later".into()],
            },
            DescendantShadow {
                host: "nested".into(),
                descendants: vec!["deep".into()],
            },
        ];
        let choice = target(&raw, false, true);
        assert_eq!(choice.candidate.as_deref(), Some("outer"));
        assert_eq!(
            choice.target, "nested",
            "no checked condition or second shadow projection"
        );
        raw.shadows[0].descendants = vec!["hidden".into()];
        assert_eq!(
            target(&raw, false, false).target,
            "outer",
            "no qualifying own shadow match retains host"
        );
        // An assigned object not in this shadowRoot's direct query order is
        // absent from its querySelector result, regardless of its eligibility.
        assert_ne!(target(&raw, false, false).target, "assigned");
    }

    #[test]
    fn empty_fallback_is_direct_and_selected_non_html_never_falls_through() {
        let mut raw = facts(vec![node("fallback-shadow", "button")], &[]);
        raw.shadows.push(DescendantShadow {
            host: "fallback".into(),
            descendants: vec!["fallback-shadow".into()],
        });
        assert_eq!(
            target(&raw, false, true),
            DescendantChoice {
                candidate: None,
                target: "fallback".into(),
                is_html: true
            }
        );
        let mut svg = node("svg", "g");
        svg.tabindex = Some("0".into());
        svg.is_html = false;
        raw = facts(vec![svg, node("later", "button")], &["svg", "later"]);
        assert_eq!(
            target(&raw, false, false),
            DescendantChoice {
                candidate: Some("svg".into()),
                target: "svg".into(),
                is_html: false
            }
        );
        assert_eq!(target(&raw, true, false).target, "later");
    }
}
