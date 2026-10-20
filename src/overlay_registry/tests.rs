use super::*;
fn fact(id: &str, generation: u64, ancestors: &[&str], ready: bool) -> NativeFact {
    NativeFact {
        id: id.into(),
        generation,
        connected: true,
        trigger_ancestors: ancestors.iter().map(|id| (*id).into()).collect(),
        assignment_ancestors: None,
        discovery_ready: ready,
    }
}
fn dropdowns(ids: &[&str]) -> Registry {
    let mut reg = Registry::new();
    for id in ids {
        reg.register((*id).into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
    }
    reg
}

#[test]
fn current_children_admit_after_parent_open_and_follow_rebind_and_detach() {
    let mut reg = dropdowns(&["a", "b", "child", "peer"]);
    reg.observe(&[
        fact("a", 1, &[], true),
        fact("b", 1, &[], true),
        fact("peer", 1, &["a"], true),
    ]);
    reg.set_open("a", true);
    reg.observe(&[fact("child", 1, &["a"], true)]);
    assert_eq!(reg.cached_children("a"), ["child", "peer"]);
    assert_eq!(reg.cached_parent("child").as_deref(), Some("a"));
    reg.observe(&[fact("child", 1, &["b"], true)]);
    assert_eq!(reg.cached_children("a"), ["peer"]);
    assert_eq!(reg.cached_children("b"), ["child"]);
    reg.observe(&[fact("child", 1, &[], true)]);
    assert!(reg.cached_children("b").is_empty());
    assert_eq!(reg.cached_root("child").as_deref(), Some("child"));
}

#[test]
fn current_nearest_parent_readiness_and_unknown_owner_need_no_child_replay() {
    let mut reg = dropdowns(&["outer", "child"]);
    reg.observe(&[
        fact("outer", 1, &[], true),
        fact("child", 1, &["late", "outer"], true),
    ]);
    assert_eq!(reg.cached_parent("child"), None);
    reg.register("late".into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
    reg.observe(&[fact("late", 1, &["outer"], false)]);
    assert_eq!(reg.cached_parent("child"), None);
    reg.observe(&[fact("late", 1, &["outer"], true)]);
    assert_eq!(
        reg.cached_path_including("child"),
        ["child", "late", "outer"]
    );
    reg.observe(&[fact("child", 1, &["late", "outer"], false)]);
    assert!(reg.cached_children("late").is_empty());
    reg.observe(&[fact("child", 1, &["late", "outer"], true)]);
    assert_eq!(reg.cached_children("late"), ["child"]);
    reg.observe(&[fact("late", 1, &["outer"], false)]);
    assert_eq!(reg.cached_parent("child"), None);
}

#[test]
fn generation_disconnect_and_registration_changes_reject_stale_facts() {
    let mut reg = dropdowns(&["parent", "child"]);
    reg.observe(&[
        fact("parent", 1, &[], true),
        fact("child", 1, &["parent"], true),
    ]);
    reg.register(
        "child".into(),
        2,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
    );
    reg.unregister("child", 1);
    reg.observe(&[fact("child", 1, &["parent"], true)]);
    assert!(reg.cached_children("parent").is_empty());
    reg.observe(&[fact("child", 2, &["parent"], true)]);
    assert_eq!(reg.cached_children("parent"), ["child"]);
    let mut disconnected = fact("child", 2, &["parent"], true);
    disconnected.connected = false;
    reg.observe(&[disconnected]);
    assert!(reg.cached_children("parent").is_empty());
    reg.observe(&[fact("child", 2, &["parent"], true)]);
    reg.unregister("parent", 1);
    assert_eq!(reg.cached_parent("child"), None);
    reg.register(
        "parent".into(),
        2,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
    );
    reg.observe(&[fact("parent", 1, &[], true)]);
    assert_eq!(reg.cached_parent("child"), None);
    reg.observe(&[fact("parent", 2, &[], true)]);
    assert_eq!(reg.cached_parent("child"), None);
    reg.observe(&[fact("child", 2, &["parent"], true)]);
    assert_eq!(reg.cached_parent("child").as_deref(), Some("parent"));
}

#[test]
fn header_layout_moves_update_logical_parent_and_cross_kind_focus_stays_distinct() {
    let mut reg = dropdowns(&["child", "sub"]);
    reg.register("header".into(), 1, OverlayKind::Header, CloseBehavior::Both);
    reg.register(
        "popover".into(),
        1,
        OverlayKind::Popover,
        CloseBehavior::Both,
    );
    reg.observe(&[
        fact("header", 1, &[], true),
        fact("popover", 1, &["header"], true),
        fact("child", 1, &[], true),
        fact("sub", 1, &["child"], true),
    ]);
    for id in ["header", "popover", "child", "sub"] {
        reg.set_open(id, true);
    }
    reg.observe(&[
        fact("child", 1, &["popover", "header"], true),
        fact("sub", 1, &["child", "popover", "header"], true),
    ]);
    assert_eq!(reg.cached_root("sub").as_deref(), Some("header"));
    assert_eq!(reg.cached_parent("child").as_deref(), Some("header"));
    assert_eq!(reg.focus_parent("child").as_deref(), Some("popover"));
    assert_eq!(reg.live_ancestors("sub"), ["child", "popover", "header"]);
    assert!(reg.path_in_hierarchy("header", &["sub".into()], &[]));
    reg.observe(&[fact("child", 1, &[], true)]);
    assert_eq!(reg.cached_root("sub").as_deref(), Some("child"));
}

#[test]
fn explicit_empty_logical_ancestry_keeps_anchor_parentless_but_accepts_its_children() {
    let mut reg = dropdowns(&["root", "anchor", "child"]);
    let mut anchor = fact("anchor", 1, &["root"], true);
    anchor.assignment_ancestors = Some(Vec::new());
    reg.observe(&[
        fact("root", 1, &[], true),
        anchor.clone(),
        fact("child", 1, &["anchor", "root"], true),
    ]);
    for id in ["root", "anchor", "child"] {
        reg.set_open(id, true);
    }
    assert!(reg.cached_children("root").is_empty());
    assert_eq!(reg.cached_children("anchor"), ["child"]);
    assert_eq!(reg.cached_root("child").as_deref(), Some("anchor"));
    assert_eq!(reg.live_parent("anchor").as_deref(), Some("root"));
    reg.register(
        "anchor".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Inside,
    );
    assert_eq!(reg.cached_parent("anchor"), None);
    reg.observe(&[fact("anchor", 1, &["root"], true)]);
    assert_eq!(reg.cached_parent("anchor").as_deref(), Some("root"));
}

#[test]
fn current_graph_deduplicates_ancestry_and_cuts_cycles_at_first_registration() {
    let mut reg = dropdowns(&["a", "b", "c"]);
    reg.observe(&[
        fact("a", 1, &["a", "b", "b"], true),
        fact("b", 1, &["c"], true),
        fact("c", 1, &["a"], true),
    ]);
    assert_eq!(reg.nodes["a"].trigger_ancestors, ["b"]);
    assert_eq!(reg.cached_parent("a"), None);
    assert_eq!(reg.cached_root("b").as_deref(), Some("a"));
    assert_eq!(reg.cached_children("a"), ["c"]);
    assert_eq!(reg.cached_path_including("b"), ["b", "c", "a"]);
    reg.observe(&[fact("a", 1, &[], true)]);
    assert_eq!(reg.cached_path_including("b"), ["b", "c", "a"]);
}

#[test]
fn current_popover_children_do_not_depend_on_open_presence_or_policy() {
    let mut reg = Registry::new();
    for id in ["parent", "child"] {
        reg.register(id.into(), 1, OverlayKind::Popover, CloseBehavior::Both);
    }
    reg.observe(&[
        fact("parent", 1, &[], true),
        fact("child", 1, &["parent"], true),
    ]);
    assert_eq!(reg.cached_children("parent"), ["child"]);
    assert!(reg.topmost().is_none());
    reg.set_presence("parent", true);
    reg.set_open("child", true);
    reg.set_presence("parent", false);
    reg.set_close_behavior("parent", CloseBehavior::None);
    assert!(reg.is_open("child"));
    assert_eq!(reg.cached_parent("child").as_deref(), Some("parent"));
    reg.observe(&[fact("child", 1, &[], true)]);
    assert!(reg.cached_children("parent").is_empty());
}

#[test]
fn current_paths_exclude_siblings_and_descendants() {
    let mut reg = dropdowns(&["outer", "parent", "sibling", "child"]);
    reg.observe(&[
        fact("outer", 1, &[], true),
        fact("parent", 1, &["outer"], true),
        fact("sibling", 1, &["outer"], true),
        fact("child", 1, &["parent", "outer"], true),
    ]);
    assert_eq!(reg.cached_path_including("parent"), ["parent", "outer"]);
    assert_eq!(
        reg.cached_path_including("child"),
        ["child", "parent", "outer"]
    );
    assert!(reg.cached_path_including("missing").is_empty());
}

#[test]
fn dropdown_exclusions_follow_current_logical_and_live_cross_kind_ancestors() {
    let mut reg = dropdowns(&["old", "active", "sibling", "descendant", "live"]);
    reg.register(
        "middle".into(),
        1,
        OverlayKind::Popover,
        CloseBehavior::Both,
    );
    reg.observe(&[
        fact("old", 1, &[], true),
        fact("active", 1, &["old"], true),
        fact("sibling", 1, &["old"], true),
        fact("descendant", 1, &["active", "old"], true),
        fact("live", 1, &[], true),
        fact("middle", 1, &["live"], true),
    ]);
    for id in ["old", "active", "sibling", "descendant", "live", "middle"] {
        reg.set_open(id, true);
    }
    reg.observe(&[fact("active", 1, &["middle", "live"], true)]);
    assert_eq!(reg.cached_parent("active").as_deref(), Some("live"));
    assert_eq!(reg.live_ancestors("active"), ["middle", "live"]);
    assert_eq!(
        reg.dropdown_dismiss_exclusions("active"),
        ["active", "live"]
    );
}

#[test]
fn dropdown_trigger_iteration_tracks_registration_lifetime_before_policy() {
    let mut reg = Registry::new();
    reg.register(
        "protected".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Inside,
    );
    reg.register(
        "foreign".into(),
        1,
        OverlayKind::Popover,
        CloseBehavior::Both,
    );
    reg.register("later".into(), 1, OverlayKind::Header, CloseBehavior::Both);
    let mut visited = BTreeSet::new();
    let first = reg.next_dropdown_dismiss_other(&visited).unwrap();
    assert_eq!((first.0.as_str(), first.1), ("protected", 1));
    visited.insert(first.2);
    reg.set_close_behavior("protected", CloseBehavior::Both);
    reg.register(
        "protected".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
    );
    let later = reg.next_dropdown_dismiss_other(&visited).unwrap();
    assert_eq!(later.0, "later");
    visited.insert(later.2);
    assert!(reg.next_dropdown_dismiss_other(&visited).is_none());
    reg.unregister("protected", 1);
    reg.register(
        "protected".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
    );
    let reconnected = reg.next_dropdown_dismiss_other(&visited).unwrap();
    assert_eq!((reconnected.0.as_str(), reconnected.1), ("protected", 1));
    assert_ne!(first.2, reconnected.2);
    visited.insert(reconnected.2);
    reg.register("later".into(), 2, OverlayKind::Header, CloseBehavior::Both);
    let replaced = reg.next_dropdown_dismiss_other(&visited).unwrap();
    assert_eq!((replaced.0.as_str(), replaced.1), ("later", 2));
    assert_ne!(later.2, replaced.2);
}

#[test]
fn topmost_follows_presentation_order_independent_of_click_policy() {
    let mut reg = Registry::new();
    reg.register("header".into(), 1, OverlayKind::Header, CloseBehavior::Both);
    reg.register(
        "child".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
    );
    reg.register(
        "sub".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Outside,
    );
    reg.register(
        "inside_only".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Inside,
    );
    reg.observe(&[
        fact("header", 1, &[], true),
        fact("child", 1, &["header"], true),
        fact("sub", 1, &["child"], true),
        fact("inside_only", 1, &[], true),
    ]);
    reg.set_open("header", true);
    reg.set_open("child", true);
    reg.set_open("sub", true);
    reg.set_open("inside_only", true);
    reg.set_open("header", true);
    assert_eq!(
        reg.topmost().as_deref(),
        Some("inside_only"),
        "topmost ignores click policy and unchanged opens"
    );
}

#[test]
fn disconnected_visibility_is_not_topmost_and_policy_refresh_preserves_current_facts() {
    let mut reg = dropdowns(&["parent", "child"]);
    reg.observe(&[
        fact("parent", 1, &[], true),
        fact("child", 1, &["parent"], true),
    ]);
    reg.set_open("parent", true);
    reg.set_open("child", true);
    reg.register(
        "child".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Outside,
    );
    assert_eq!(reg.cached_parent("child").as_deref(), Some("parent"));
    assert_eq!(reg.close_behavior("child"), Some(CloseBehavior::Outside));
    let mut disconnected = fact("child", 1, &[], false);
    disconnected.connected = false;
    reg.observe(&[disconnected]);
    assert_eq!(reg.topmost().as_deref(), Some("parent"));
    assert!(!reg.is_connected("child"));
}
