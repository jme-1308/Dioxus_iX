// Category flyout cleanup through the rendered Menu transports.
use super::*;
use crate::dropdown_inventory::{HostFacts, HostPartKind, ItemKind};
use crate::overlay_runtime::{Reason, RuntimeHandle};

fn cue_runtime(h: &mut MenuHarness, category: &CategoryNavigation) -> RuntimeHandle {
    let node = category.0.overlay.borrow().as_ref().unwrap().clone();
    let runtime = node.runtime().unwrap();
    let owner = runtime.owner_id();
    let transport = h
        .attributes
        .iter()
        .find(|(_, attributes)| attributes.get("data-ix-overlay-transport") == Some(&owner))
        .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
        .unwrap();
    menu_dispatch(
        &h.dom,
        "input",
        transport,
        serde_json::json!({
            "owner": owner,
            "document": format!("menu-cue-test-{owner}"),
            "sequence": 1,
            "kind": "modality",
            "input_kind": "keydown",
            "key": "Tab",
            "nodes": [{"id": node.id(), "generation": node.generation(), "connected": true}]
        })
        .to_string(),
    );
    h.settle();
    assert!(runtime.keyboard_mode());
    runtime
}

fn mark_rendered_child(h: &MenuHarness, runtime: &RuntimeHandle, label: &str) -> HostFacts {
    let id = &h.attributes[&h.target(label).raw()]["id"];
    let html = h.html();
    let tag = item_opening_tag(&html, id);
    let key = tag
        .split("data-ix-item-key=\"")
        .nth(1)
        .map(|value| value.split('"').next().unwrap().to_string())
        // Ordinary MenuHarness children have native controls rather than a
        // generic Dropdown host. Register their descriptive cue explicitly.
        .unwrap_or_else(|| format!("menu-cue-child-{id}"));
    let host: HostFacts = serde_json::from_value(serde_json::json!({
        "item": format!("captured-{id}"),
        "kind": "menu",
        "target": id,
        "capture": format!("captured-{id}"),
        "registration_key": key,
        "source_focusable": true
    }))
    .unwrap();
    runtime.register_item_cue(host.registration_key.clone(), ItemKind::Menu);
    h.dom.in_scope(ScopeId::ROOT, || {
        runtime.register_item_mount(
            &host.registration_key,
            &host.target,
            Rc::new(MountedData::new(MenuTestMount::default())),
        );
        runtime.mark_item_visual(&host);
    });
    assert!(runtime.item_visual_matches(&host));
    assert_eq!(
        runtime.item_visual_key(),
        Some(host.registration_key.clone())
    );
    host
}

#[test]
fn logical_disconnect_clears_child_cue_and_preserves_completed_focus_and_inventories() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let runtime = cue_runtime(&mut h, &category);
    let direct = h.facts(&["Alpha", "Category", "Omega", "Footer"]);
    h.observe("ready", "", None, false, direct.clone());
    let children = h.category_facts(&["Child", "Disabled child", "Two"]);
    h.category_event(
        "category-key",
        "Enter",
        "Category",
        None,
        1,
        children.clone(),
    );
    h.category_event(
        "category-key",
        "ArrowDown",
        "Category",
        None,
        2,
        children.clone(),
    );
    assert!(category.is_open());
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Child")));
    assert!(runtime.pending_item_focus_count() == 0);
    let focused = h.focuses.borrow().clone();
    let tabs = h.tabs();
    let cue = mark_rendered_child(&h, &runtime, "Child");

    assert_eq!(
        h.raw(serde_json::json!({"owner": category.0.owner,
            "kind": "category-disconnect", "category": category.0.id,
            "sequence": 3, "connected": false, "category_items": [], "items": []})),
        (false, true)
    );
    assert!(!category.is_open());
    assert!(!runtime.item_visual_matches(&cue));
    assert_eq!(runtime.item_visual_key(), None);
    assert_eq!(runtime.item_visual_target(), None);
    assert_eq!(h.context.model.borrow().observed, direct);
    assert_eq!(category.0.model.borrow().observed, children);
    assert_eq!(h.tabs(), tabs);
    assert!(runtime.pending_item_focus_count() == 0);
    assert_eq!(
        *h.focuses.borrow(),
        focused,
        "closing the flyout cannot refocus its hidden children"
    );
    assert!(category.0.model.borrow().alive);
    assert!(
        category
            .0
            .overlay
            .borrow()
            .as_ref()
            .unwrap()
            .is_registered()
    );
}

