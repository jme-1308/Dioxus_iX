use super::*;
use std::cell::Cell;

struct Model {
    open: Rc<Cell<bool>>,
    allow: Rc<Cell<bool>>,
}

fn model(
    domain: &Domain,
    owner: &str,
    id: &str,
    kind: OverlayKind,
    log: Rc<RefCell<Vec<String>>>,
) -> Model {
    let open = Rc::new(Cell::new(false));
    let allow = Rc::new(Cell::new(true));
    let read = open.clone();
    let apply = open.clone();
    let accept = allow.clone();
    let label = id.to_owned();
    domain.register(
        owner.to_owned(),
        id.to_owned(),
        1,
        kind,
        CloseBehavior::Both,
        DomainCallbacks {
            suppress_trigger_restore: Rc::new(|| true),
            read: Rc::new(move || Some(read.get())),
            request: Rc::new(move |change| {
                log.borrow_mut().push(format!(
                    "{label}:{}:{:?}:{}",
                    change.open, change.reason, change.restore_focus
                ));
                if !accept.get() {
                    return false;
                }
                apply.set(change.open);
                true
            }),
        },
    );
    Model { open, allow }
}

fn fact(id: &str, ancestors: &[&str]) -> NativeFact {
    NativeFact {
        id: id.into(),
        generation: 1,
        connected: true,
        trigger_ancestors: ancestors.iter().map(|id| (*id).to_owned()).collect(),
        assignment_ancestors: None,
        discovery_ready: true,
    }
}

fn close() -> Change {
    Change {
        open: false,
        reason: Reason::Outside,
        restore_focus: false,
    }
}

#[test]
fn replacing_foreign_parent_requires_a_fresh_child_owner_relationship() {
    let domain = Domain::join("explicit cross-owner parent lifetime");
    let log = Rc::new(RefCell::new(Vec::new()));
    let _parent = model(
        &domain,
        "owner-a",
        "parent",
        OverlayKind::Dropdown,
        log.clone(),
    );
    let _child = model(&domain, "owner-b", "child", OverlayKind::Dropdown, log);
    domain.observe_owner("owner-a", &[fact("parent", &[])]);
    domain.observe_owner("owner-b", &[fact("child", &["parent"])]);
    assert_eq!(domain.cached_parent("child").as_deref(), Some("parent"));
    let callbacks = domain.0.borrow().entries["parent"].callbacks.clone();
    domain.register(
        "owner-a".into(),
        "parent".into(),
        2,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        callbacks.clone(),
    );
    let mut replacement = fact("parent", &[]);
    replacement.generation = 2;
    domain.observe_owner("owner-a", &[replacement]);
    assert_eq!(
        domain.cached_parent("child"),
        None,
        "publishing only the parent owner's replacement cannot revive an old child relationship"
    );
    domain.observe_owner("owner-b", &[fact("child", &["parent"])]);
    assert_eq!(domain.cached_parent("child").as_deref(), Some("parent"));
    domain.unregister("parent", 2);
    domain.register(
        "owner-a".into(),
        "parent".into(),
        3,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        callbacks,
    );
    let mut remounted = fact("parent", &[]);
    remounted.generation = 3;
    domain.observe_owner("owner-a", &[remounted]);
    assert_eq!(
        domain.cached_parent("child"),
        None,
        "unregister and same-ID remount also require a fresh child-owner relationship"
    );
    domain.observe_owner("owner-b", &[fact("child", &["parent"])]);
    assert_eq!(domain.cached_parent("child").as_deref(), Some("parent"));
}

