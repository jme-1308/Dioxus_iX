// Native-observed Menu and Category navigation executes registered mounted controls.
use super::*;

fn mounted_native_menu() -> (MenuHarness, Vec<MenuDomItem>) {
    let mut h = MenuHarness::new();
    let facts = h.facts(&["Alpha", "Disabled", "Link", "Omega", "Footer"]);
    h.observe("ready", "", None, false, facts.clone());
    assert!(h.context.model.borrow().native);
    h.focuses.borrow_mut().clear();
    (h, facts)
}

fn native_category(h: &mut MenuHarness) -> (CategoryNavigation, Vec<CategoryDomItem>) {
    let category = h.category("Category");
    let facts = h.category_facts(&["Child", "Disabled child", "Two"]);
    h.category_event("category-order", "", "Category", None, 0, facts.clone());
    assert!(category.0.model.borrow().native);
    (category, facts)
}

#[test]
fn native_menu_roving_reaches_mounted_controls_and_keeps_eligible_endpoints() {
    let _guard = crate::test_event_converter_guard();
    let (mut h, mut facts) = mounted_native_menu();
    facts
        .iter_mut()
        .find(|item| item.id == h.id("Link"))
        .unwrap()
        .hidden = true;
    h.observe("order", "", None, false, facts.clone());
    for (key, origin, expected) in [
        ("ArrowDown", "Alpha", "Omega"),
        ("End", "Omega", "Footer"),
        ("Home", "Footer", "Alpha"),
        ("ArrowUp", "Alpha", "Footer"),
    ] {
        assert_eq!(
            h.observe("key", key, Some(h.id(origin)), false, facts.clone()),
            (true, false)
        );
        assert_eq!(
            h.focuses.borrow().last().copied(),
            Some(h.id(expected)),
            "native-observed navigation must call the registered MountedData"
        );
    }
    assert_eq!(h.focuses.borrow().len(), 4);
    assert_eq!(h.config.borrow().activations, 0);
}

#[test]
fn native_menu_mounted_requests_reject_newer_intent_and_replaced_or_removed_controls() {
    let _guard = crate::test_event_converter_guard();
    let (mut h, _) = mounted_native_menu();
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
fn native_menu_pending_mounted_request_does_not_keep_its_owner_alive() {
    let _guard = crate::test_event_converter_guard();
    let (h, _) = mounted_native_menu();
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
fn native_category_panel_navigation_and_return_reach_mounted_parent_and_children() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.click("Category");
    let (category, facts) = native_category(&mut h);
    assert!(category.is_expanded() && category.is_open());
    h.focuses.borrow_mut().clear();
    assert_eq!(
        h.category_event(
            "category-key",
            "ArrowDown",
            "Category",
            None,
            1,
            facts.clone()
        ),
        (true, false)
    );
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Child")));
    assert_eq!(
        h.category_event(
            "category-key",
            "ArrowDown",
            "Category",
            Some("Child"),
            2,
            facts.clone()
        ),
        (true, false)
    );
    assert_eq!(
        h.focuses.borrow().last().copied(),
        Some(h.id("Disabled child")),
        "expanded native buttons retain their ordinary ARIA-disabled traversal"
    );
    assert_eq!(
        h.category_event(
            "category-key",
            "Enter",
            "Category",
            Some("Child"),
            3,
            facts.clone()
        ),
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
        h.category_event(
            "category-key",
            "Escape",
            "Category",
            Some("Child"),
            4,
            facts
        ),
        (true, false)
    );
    assert!(!category.is_open());
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Category")));
}

#[test]
fn native_category_collapsed_end_and_accepted_dismissal_use_live_mounted_controls() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    h.click("Category");
    let (category, facts) = native_category(&mut h);
    h.focuses.borrow_mut().clear();
    assert_eq!(
        h.category_event("category-key", "End", "Category", None, 1, facts.clone()),
        (true, false)
    );
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Two")));
    let before = h.focuses.borrow().len();
    assert_eq!(
        h.category_event("category-key", "Tab", "Category", Some("Two"), 2, facts),
        (false, true)
    );
    assert_eq!(
        h.focuses.borrow().len(),
        before,
        "collapsed Tab keeps its native default"
    );
    h.document_mode("keydown", "ArrowDown", 1);
    h.dom.in_scope(ScopeId::ROOT, || {
        category.dismiss_captured(true, &[category.0.id])
    });
    h.settle();
    assert!(!category.is_open());
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Category")));
}

#[test]
fn native_category_mounted_focus_rejects_stale_mount_visibility_and_owner_lifetimes() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.click("Category");
    let (category, _) = native_category(&mut h);
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
