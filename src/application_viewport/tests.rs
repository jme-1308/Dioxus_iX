use super::*;
use ApplicationBreakpoint::{Large, Medium, Small};

fn facts(sm: bool, md: bool, lg: bool) -> DocumentMediaFacts {
    DocumentMediaFacts { sm, md, lg }
}

fn document_facts(
    width: f64,
    height: f64,
    media: Option<DocumentMediaFacts>,
) -> DocumentViewportFacts {
    DocumentViewportFacts {
        width,
        height,
        visual_left: 0.0,
        visual_top: 0.0,
        visual_width: width,
        visual_height: height,
        media,
    }
}

#[test]
fn viewport_resolves_force_initial_gap_and_supported_order() {
    assert_eq!(resolve(None, None, None), Large);
    assert_eq!(resolve(Some(Small), None, None), Small);
    assert_eq!(resolve(None, None, Some(facts(true, false, false))), Small);
    assert_eq!(resolve(None, None, Some(facts(false, true, false))), Medium);
    assert_eq!(resolve(None, None, Some(facts(false, false, true))), Large);
    assert_eq!(resolve(None, None, Some(facts(false, false, false))), Small);
    assert_eq!(
        resolve(
            None,
            Some(&[Small, Medium]),
            Some(facts(false, false, false))
        ),
        Medium
    );
    assert_eq!(
        resolve(
            None,
            Some(&[Medium, Large]),
            Some(facts(false, false, false))
        ),
        Medium
    );
    assert_eq!(
        resolve(None, Some(&[Large, Small]), Some(facts(true, false, true))),
        Small
    );
    assert_eq!(
        resolve(None, Some(&[]), Some(facts(true, false, false))),
        Large
    );
}

#[test]
fn viewport_resize_notifies_when_only_dimensions_change() {
    let mut model = ViewportModel::new(None, None);
    assert_eq!(model.viewport_size, None, "SSR has no document dimensions");
    assert!(!model.observed);
    assert!(model.observe(document_facts(
        1000.0,
        700.0,
        Some(facts(false, true, false))
    )));
    assert!(model.observed);
    assert_eq!(model.breakpoint(), Medium);
    assert!(
        model.observe(document_facts(
            1100.0,
            760.0,
            Some(facts(false, true, false))
        )),
        "geometry changes notify existing subscribers independently of media matches"
    );
    assert_eq!(model.breakpoint(), Medium);
    assert_eq!(model.viewport_size, Some((1100.0, 760.0)));
    assert!(!model.observe(document_facts(
        1100.0,
        760.0,
        Some(facts(false, true, false))
    )));
}

#[test]
fn viewport_dimensions_validate_geometry_independently_of_media_availability() {
    let mut model = ViewportModel::new(Some(Small), None);
    assert!(model.observe(document_facts(1000.0, 700.0, None)));
    assert_eq!(model.viewport_size, Some((1000.0, 700.0)));
    assert!(
        model.observed,
        "unavailable media is a completed native observation"
    );
    assert_eq!(model.breakpoint(), Small);
    for (width, height) in [
        (0.0, 700.0),
        (-1.0, 700.0),
        (1000.0, 0.0),
        (f64::NAN, 700.0),
        (1000.0, f64::INFINITY),
    ] {
        model.observe(document_facts(
            width,
            height,
            Some(facts(false, true, false)),
        ));
        assert_eq!(model.viewport_size, None);
        assert_eq!(
            model.breakpoint(),
            Small,
            "geometry cannot override a forced breakpoint"
        );
    }
    model.sync_inputs(None, None);
    model.observe(document_facts(1000.0, 700.0, None));
    assert_eq!(
        model.breakpoint(),
        Large,
        "unavailable media retains the explicit default"
    );
    assert_eq!(model.viewport_size, Some((1000.0, 700.0)));
}