#[test]
fn popover_prepare_uses_current_ancestors_policy_veto_and_registration_order() {
    let domain = Domain::join("popover controller prepare");
    let log = Rc::new(RefCell::new(Vec::new()));
    let parent = model(&domain, "a", "parent", OverlayKind::Popover, log.clone());
    let target = model(&domain, "a", "target", OverlayKind::Popover, log.clone());
    let sibling = model(&domain, "b", "sibling", OverlayKind::Popover, log.clone());
    let sticky = model(&domain, "b", "sticky", OverlayKind::Popover, log.clone());
    let veto = model(&domain, "b", "veto", OverlayKind::Popover, log.clone());
    let closed = model(&domain, "b", "closed", OverlayKind::Popover, log.clone());
    let dropdown = model(&domain, "c", "dropdown", OverlayKind::Dropdown, log.clone());
    domain.observe_all(&[
        fact("parent", &[]),
        fact("target", &["parent"]),
        fact("sibling", &["parent"]),
        fact("sticky", &[]),
        fact("veto", &[]),
        fact("closed", &[]),
        fact("dropdown", &[]),
    ]);
    for open in [&parent, &sibling, &sticky, &veto, &dropdown] {
        open.open.set(true);
    }
    domain.set_close_behavior("sticky", 1, CloseBehavior::Inside);
    veto.allow.set(false);
    assert!(domain.prepare_present("target"));
    assert!(parent.open.get() && sticky.open.get() && veto.open.get());
    assert!(dropdown.open.get());
    assert!(!target.open.get() && !sibling.open.get() && !closed.open.get());
    assert_eq!(
        *log.borrow(),
        [
            "sibling:false:Outside:true",
            "veto:false:Outside:true",
            "closed:false:Outside:true"
        ]
    );
    assert_eq!(domain.cached_parent("target").as_deref(), Some("parent"));
    assert_ne!(domain.topmost().as_deref(), Some("target"));
    domain.observe_owner("a", &[fact("parent", &[]), fact("target", &[])]);
    assert_eq!(
        domain.cached_parent("target"),
        None,
        "detachment updates the current relationship"
    );
}

#[test]
fn popover_children_follow_late_binding_and_detach_while_parent_is_open() {
    let domain = Domain::join("popover current children");
    let log = Rc::new(RefCell::new(Vec::new()));
    let parent = model(&domain, "a", "parent", OverlayKind::Popover, log.clone());
    let child = model(&domain, "b", "child", OverlayKind::Popover, log);
    let mut pending = fact("child", &["parent"]);
    pending.discovery_ready = false;
    domain.observe_all(&[fact("parent", &[]), pending.clone()]);
    assert!(domain.prepare_present("parent"));
    parent.open.set(true);
    domain.presented("parent", 1);
    assert!(domain.cached_children("parent").is_empty());
    pending.discovery_ready = true;
    domain.observe_owner("b", &[pending]);
    assert_eq!(domain.cached_children("parent"), ["child"]);
    assert_eq!(domain.topmost().as_deref(), Some("parent"));
    child.open.set(true);
    domain.presented("child", 1);
    domain.observe_owner("b", &[fact("child", &[])]);
    assert!(domain.cached_children("parent").is_empty());
    domain.close_children("parent", Reason::Parent, true);
    assert!(
        parent.open.get() && child.open.get(),
        "a detached child is no longer dismissed by its former parent"
    );
}

#[test]
fn presenting_popover_ancestor_dismisses_current_descendant() {
    let domain = Domain::join("popover descendant opening");
    let log = Rc::new(RefCell::new(Vec::new()));
    let parent = model(&domain, "a", "parent", OverlayKind::Popover, log.clone());
    let child = model(&domain, "b", "child", OverlayKind::Popover, log.clone());
    domain.observe_all(&[fact("parent", &[]), fact("child", &["parent"])]);
    child.open.set(true);
    domain.presented("child", 1);
    assert!(domain.prepare_present("parent"));
    assert!(!parent.open.get() && !child.open.get());
    assert_eq!(*log.borrow(), ["child:false:Outside:true"]);
    assert_eq!(domain.cached_children("parent"), ["child"]);
}

#[test]
fn prepare_present_visits_new_registration_without_holding_callback_borrow() {
    let domain = Domain::join("popover live registration pass");
    let log = Rc::new(RefCell::new(Vec::new()));
    let target = model(&domain, "a", "target", OverlayKind::Popover, log.clone());
    let append = domain.clone();
    let append_log = log.clone();
    let once = Rc::new(Cell::new(true));
    domain.register(
        "a".into(),
        "first".into(),
        1,
        OverlayKind::Popover,
        CloseBehavior::Both,
        DomainCallbacks {
            suppress_trigger_restore: Rc::new(|| true),
            read: Rc::new(|| Some(false)),
            request: Rc::new(move |_| {
                append_log.borrow_mut().push("first".into());
                if once.replace(false) {
                    let late_log = append_log.clone();
                    append.register(
                        "b".into(),
                        "late".into(),
                        1,
                        OverlayKind::Popover,
                        CloseBehavior::Both,
                        DomainCallbacks {
                            suppress_trigger_restore: Rc::new(|| true),
                            read: Rc::new(|| Some(false)),
                            request: Rc::new(move |_| {
                                late_log.borrow_mut().push("late".into());
                                true
                            }),
                        },
                    );
                }
                false
            }),
        },
    );
    domain.observe_all(&[fact("target", &[]), fact("first", &[])]);
    assert!(domain.prepare_present("target"));
    assert!(!target.open.get());
    assert_eq!(*log.borrow(), ["first", "late"]);
}

