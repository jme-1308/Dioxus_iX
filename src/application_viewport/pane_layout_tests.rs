use super::*;
use crate::{Pane, PaneLayout};
use ApplicationBreakpoint::{Large, Medium, Small};
use std::{cell::Cell, collections::BTreeMap};

#[derive(Clone)]
struct Snapshot {
    viewport: ApplicationViewport,
    breakpoint: ApplicationBreakpoint,
    ready: bool,
    size: Option<(f64, f64)>,
}
#[derive(Clone, PartialEq)]
struct Settings {
    visible: bool,
    left: bool,
    force: ApplicationBreakpoint,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            visible: true,
            left: true,
            force: Large,
        }
    }
}
#[derive(Clone, Copy, Default)]
enum Topology {
    #[default]
    Single,
    Ancestor,
    Siblings,
}
#[derive(Default)]
struct Capture {
    topology: Topology,
    settings: Cell<Option<Signal<Settings>>>,
    views: RefCell<BTreeMap<String, Snapshot>>,
    document: RefCell<Option<crate::document_viewport::DocumentViewport>>,
}
impl PartialEq for Capture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn Probe(capture: Rc<Capture>, name: String) -> Element {
    // A pure context reader: this must never establish its own viewport policy.
    let viewport = use_context::<ApplicationViewport>();
    let breakpoint = viewport.breakpoint();
    let ready = viewport.is_ready();
    let size = viewport.viewport_size();
    capture.views.borrow_mut().insert(
        name.clone(),
        Snapshot {
            viewport,
            breakpoint,
            ready,
            size,
        },
    );
    rsx! {span {"data-policy-probe":name,"{breakpoint:?}"}}
}
#[component]
fn ActualLayout(
    capture: Rc<Capture>,
    prefix: String,
    #[props(default = true)] left: bool,
    children: Option<Element>,
) -> Element {
    let left=left.then(||rsx!{Pane {heading:format!("{prefix} left"),Probe {capture:capture.clone(),name:format!("{prefix}-left")}}});
    rsx! {PaneLayout {
        left,
        right:rsx!{Pane {heading:format!("{prefix} right"),Probe {capture:capture.clone(),name:format!("{prefix}-right")}}},
        top:rsx!{Pane {heading:format!("{prefix} top"),Probe {capture:capture.clone(),name:format!("{prefix}-top")}}},
        bottom:rsx!{Pane {heading:format!("{prefix} bottom"),Probe {capture:capture.clone(),name:format!("{prefix}-bottom")}}},
        Probe {capture,name:format!("{prefix}-owner")}
        {children}
    }}
}
#[component]
fn ForcedOwner(
    capture: Rc<Capture>,
    name: String,
    force: ApplicationBreakpoint,
    children: Element,
) -> Element {
    let _owner = use_application_viewport(Some(force), Some(DEFAULT_SUPPORTED.to_vec()));
    rsx! {Probe {capture,name}{children}}
}
fn fixture(capture: Rc<Capture>) -> Element {
    let settings = use_signal(Settings::default);
    capture.settings.set(Some(settings));
    *capture.document.borrow_mut() = Some(use_document_viewport(false));
    let current = settings();
    match capture.topology {
        Topology::Single => {
            rsx! {if current.visible {ActualLayout {capture,prefix:"solo",left:current.left}}}
        }
        Topology::Siblings => {
            rsx! {ActualLayout {capture:capture.clone(),prefix:"a"}ActualLayout {capture,prefix:"b"}}
        }
        Topology::Ancestor => {
            rsx! {ForcedOwner {capture:capture.clone(),name:"ancestor",force:current.force,
                ActualLayout {capture:capture.clone(),prefix:"outer",
                    ActualLayout {capture:capture.clone(),prefix:"nested"}
                    ForcedOwner {capture:capture.clone(),name:"nearest",force:Small,
                        ActualLayout {capture,prefix:"override"}
                    }
                }
            }}
        }
    }
}
struct Harness {
    dom: VirtualDom,
    capture: Rc<Capture>,
}
impl Harness {
    fn new(topology: Topology) -> Self {
        let capture = Rc::new(Capture {
            topology,
            ..Default::default()
        });
        let mut dom = VirtualDom::new_with_props(fixture, capture.clone());
        dom.rebuild_in_place();
        let mut result = Self { dom, capture };
        result.settle();
        result
    }
    fn settle(&mut self) {
        for _ in 0..5 {
            self.dom.render_immediate_to_vec();
            self.dom.process_events();
        }
    }
    fn update(&mut self, change: impl FnOnce(&mut Settings)) {
        self.dom
            .in_runtime(|| self.capture.settings.get().unwrap().with_mut(change));
        self.settle();
    }
    fn view(&self, name: &str) -> Snapshot {
        self.capture.views.borrow()[name].clone()
    }
    fn facts(&mut self, width: f64, height: f64, media: DocumentMediaFacts) {
        let facts = DocumentViewportFacts {
            width,
            height,
            visual_left: 0.0,
            visual_top: 0.0,
            visual_width: width,
            visual_height: height,
            media: Some(media),
        };
        self.dom.in_runtime(|| {
            self.capture
                .document
                .borrow()
                .as_ref()
                .unwrap()
                .test_facts(facts)
        });
        self.settle();
    }
    fn same_layout_policy(&self, prefix: &str) -> Rc<RefCell<ViewportModel>> {
        let owner = self.view(&format!("{prefix}-owner")).viewport.model;
        for side in ["left", "right", "top", "bottom"] {
            assert!(
                Rc::ptr_eq(
                    &owner,
                    &self.view(&format!("{prefix}-{side}")).viewport.model
                ),
                "{prefix}-{side} created another responsive policy"
            );
        }
        owner
    }
}