#[test]
fn viewport_disposal_rejects_future_shared_facts_and_input_changes() {
    let mut model = ViewportModel::new(None, None);
    model.observe(document_facts(
        1000.0,
        700.0,
        Some(facts(false, true, false)),
    ));
    model.invalidate();
    assert!(!model.observe(document_facts(
        1400.0,
        900.0,
        Some(facts(false, false, true))
    )));
    assert!(!model.sync_inputs(Some(Small), Some(vec![Small])));
    assert_eq!(
        model.viewport_size, None,
        "owner teardown clears dimensions"
    );
    assert_eq!(
        model.breakpoint(),
        Medium,
        "late facts do not modify retired policy state"
    );
}

#[test]
fn viewport_shared_facts_resolve_latest_owner_inputs_without_replacing_them() {
    let mut model = ViewportModel::new(Some(Large), None);
    let small = document_facts(700.0, 700.0, Some(facts(true, false, false)));
    assert!(model.observe(small));
    assert_eq!(model.breakpoint(), Large);
    assert!(model.sync_inputs(None, Some(vec![Small, Medium, Large])));
    assert_eq!(model.breakpoint(), Small);
    // CSS media facts remain authoritative even if document width stays
    // unchanged (for example, a browser font preference changes em queries).
    assert!(model.observe(document_facts(
        700.0,
        700.0,
        Some(facts(false, true, false))
    )));
    assert_eq!(model.breakpoint(), Medium);
    assert_eq!(model.force, None);
    assert_eq!(model.supported, Some(vec![Small, Medium, Large]));
    assert!(!model.observe(document_facts(
        700.0,
        700.0,
        Some(facts(false, true, false))
    )));
}

#[derive(Clone)]
struct Capture(Rc<RefCell<Vec<(usize, ApplicationBreakpoint)>>>);
impl PartialEq for Capture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[component]
fn Consumer(capture: Capture) -> Element {
    let viewport = use_application_viewport(None, None);
    capture
        .0
        .borrow_mut()
        .push((Rc::as_ptr(&viewport.model) as usize, viewport.breakpoint()));
    rsx! { div {} }
}
#[component]
fn NestedOwner(capture: Capture) -> Element {
    let viewport = use_application_viewport(None, Some(DEFAULT_SUPPORTED.to_vec()));
    capture
        .0
        .borrow_mut()
        .push((Rc::as_ptr(&viewport.model) as usize, viewport.breakpoint()));
    rsx! { Consumer {capture} }
}
#[component]
fn Owner(capture: Capture) -> Element {
    let viewport = use_application_viewport(Some(Small), Some(DEFAULT_SUPPORTED.to_vec()));
    capture
        .0
        .borrow_mut()
        .push((Rc::as_ptr(&viewport.model) as usize, viewport.breakpoint()));
    rsx! { Consumer {capture:capture.clone()} NestedOwner {capture} }
}

#[test]
fn viewport_consumers_share_nearest_owner_but_nested_application_is_isolated() {
    let capture = Capture(Rc::new(RefCell::new(vec![])));
    let mut dom = VirtualDom::new_with_props(
        Owner,
        OwnerProps {
            capture: capture.clone(),
        },
    );
    dom.rebuild_in_place();
    let records = capture.0.borrow();
    assert_eq!(records.len(), 4);
    assert_eq!(records[0], records[1]);
    assert_eq!(records[0].1, Small);
    assert_ne!(records[0].0, records[2].0);
    assert_eq!(records[2], records[3]);
    assert_eq!(
        records[2].1, Large,
        "nested auto does not inherit outer force"
    );
}

#[derive(Clone)]
struct UpdatingCapture {
    force: Rc<RefCell<Option<Signal<Option<ApplicationBreakpoint>>>>>,
    values: Rc<RefCell<Vec<ViewportProjection>>>,
    owner: Rc<RefCell<Option<Rc<RefCell<ViewportModel>>>>>,
    document: Rc<RefCell<Option<crate::document_viewport::DocumentViewport>>>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct ViewportProjection {
    breakpoint: ApplicationBreakpoint,
    ready: bool,
    size: Option<(f64, f64)>,
}
impl PartialEq for UpdatingCapture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.force, &other.force)
    }
}
#[component]
fn UpdatingOwner(capture: UpdatingCapture) -> Element {
    let force = use_signal(|| Some(Small));
    *capture.force.borrow_mut() = Some(force);
    *capture.document.borrow_mut() = Some(use_document_viewport(false));
    let viewport = use_application_viewport(force(), Some(DEFAULT_SUPPORTED.to_vec()));
    *capture.owner.borrow_mut() = Some(viewport.model.clone());
    let current = use_memo(move || ViewportProjection {
        breakpoint: viewport.breakpoint(),
        ready: viewport.is_ready(),
        size: viewport.viewport_size(),
    });
    capture.values.borrow_mut().push(current());
    rsx! { div {} }
}