#[test]
fn prepare_present_rejects_removed_target_after_dismissal_callback() {
    let domain = Domain::join("popover removed target");
    let log = Rc::new(RefCell::new(Vec::new()));
    let target = model(&domain, "target-owner", "target", OverlayKind::Popover, log);
    let remove = domain.clone();
    domain.register(
        "other-owner".into(),
        "other".into(),
        1,
        OverlayKind::Popover,
        CloseBehavior::Both,
        DomainCallbacks {
            suppress_trigger_restore: Rc::new(|| true),
            read: Rc::new(|| Some(true)),
            request: Rc::new(move |_| {
                remove.remove_owner("target-owner");
                true
            }),
        },
    );
    domain.observe_all(&[fact("target", &[]), fact("other", &[])]);
    assert!(!domain.prepare_present("target"));
    assert!(!target.open.get());
    assert!(domain.entry("target").is_none());
    assert!(!domain.prepare_present("missing"));
}

#[test]
fn native_document_identity_shares_owners_and_isolates_other_documents() {
    let first = Domain::join("same native document");
    let second = Domain::join("same native document");
    let foreign = Domain::join("another native document");
    assert!(Rc::ptr_eq(&first.0, &second.0));
    assert!(!Rc::ptr_eq(&first.0, &foreign.0));
    let log = Rc::new(RefCell::new(Vec::new()));
    let a = model(&first, "a", "a", OverlayKind::Dropdown, log.clone());
    let b = model(&second, "b", "b", OverlayKind::Dropdown, log.clone());
    first.observe_owner("a", &[fact("a", &[])]);
    second.observe_owner("b", &[fact("b", &[])]);
    a.open.set(true);
    first.sync_open("a", 1, true);
    b.open.set(true);
    second.sync_open("b", 1, true);
    assert_eq!(first.topmost().as_deref(), Some("b"));
    assert!(foreign.topmost().is_none());
    assert!(first.escape());
    assert!(a.open.get() && !b.open.get());
    assert_eq!(*log.borrow(), ["b:false:Escape:true"]);
}

#[test]
fn owner_partition_does_not_disconnect_or_accept_facts_for_foreign_nodes() {
    let domain = Domain::join("partitioned");
    let log = Rc::new(RefCell::new(Vec::new()));
    let a = model(&domain, "owner-a", "a", OverlayKind::Dropdown, log.clone());
    let b = model(&domain, "owner-b", "b", OverlayKind::Dropdown, log);
    domain.observe_owner("owner-a", &[fact("a", &[])]);
    domain.observe_owner("owner-b", &[fact("b", &[])]);
    a.open.set(true);
    b.open.set(true);
    domain.sync_open("a", 1, true);
    domain.sync_open("b", 1, true);
    domain.observe_owner(
        "owner-a",
        &[NativeFact {
            id: "b".into(),
            generation: 1,
            ..NativeFact::default()
        }],
    );
    assert_eq!(domain.topmost().as_deref(), Some("b"));
    domain.remove_owner("owner-a");
    assert_eq!(domain.topmost().as_deref(), Some("b"));
    domain.observe_all(&[]);
    assert!(
        domain.topmost().is_none(),
        "complete document snapshot can disconnect b"
    );
}