#[test]
fn closed_disconnect_and_inline_panel_close_preserve_the_current_cue() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let other = h.category("Second category");
    let runtime = cue_runtime(&mut h, &category);
    h.click("Second category");
    assert!(!category.is_open() && other.is_open());
    let other_cue = mark_rendered_child(&h, &runtime, "Solo");
    h.raw(serde_json::json!({"owner": category.0.owner,
        "kind": "category-disconnect", "category": category.0.id,
        "sequence": 1, "connected": false, "category_items": [], "items": []}));
    assert!(other.is_open());
    assert!(runtime.item_visual_matches(&other_cue));
    assert_eq!(runtime.item_visual_key(), Some(other_cue.registration_key));

    let mut inline = MenuHarness::new();
    let inline_category = inline.category("Category");
    let inline_runtime = cue_runtime(&mut inline, &inline_category);
    let inline_children = inline.category_facts(&["Child", "Disabled child", "Two"]);
    inline.category_event("category-order", "", "Category", None, 0, inline_children);
    assert!(inline_category.is_expanded());
    inline.click("Category");
    assert!(inline_category.is_open());
    assert_eq!(*inline_category.0.flyout.peek(), None);
    let inline_cue = mark_rendered_child(&inline, &inline_runtime, "Child");
    inline.click("Category");
    assert!(!inline_category.is_open());
    assert!(inline_runtime.item_visual_matches(&inline_cue));
    assert_eq!(
        inline_runtime.item_visual_key(),
        Some(inline_cue.registration_key)
    );
}

#[test]
fn logical_same_menu_ownership_loss_clears_child_cue_without_stealing_focus() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let other = h.category("Second category");
    let runtime = cue_runtime(&mut h, &category);
    let direct = h.facts(&["Alpha", "Category", "Omega", "Footer"]);
    h.observe("ready", "", None, false, direct.clone());
    let children = h.category_facts(&["Child", "Disabled child", "Two"]);
    h.category_event(
        "category-key",
        "Enter",
        "Category",
        None,
        1,
        children.clone(),
    );
    h.category_event(
        "category-key",
        "ArrowDown",
        "Category",
        None,
        2,
        children.clone(),
    );
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Child")));
    assert!(runtime.pending_item_focus_count() == 0);
    let focused = h.focuses.borrow().clone();
    let cue = mark_rendered_child(&h, &runtime, "Child");
    let tabs = h.tabs();
    let other_node = other.0.overlay.borrow().as_ref().unwrap().clone();

    // Direct controller presentation changes the shared flyout owner without
    // the parent-click broadcast first dismissing the previous owner. The
    // reactive ownership-loss handler must perform its own visual cleanup.
    h.dom.in_scope(ScopeId::ROOT, || {
        other_node.request(true, Reason::Trigger, false);
    });
    h.settle();

    assert!(!category.is_open() && other.is_open());
    assert_eq!(*category.0.flyout.peek(), Some(other.0.id));
    assert!(!runtime.item_visual_matches(&cue));
    assert_eq!(runtime.item_visual_key(), None);
    assert_eq!(runtime.item_visual_target(), None);
    assert_eq!(h.context.model.borrow().observed, direct);
    assert_eq!(category.0.model.borrow().observed, children);
    assert_eq!(h.tabs(), tabs);
    assert!(runtime.pending_item_focus_count() == 0);
    assert_eq!(
        *h.focuses.borrow(),
        focused,
        "closing the flyout cannot refocus its hidden children"
    );
    assert!(category.0.model.borrow().alive);
    assert!(
        category
            .0
            .overlay
            .borrow()
            .as_ref()
            .unwrap()
            .is_registered()
    );
}

#[test]
fn logical_return_clears_old_child_cue_and_keeps_the_parent_cue_after_rerender() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let runtime = cue_runtime(&mut h, &category);
    let children = h.category_facts(&["Child", "Disabled child", "Two"]);
    h.category_event(
        "category-key",
        "Enter",
        "Category",
        None,
        1,
        children.clone(),
    );
    let before_return = mark_rendered_child(&h, &runtime, "Child");
    assert_eq!(
        h.category_event(
            "category-key",
            "Enter",
            "Category",
            Some("Child"),
            2,
            children
        ),
        (true, false)
    );
    assert!(!category.is_open());
    assert!(!runtime.item_visual_matches(&before_return));
    assert_eq!(h.focuses.borrow().last().copied(), Some(category.0.id));
    menu_dispatch(&h.dom, "focus", h.target("Category"), ());
    h.settle();
    let html = h.html();
    let outer = html
        .split('>')
        .find(|tag| tag.contains(&format!("data-menu-category-root=\"{}\"", category.0.id)))
        .unwrap();
    let key = outer
        .split("data-ix-item-key=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .to_string();
    assert!(runtime.part_active(&key, HostPartKind::CategoryParent));
    assert_eq!(
        runtime.item_visual_key(),
        None,
        "Category outer has no active descendant cue"
    );
    let focused = h.focuses.borrow().clone();
    h.update();
    assert!(runtime.part_active(&key, HostPartKind::CategoryParent));
    assert_eq!(
        *h.focuses.borrow(),
        focused,
        "rerender cannot refocus a closed child"
    );
}
