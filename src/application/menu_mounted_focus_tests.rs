// Rust-owned Menu and Category navigation executes registered mounted controls.
use super::*;

fn mounted_menu() -> MenuHarness {
    let mut h = MenuHarness::new();
    {
        let mut config = h.config.borrow_mut();
        config.root_keys = ["Alpha", "Disabled", "Link", "Omega"]
            .into_iter()
            .map(str::to_string)
            .collect();
        config.utilities = false;
    }
    h.update();
    h.focuses.borrow_mut().clear();
    h
}
fn mounted_category(h: &MenuHarness) -> CategoryNavigation {
    h.category("Category")
}

#[test]
fn mounted_menu_roving_reaches_mounted_controls_and_keeps_eligible_endpoints() {
    let _guard = crate::test_event_converter_guard();
    let mut h = mounted_menu();
    h.config.borrow_mut().hidden.insert("Link".into());
    h.update();
    for (key, origin, expected) in [
        ("ArrowDown", "Alpha", "Omega"),
        ("End", "Omega", "Footer"),
        ("Home", "Footer", "Alpha"),
        ("ArrowUp", "Alpha", "Footer"),
    ] {
        assert_eq!(h.root_key(key, Some(h.id(origin)), false), (true, false));
        assert_eq!(
            h.focuses.borrow().last().copied(),
            Some(h.id(expected)),
            "Rust-owned navigation must call the registered MountedData"
        );
    }
    assert_eq!(h.focuses.borrow().len(), 4);
    assert_eq!(h.config.borrow().activations, 0);
}

#[test]
fn mounted_menu_mounted_requests_reject_newer_intent_and_replaced_or_removed_controls() {
    let _guard = crate::test_event_converter_guard();
    let mut h = mounted_menu();
    h.dom.in_scope(ScopeId::ROOT, || {
        let old = h.context.choose("Home", None, false).unwrap();
        h.context.deliver(old);
        let newer = h.context.choose("End", None, false).unwrap();
        h.context.deliver(newer);
    });
    h.settle();
    assert_eq!(h.focuses.borrow().as_slice(), &[h.id("Footer")]);

    h.focuses.borrow_mut().clear();
    h.dom.in_scope(ScopeId::ROOT, || {
        let request = h.context.choose("Home", None, false).unwrap();
        h.context.deliver(request);
    });
    let replacement = MenuTestMount {
        id: usize::MAX,
        ready: Rc::new(Cell::new(true)),
        focuses: h.focuses.clone(),
        ..Default::default()
    };
    menu_dispatch(&h.dom, "mounted", h.target("Alpha"), replacement);
    h.settle();
    assert!(
        h.focuses.borrow().is_empty(),
        "an old intent cannot focus either replacement handle"
    );
    h.dom.in_scope(ScopeId::ROOT, || {
        let request = h.context.choose("Home", None, false).unwrap();
        h.context.deliver(request);
    });
    h.settle();
    assert_eq!(
        h.focuses.borrow().as_slice(),
        &[usize::MAX],
        "a fresh request uses the new handle"
    );

    h.focuses.borrow_mut().clear();
    h.dom.in_scope(ScopeId::ROOT, || {
        let request = h.context.choose("Home", None, false).unwrap();
        h.context.deliver(request);
    });
    h.config.borrow_mut().alpha = false;
    h.update();
    assert!(
        h.focuses.borrow().is_empty(),
        "unmount rejects the queued target"
    );
    assert!(
        !h.context
            .model
            .borrow()
            .members
            .contains_key(&h.id("Alpha"))
    );
}

#[test]
fn mounted_menu_pending_mounted_request_does_not_keep_its_owner_alive() {
    let _guard = crate::test_event_converter_guard();
    let h = mounted_menu();
    let retained = h.context.clone();
    let focuses = h.focuses.clone();
    h.dom.in_scope(ScopeId::ROOT, || {
        let request = retained.choose("Home", None, false).unwrap();
        retained.deliver(request);
    });
    drop(h);
    assert!(!retained.model.borrow().alive);
    assert!(focuses.borrow().is_empty());
}