#[test]
fn one_document_delivery_is_accepted_once_across_owners() {
    let first = Domain::join("event document");
    let second = Domain::join("event document");
    let foreign = Domain::join("event foreign document");
    assert!(!first.accept_event(0));
    assert!(first.accept_event(7));
    assert!(!second.accept_event(7));
    assert!(
        second.is_current_event(7),
        "the same input broadcast can reach another local cue"
    );
    assert!(!second.accept_event(6));
    assert!(!second.is_current_event(6));
    assert!(second.accept_event(8));
    assert!(
        !first.accept_event(7),
        "an older delivery cannot overwrite the newer document event"
    );
    assert!(!first.is_current_event(7));
    assert!(first.is_current_event(8));
    assert!(foreign.accept_event(7));
}

#[test]
fn document_bubble_modality_uses_exact_source_keys_and_preserves_unmatched_input() {
    let domain = Domain::join("shared input document");
    let second_owner = Domain::join("shared input document");
    let foreign = Domain::join("foreign input document");
    assert_eq!(domain.keyboard_mode(), None);
    for key in [
        "a", "PageDown", "PageUp", "Control", "Meta", "Alt", "Spacebar",
    ] {
        domain.observe_input("keydown", key);
        assert_eq!(domain.keyboard_mode(), None);
    }
    for key in [
        "Tab",
        "ArrowDown",
        "Space",
        "Escape",
        " ",
        "Shift",
        "Enter",
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "Home",
        "End",
    ] {
        domain.observe_input("mousedown", "");
        assert_eq!(second_owner.keyboard_mode(), Some(false));
        second_owner.observe_input("keydown", key);
        assert_eq!(
            domain.keyboard_mode(),
            Some(true),
            "recognized source key {key:?}"
        );
        for (kind, unmatched) in [
            ("keydown", "a"),
            ("pointerdown", ""),
            ("mousemove", ""),
            ("keyup", key),
        ] {
            second_owner.observe_input(kind, unmatched);
            assert_eq!(domain.keyboard_mode(), Some(true));
        }
    }
    domain.observe_input("touchstart", "");
    domain.observe_input("keydown", "PageDown");
    assert_eq!(second_owner.keyboard_mode(), Some(false));
    assert_eq!(foreign.keyboard_mode(), None);
    foreign.observe_input("keydown", "Tab");
    assert_eq!(foreign.keyboard_mode(), Some(true));
    assert_eq!(domain.keyboard_mode(), Some(false));
}

#[test]
fn vetoed_topmost_escape_never_falls_through() {
    let domain = Domain::join("veto escape");
    let log = Rc::new(RefCell::new(Vec::new()));
    let a = model(&domain, "a", "a", OverlayKind::Dropdown, log.clone());
    let b = model(&domain, "b", "b", OverlayKind::Dropdown, log.clone());
    domain.observe_all(&[fact("a", &[]), fact("b", &[])]);
    a.open.set(true);
    domain.sync_open("a", 1, true);
    b.open.set(true);
    domain.sync_open("b", 1, true);
    b.allow.set(false);
    assert!(!domain.escape());
    assert!(a.open.get() && b.open.get());
    assert_eq!(*log.borrow(), ["b:false:Escape:true"]);
}

#[test]
fn outside_snapshot_rechecks_later_presence_and_reverse_presentation_order() {
    let domain = Domain::join("outside order");
    let log = Rc::new(RefCell::new(Vec::new()));
    let a = model(&domain, "a", "a", OverlayKind::Dropdown, log.clone());
    let b = model(&domain, "b", "b", OverlayKind::Dropdown, log.clone());
    domain.observe_all(&[fact("a", &[]), fact("b", &[])]);
    b.open.set(true);
    domain.sync_open("b", 1, true);
    a.open.set(true);
    domain.sync_open("a", 1, true);
    domain.sync_open("b", 1, true);
    a.allow.set(false);
    assert!(domain.outside(&[], &[]));
    assert!(a.open.get() && !b.open.get());
    assert_eq!(
        *log.borrow(),
        ["a:false:Outside:false", "b:false:Outside:false"]
    );
}