#[test]
fn viewport_input_changes_refresh_existing_memo_after_render_and_owner_drop_invalidates() {
    let capture = UpdatingCapture {
        force: Rc::new(RefCell::new(None)),
        values: Rc::new(RefCell::new(vec![])),
        owner: Rc::new(RefCell::new(None)),
        document: Rc::new(RefCell::new(None)),
    };
    let mut dom = VirtualDom::new_with_props(
        UpdatingOwner,
        UpdatingOwnerProps {
            capture: capture.clone(),
        },
    );
    dom.rebuild_in_place();
    let owner = capture.owner.borrow().as_ref().unwrap().clone();
    let mut force = capture.force.borrow().unwrap();
    assert_eq!(capture.values.borrow().last().unwrap().breakpoint, Small);
    for (requested, expected) in [(Some(Medium), Medium), (None, Large), (Some(Small), Small)] {
        dom.in_scope(ScopeId::ROOT, || force.set(requested));
        for _ in 0..4 {
            let _ = dom.render_immediate_to_vec();
        }
        assert_eq!(capture.values.borrow().last().unwrap().breakpoint, expected);
        assert_eq!(
            capture.values.borrow().last().unwrap().ready,
            requested.is_some()
        );
        assert_eq!(capture.values.borrow().last().unwrap().size, None);
        assert!(Rc::ptr_eq(capture.owner.borrow().as_ref().unwrap(), &owner));
    }
    drop(dom);
    let mut owner = owner.borrow_mut();
    assert!(!owner.alive);
    let late = document_facts(1000.0, 700.0, Some(facts(false, true, false)));
    assert!(!owner.observe(late));
}

#[test]
fn viewport_shared_facts_refresh_existing_memo_geometry_and_unavailable_media() {
    let capture = UpdatingCapture {
        force: Rc::new(RefCell::new(None)),
        values: Rc::new(RefCell::new(vec![])),
        owner: Rc::new(RefCell::new(None)),
        document: Rc::new(RefCell::new(None)),
    };
    let mut dom = VirtualDom::new_with_props(
        UpdatingOwner,
        UpdatingOwnerProps {
            capture: capture.clone(),
        },
    );
    dom.rebuild_in_place();
    let document = capture.document.borrow().clone().unwrap();
    let mut force = capture.force.borrow().unwrap();
    for (requested, media, size, expected) in [
        (
            Some(Small),
            Some(facts(false, true, false)),
            (1000.0, 700.0),
            Small,
        ),
        (
            None,
            Some(facts(false, true, false)),
            (1000.0, 700.0),
            Medium,
        ),
        (
            None,
            Some(facts(false, true, false)),
            (1100.0, 760.0),
            Medium,
        ),
        (None, None, (1150.0, 780.0), Large),
        (
            Some(Small),
            Some(facts(false, false, true)),
            (1360.0, 900.0),
            Small,
        ),
    ] {
        dom.in_scope(ScopeId::ROOT, || {
            force.set(requested);
            document.test_facts(document_facts(size.0, size.1, media));
        });
        for _ in 0..5 {
            let _ = dom.render_immediate_to_vec();
        }
        assert_eq!(
            capture.values.borrow().last(),
            Some(&ViewportProjection {
                breakpoint: expected,
                ready: true,
                size: Some(size),
            })
        );
    }
    let owner = capture.owner.borrow().clone().unwrap();
    drop(dom);
    document.test_facts(document_facts(
        700.0,
        600.0,
        Some(facts(true, false, false)),
    ));
    assert!(!owner.borrow().alive);
    assert_eq!(owner.borrow().viewport_size, None);
    assert_eq!(owner.borrow().force, Some(Small));
}