#[test]
fn four_actual_panes_share_the_layout_policy_and_refresh_from_shared_document_facts() {
    let mut h = Harness::new(Topology::Single);
    assert_eq!(dioxus_ssr::render(&h.dom).matches("<aside ").count(), 4);
    let owner = h.same_layout_policy("solo");
    assert_eq!(h.view("solo-owner").breakpoint, Large);
    assert!(!h.view("solo-owner").ready);
    h.facts(
        360.0,
        800.0,
        DocumentMediaFacts {
            sm: true,
            md: false,
            lg: false,
        },
    );
    for name in [
        "solo-owner",
        "solo-left",
        "solo-right",
        "solo-top",
        "solo-bottom",
    ] {
        let view = h.view(name);
        assert_eq!(view.breakpoint, Small);
        assert!(view.ready);
        assert_eq!(view.size, Some((360.0, 800.0)));
        assert!(Rc::ptr_eq(&owner, &view.viewport.model));
    }
    let html = dioxus_ssr::render(&h.dom);
    assert_eq!(
        html.matches("data-mobile=\"true\"").count(),
        4,
        "actual Pane responsive state follows the shared owner"
    );
}

#[test]
fn ancestor_force_updates_reach_nested_layouts_while_nearest_and_sibling_owners_stay_isolated() {
    let mut h = Harness::new(Topology::Ancestor);
    let outer = h.same_layout_policy("outer");
    let nested = h.same_layout_policy("nested");
    let nearest = h.same_layout_policy("override");
    assert!(Rc::ptr_eq(&outer, &nested));
    assert!(Rc::ptr_eq(&outer, &h.view("ancestor").viewport.model));
    assert!(Rc::ptr_eq(&nearest, &h.view("nearest").viewport.model));
    assert!(!Rc::ptr_eq(&outer, &nearest));
    h.update(|settings| settings.force = Medium);
    for name in ["outer-owner", "outer-left", "nested-owner", "nested-bottom"] {
        assert_eq!(h.view(name).breakpoint, Medium);
    }
    assert_eq!(h.view("override-left").breakpoint, Small);
    h.update(|settings| settings.force = Small);
    assert_eq!(h.view("nested-top").breakpoint, Small);
    assert!(Rc::ptr_eq(&outer, &h.view("nested-top").viewport.model));
    drop(h);
    assert!(!outer.borrow().alive);
    assert!(!nearest.borrow().alive);

    let mut siblings = Harness::new(Topology::Siblings);
    let a = siblings.same_layout_policy("a");
    let b = siblings.same_layout_policy("b");
    assert!(!Rc::ptr_eq(&a, &b));
    siblings.facts(
        1200.0,
        780.0,
        DocumentMediaFacts {
            sm: false,
            md: true,
            lg: false,
        },
    );
    assert_eq!(siblings.view("a-left").breakpoint, Medium);
    assert_eq!(siblings.view("b-right").breakpoint, Medium);
    assert!(a.borrow().alive && b.borrow().alive);
    drop(siblings);
    assert!(!a.borrow().alive && !b.borrow().alive);
}

#[test]
fn child_removal_preserves_policy_but_layout_drop_releases_it_and_remount_is_fresh() {
    let mut h = Harness::new(Topology::Single);
    let owner = h.same_layout_policy("solo");
    let weak = Rc::downgrade(&owner);
    h.update(|settings| settings.left = false);
    assert!(owner.borrow().alive);
    assert_eq!(dioxus_ssr::render(&h.dom).matches("<aside ").count(), 3);
    h.facts(
        360.0,
        800.0,
        DocumentMediaFacts {
            sm: true,
            md: false,
            lg: false,
        },
    );
    assert_eq!(h.view("solo-right").breakpoint, Small);
    assert_eq!(h.view("solo-bottom").size, Some((360.0, 800.0)));
    assert!(Rc::ptr_eq(&owner, &h.view("solo-owner").viewport.model));
    h.update(|settings| settings.visible = false);
    assert!(!owner.borrow().alive);
    assert_eq!(owner.borrow().viewport_size, None);
    let retired_facts = owner.borrow().facts;
    h.facts(
        1280.0,
        900.0,
        DocumentMediaFacts {
            sm: false,
            md: false,
            lg: true,
        },
    );
    assert_eq!(
        owner.borrow().facts,
        retired_facts,
        "retired policy no longer consumes document updates"
    );
    h.capture.views.borrow_mut().clear();
    drop(owner);
    assert!(
        weak.upgrade().is_none(),
        "removed scopes must release their policy resources"
    );
    h.update(|settings| {
        settings.visible = true;
        settings.left = true;
    });
    let fresh = h.same_layout_policy("solo");
    assert!(fresh.borrow().alive);
    assert_eq!(h.view("solo-left").breakpoint, Large);
    assert_eq!(h.view("solo-left").size, Some((1280.0, 900.0)));
    assert_eq!(dioxus_ssr::render(&h.dom).matches("<aside ").count(), 4);
    drop(h);
    assert!(!fresh.borrow().alive);
}