#[test]
fn departing_dropdown_click_preserves_new_peer_until_a_genuine_outside_click() {
    let domain = Domain::join("selection hands off to another field");
    let log = Rc::new(RefCell::new(Vec::new()));
    let first = model(&domain, "a", "first", OverlayKind::Dropdown, log.clone());
    let next = model(&domain, "b", "next", OverlayKind::Dropdown, log.clone());
    domain.observe_all(&[fact("first", &[]), fact("next", &[])]);
    first.open.set(true);
    domain.sync_open("first", 1, true);

    // The selection handler commits, closes the first field, and opens its
    // peer before the original click reaches the shared document observer.
    first.open.set(false);
    next.open.set(true);
    assert!(!domain.outside(&["first".into()], &[]));
    assert!(!first.open.get() && next.open.get());
    assert!(log.borrow().is_empty());

    assert!(domain.outside(&[], &[]));
    assert!(!next.open.get());
    assert_eq!(*log.borrow(), ["next:false:Outside:false"]);
}

#[test]
fn current_assignment_follows_cross_owner_trigger_detachment() {
    let domain = Domain::join("current cross-owner detachment");
    let log = Rc::new(RefCell::new(Vec::new()));
    let parent = model(&domain, "a", "parent", OverlayKind::Dropdown, log.clone());
    let child = model(&domain, "b", "child", OverlayKind::Dropdown, log.clone());
    domain.observe_owner("a", &[fact("parent", &[])]);
    domain.observe_owner("b", &[fact("child", &["parent"])]);
    parent.open.set(true);
    domain.presented("parent", 1);
    child.open.set(true);
    domain.presented("child", 1);
    assert_eq!(domain.cached_root("child").as_deref(), Some("parent"));
    assert!(domain.owns_topmost("parent"));
    domain.observe_owner("b", &[fact("child", &[])]);
    assert!(domain.live_ancestors("child").is_empty());
    assert_eq!(domain.cached_root("child").as_deref(), Some("child"));
    assert!(domain.escape());
    assert_eq!(*log.borrow(), ["child:false:Escape:true"]);
    assert!(parent.open.get() && !child.open.get());
}

#[test]
fn popover_escape_selects_self_and_header_adapter_suppresses_restore() {
    let domain = Domain::join("kind-specific escape");
    let log = Rc::new(RefCell::new(Vec::new()));
    let parent = model(&domain, "a", "parent", OverlayKind::Popover, log.clone());
    let child = model(&domain, "b", "child", OverlayKind::Popover, log.clone());
    domain.observe_all(&[fact("parent", &[]), fact("child", &["parent"])]);
    parent.open.set(true);
    domain.sync_open("parent", 1, true);
    child.open.set(true);
    domain.sync_open("child", 1, true);
    assert!(domain.escape());
    assert!(parent.open.get() && !child.open.get());
    let header = model(&domain, "a", "header", OverlayKind::Header, log.clone());
    domain.observe_owner("a", &[fact("parent", &[]), fact("header", &[])]);
    header.open.set(true);
    domain.sync_open("header", 1, true);
    assert!(domain.escape());
    assert_eq!(
        *log.borrow(),
        ["child:false:Escape:true", "header:false:Escape:false"]
    );
}

#[test]
fn current_child_focus_policy_propagates_only_same_type_popover_restore() {
    for kind in [OverlayKind::Popover, OverlayKind::Dropdown] {
        for restore in [false, true] {
            let domain = Domain::join(&format!("current child focus {kind:?} {restore}"));
            let log = Rc::new(RefCell::new(Vec::new()));
            let parent = model(&domain, "a", "parent", kind, log.clone());
            let child = model(&domain, "b", "child", kind, log.clone());
            domain.observe_all(&[fact("parent", &[]), fact("child", &["parent"])]);
            parent.open.set(true);
            domain.presented("parent", 1);
            child.open.set(true);
            domain.presented("child", 1);
            assert_eq!(domain.cached_children("parent"), ["child"]);

            domain.close_children("parent", Reason::Parent, restore);
            assert!(parent.open.get() && !child.open.get());
            let expected_restore = kind == OverlayKind::Popover && restore;
            assert_eq!(
                *log.borrow(),
                [format!("child:false:Parent:{expected_restore}")]
            );
        }
    }
}