#[test]
fn mounted_category_panel_navigation_and_return_reach_mounted_parent_and_children() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.click("Category");
    let category = mounted_category(&h);
    assert!(category.is_expanded() && category.is_open());
    h.focuses.borrow_mut().clear();
    assert_eq!(h.category_key("ArrowDown", "Category", None), (true, false));
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Child")));
    assert_eq!(
        h.category_key("ArrowDown", "Category", Some("Child")),
        (true, false)
    );
    assert_eq!(
        h.focuses.borrow().last().copied(),
        Some(h.id("Disabled child")),
        "expanded native buttons retain their ordinary ARIA-disabled traversal"
    );
    assert_eq!(
        h.category_key("Enter", "Category", Some("Child")),
        (true, false)
    );
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Category")));
    assert!(
        !category.is_open(),
        "expanded Enter explicitly toggles the panel closed"
    );
    assert_eq!(
        h.config.borrow().activations,
        0,
        "the original child default cannot also activate"
    );
    h.click("Category");
    assert!(category.is_open());
    assert_eq!(
        h.category_key("Escape", "Category", Some("Child")),
        (true, false)
    );
    assert!(!category.is_open());
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Category")));
}

#[test]
fn mounted_category_collapsed_end_and_accepted_dismissal_use_live_mounted_controls() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    h.click("Category");
    let category = mounted_category(&h);
    h.focuses.borrow_mut().clear();
    assert_eq!(h.category_key("End", "Category", None), (true, false));
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Two")));
    let before = h.focuses.borrow().len();
    assert_eq!(
        h.category_key("Tab", "Category", Some("Two")),
        (false, true)
    );
    assert_eq!(
        h.focuses.borrow().len(),
        before,
        "collapsed Tab keeps its native default"
    );
    assert_eq!(h.key(h.target("Two"), Key::Escape), (false, false));
    assert!(!category.is_open());
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Category")));
}

#[test]
fn mounted_category_mounted_focus_rejects_stale_mount_visibility_and_owner_lifetimes() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.click("Category");
    let category = mounted_category(&h);
    let child = h.id("Child");
    let two = h.id("Two");
    h.focuses.borrow_mut().clear();
    h.dom.in_scope(ScopeId::ROOT, || {
        category.focus(Some(child));
        category.focus(Some(two));
    });
    h.settle();
    assert_eq!(h.focuses.borrow().as_slice(), &[two]);

    h.focuses.borrow_mut().clear();
    h.dom
        .in_scope(ScopeId::ROOT, || category.focus(Some(child)));
    menu_dispatch(
        &h.dom,
        "mounted",
        h.target("Child"),
        MenuTestMount {
            id: usize::MAX,
            ready: Rc::new(Cell::new(true)),
            focuses: h.focuses.clone(),
            ..Default::default()
        },
    );
    h.settle();
    assert!(
        h.focuses.borrow().is_empty(),
        "replacement invalidates the captured child handle"
    );
    h.dom
        .in_scope(ScopeId::ROOT, || category.focus(Some(child)));
    h.settle();
    assert_eq!(h.focuses.borrow().as_slice(), &[usize::MAX]);

    h.focuses.borrow_mut().clear();
    h.dom.in_scope(ScopeId::ROOT, || {
        category.focus(Some(child));
        category.set_panel(false);
    });
    h.settle();
    assert!(
        h.focuses.borrow().is_empty(),
        "closing cannot focus hidden children"
    );
    h.click("Category");
    h.dom
        .in_scope(ScopeId::ROOT, || category.focus(Some(child)));
    h.config.borrow_mut().child_present = false;
    h.update();
    assert!(
        h.focuses.borrow().is_empty(),
        "removed child rejects pending focus"
    );
    assert!(!category.0.model.borrow().children.contains_key(&child));

    h.dom.in_scope(ScopeId::ROOT, || category.focus(Some(two)));
    let focuses = h.focuses.clone();
    drop(h);
    assert!(!category.0.model.borrow().alive);
    assert!(focuses.borrow().is_empty());
}