#[test]
fn cross_type_child_close_uses_live_parent_and_child_focus_policy() {
    for (parent_kind, child_kind, expected_restore) in [
        (OverlayKind::Dropdown, OverlayKind::Popover, true),
        (OverlayKind::Popover, OverlayKind::Dropdown, false),
    ] {
        let domain = Domain::join(&format!("cross type focus {parent_kind:?}"));
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&domain, "a", "parent", parent_kind, log.clone());
        let child = model(&domain, "b", "child", child_kind, log.clone());
        domain.observe_all(&[fact("parent", &[]), fact("child", &["parent"])]);
        parent.open.set(true);
        domain.presented("parent", 1);
        child.open.set(true);
        domain.presented("child", 1);
        assert!(domain.cached_children("parent").is_empty());
        assert_eq!(domain.live_ancestors("child"), ["parent"]);

        // Source cross-type parent-close selects the child's policy even
        // when the parent request releases its own trigger focus.
        domain.close_children("parent", Reason::Parent, false);
        assert!(parent.open.get() && !child.open.get());
        assert_eq!(
            *log.borrow(),
            [format!("child:false:Parent:{expected_restore}")]
        );

        // Detachment also removes cross-type focus ancestry.
        log.borrow_mut().clear();
        child.open.set(true);
        domain.presented("child", 1);
        domain.observe_owner("b", &[fact("child", &[])]);
        domain.close_children("parent", Reason::Parent, true);
        assert!(child.open.get());
        assert!(log.borrow().is_empty());
    }
}

#[test]
fn native_child_before_foreign_owner_registration_retries_controller_discovery() {
    let domain = Domain::join("late native ancestor owner");
    let log = Rc::new(RefCell::new(Vec::new()));
    let child = model(
        &domain,
        "child owner",
        "child",
        OverlayKind::Dropdown,
        log.clone(),
    );
    // The native foreign host is already an ancestor, but its Rust owner
    // has not registered yet. This must not latch a permanent root result.
    domain.observe_owner("child owner", &[fact("child", &["parent"])]);
    child.open.set(true);
    domain.presented("child", 1);
    assert_eq!(domain.cached_root("child").as_deref(), Some("child"));

    let parent = model(
        &domain,
        "parent owner",
        "parent",
        OverlayKind::Dropdown,
        log.clone(),
    );
    domain.observe_owner("parent owner", &[fact("parent", &[])]);
    // Parent readiness admits the retained current ancestry immediately;
    // no replay or child presentation is required.
    assert_eq!(domain.cached_root("child").as_deref(), Some("parent"));
    parent.open.set(true);
    domain.presented("parent", 1);
    assert_eq!(domain.cached_children("parent"), ["child"]);
    assert_eq!(domain.cached_root("child").as_deref(), Some("parent"));
    domain.close_children("parent", Reason::Parent, false);
    assert!(parent.open.get() && !child.open.get());
    assert_eq!(*log.borrow(), ["child:false:Parent:false"]);
}

#[test]
fn callbacks_can_reenter_and_remove_an_owner_without_domain_borrows() {
    let domain = Domain::join("reentrant request");
    let open = Rc::new(Cell::new(true));
    let read = open.clone();
    let apply = open.clone();
    let nested = domain.clone();
    domain.register(
        "a".into(),
        "a".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        DomainCallbacks {
            suppress_trigger_restore: Rc::new(|| true),
            read: Rc::new(move || Some(read.get())),
            request: Rc::new(move |_| {
                assert!(nested.accept_event(19));
                nested.remove_owner("a");
                apply.set(false);
                true
            }),
        },
    );
    domain.observe_all(&[fact("a", &[])]);
    assert!(domain.request("a", close()));
    assert!(!open.get() && domain.topmost().is_none());
}

#[test]
fn category_broadcast_skips_own_id_and_resumes_outer_registration_order() {
    let domain = Domain::join("category nested broadcasts");
    let log = Rc::new(RefCell::new(Vec::new()));
    let first = log.clone();
    let nested = Rc::downgrade(&domain.0);
    domain.register_category(
        "a".into(),
        1,
        Rc::new(move |source, active| {
            first.borrow_mut().push(format!("a:{source}:{active:?}"));
            if source == 99 {
                Domain(nested.upgrade().unwrap()).broadcast_category(1, &[1]);
            }
        }),
    );
    let second = log.clone();
    domain.register_category(
        "b".into(),
        2,
        Rc::new(move |source, active| {
            second.borrow_mut().push(format!("b:{source}:{active:?}"));
        }),
    );
    domain.broadcast_category(99, &[2]);
    assert_eq!(*log.borrow(), ["a:99:[2]", "b:1:[1]", "b:99:[2]"]);
    domain.unregister_category("wrong owner", 2);
    domain.remove_owner("a");
    log.borrow_mut().clear();
    domain.broadcast_category(99, &[]);
    assert_eq!(*log.borrow(), ["b:99:[]"]);
    domain.unregister_category("b", 2);
    assert!(domain.0.borrow().categories.is_empty());
}

#[test]
fn replacement_during_read_rejects_stale_callback_and_old_generation_cleanup() {
    let domain = Domain::join("replacement during read");
    let replacing = domain.clone();
    let once = Rc::new(Cell::new(true));
    domain.register(
        "a".into(),
        "node".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        DomainCallbacks {
            suppress_trigger_restore: Rc::new(|| true),
            read: Rc::new(move || {
                if once.replace(false) {
                    replacing.register(
                        "b".into(),
                        "node".into(),
                        2,
                        OverlayKind::Dropdown,
                        CloseBehavior::Both,
                        DomainCallbacks {
                            suppress_trigger_restore: Rc::new(|| true),
                            read: Rc::new(|| Some(true)),
                            request: Rc::new(|_| false),
                        },
                    );
                }
                Some(true)
            }),
            request: Rc::new(|_| panic!("replaced callback cannot receive a command")),
        },
    );
    assert!(!domain.request("node", close()));
    domain.unregister("node", 1);
    domain.remove_owner("a");
    assert_eq!(domain.entry("node").unwrap().generation, 2);
}

#[test]
fn surface_metadata_rechecks_reentrant_updates_removal_and_disconnection() {
    let domain = Domain::join("authored surface reentry");
    let log = Rc::new(RefCell::new(Vec::new()));
    model(&domain, "a", "a", OverlayKind::Dropdown, log.clone());
    model(&domain, "b", "b", OverlayKind::Popover, log.clone());
    model(&domain, "c", "c", OverlayKind::Dropdown, log);
    domain.observe_all(&[fact("a", &[]), fact("b", &[]), fact("c", &[])]);
    let metadata = |trigger: &str| SurfaceMetadata {
        trigger_id: trigger.into(),
        ..Default::default()
    };
    domain.set_surface_metadata("a", "a", 1, metadata("old"));
    domain.set_surface_metadata("b", "b", 1, metadata("b-trigger"));
    domain.set_surface_reader("a", "a", 1, Rc::new(|| Some(Rc::new(|| true))));
    domain.set_surface_reader(
        "c",
        "c",
        1,
        Rc::new(|| panic!("removed reader must not run")),
    );
    let update = domain.clone();
    domain.set_surface_reader(
        "b",
        "b",
        1,
        Rc::new(move || {
            update.set_surface_metadata(
                "a",
                "a",
                1,
                SurfaceMetadata {
                    trigger_id: "current".into(),
                    ..Default::default()
                },
            );
            update.remove_owner("c");
            Some(Rc::new(|| true))
        }),
    );
    let surfaces = domain.surface_metadata();
    assert_eq!(surfaces.len(), 2);
    assert_eq!(
        surfaces[0].2.trigger_id, "current",
        "read current metadata after later callbacks"
    );
    assert_eq!(surfaces[1].2.trigger_id, "b-trigger");
    domain.set_surface_reader("a", "a", 1, Rc::new(|| None));
    assert_eq!(
        domain
            .surface_metadata()
            .iter()
            .map(|surface| surface.0.as_str())
            .collect::<Vec<_>>(),
        ["b"]
    );
    let mut disconnected = fact("b", &[]);
    disconnected.connected = false;
    domain.observe_all(&[disconnected]);
    assert!(domain.surface_metadata().is_empty());
}

#[test]
fn weak_document_table_does_not_retain_dropped_domains() {
    let domain = Domain::join("weak lifetime");
    let weak = Rc::downgrade(&domain.0);
    assert!(domain.accept_event(42));
    drop(domain);
    assert!(weak.upgrade().is_none());
    let fresh = Domain::join("weak lifetime");
    assert!(
        fresh.accept_event(1),
        "zero-subscriber state retention is intentionally not claimed"
    );
}
