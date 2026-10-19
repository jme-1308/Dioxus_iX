// Settings/About focus through actual Menu controls and registered mounted handles.
use super::*;

#[derive(Clone, Default)]
struct OverlayFocusCapture {
    active: Rc<Cell<Option<Signal<Option<MenuOverlayKind>>>>>,
    visible: Rc<Cell<Option<Signal<bool>>>>,
    context: Rc<Cell<Option<MenuOverlayContext>>>,
    veto: Rc<Cell<bool>>,
}
impl PartialEq for OverlayFocusCapture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.active, &other.active)
    }
}
#[component]
fn OverlayFocusProbe(capture: OverlayFocusCapture) -> Element {
    capture
        .context
        .set(Some(consume_context::<MenuOverlayContext>()));
    rsx! { input { id: "overlay-focus-field" } }
}
fn overlay_focus_fixture(capture: OverlayFocusCapture) -> Element {
    let active = use_signal(|| None);
    let visible = use_signal(|| true);
    capture.active.set(Some(active));
    capture.visible.set(Some(visible));
    let settings = capture.clone();
    let about = capture.clone();
    let veto = capture.veto.clone();
    rsx! { if visible() {
        Menu { start_expanded: true, pinned: true, active_overlay: active,
            onbeforeopensettings: move |event: MenuOpenEvent| if veto.get() { event.prevent_default(); },
            settings: rsx! { MenuSettings { OverlayFocusProbe { capture: settings } } },
            about: rsx! { MenuAbout { OverlayFocusProbe { capture: about } } },
            MenuItem { label: "Navigation" }
        }
    } }
}
struct OverlayFocusHarness {
    dom: VirtualDom,
    capture: OverlayFocusCapture,
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    focuses: Rc<RefCell<Vec<usize>>>,
}
impl OverlayFocusHarness {
    fn new() -> Self {
        dioxus::html::set_event_converter(Box::new(MenuTestConverter));
        let capture = OverlayFocusCapture::default();
        let mut dom = VirtualDom::new_with_props(overlay_focus_fixture, capture.clone());
        let mutations = dom.rebuild_to_vec();
        let mut h = Self {
            dom,
            capture,
            attrs: BTreeMap::new(),
            focuses: Default::default(),
        };
        h.record(mutations);
        h.settle();
        for label in ["Settings", "About & legal information"] {
            let target = h.target(label);
            let id = h.attrs[&target.raw()]["data-menu-control"].parse().unwrap();
            h.mount(label, id);
        }
        h
    }
    fn record(&mut self, mutations: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for mutation in mutations.edits {
            match mutation {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } => {
                    self.attrs
                        .entry(current.raw())
                        .or_default()
                        .insert(name, value);
                }
                _ => {}
            }
        }
    }
    fn settle(&mut self) {
        for _ in 0..5 {
            let mutations = self.dom.render_immediate_to_vec();
            self.record(mutations);
        }
    }
    fn target(&self, label: &str) -> dioxus::core::ElementId {
        let html = dioxus_ssr::render(&self.dom);
        let escaped_label = label.replace('&', "&#38;");
        let tag = html
            .split('>')
            .find(|tag| tag.contains(&format!(" aria-label=\"{escaped_label}\"")))
            .unwrap_or_else(|| panic!("Missing {label} in rendered fixture: {html}"));
        let id = tag
            .split(" id=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        self.attrs
            .iter()
            .find(|(_, attrs)| attrs.get("id").is_some_and(|value| value == id))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap()
    }
    fn mount(&mut self, label: &str, id: usize) {
        let mount = MenuTestMount {
            id,
            focuses: self.focuses.clone(),
            ..Default::default()
        };
        mount.ready.set(true);
        menu_dispatch(&self.dom, "mounted", self.target(label), mount);
        self.settle();
    }
    fn click(&mut self, label: &str) {
        menu_dispatch(&self.dom, "click", self.target(label), ());
        self.settle();
    }
    fn active(&mut self, next: Option<MenuOverlayKind>) {
        self.dom.in_scope(ScopeId::ROOT, || {
            self.capture.active.get().unwrap().set(next)
        });
        self.settle();
    }
    fn context(&self) -> MenuOverlayContext {
        self.capture.context.get().unwrap()
    }
    fn invoker(&self, label: &str) -> usize {
        self.attrs[&self.target(label).raw()]["data-menu-control"]
            .parse()
            .unwrap()
    }
}

#[test]
fn settings_about_autofocus_and_close_escape_return_to_registered_utility() {
    let _guard = crate::test_event_converter_guard();
    let mut h = OverlayFocusHarness::new();
    h.click("Settings");
    h.mount("Close Settings", 10_001);
    assert_eq!(h.focuses.borrow().last().copied(), Some(10_001));
    h.click("Close Settings");
    assert_eq!(
        h.focuses.borrow().last().copied(),
        Some(h.invoker("Settings"))
    );
    h.click("About & legal information");
    h.mount("Close About", 10_002);
    assert_eq!(h.focuses.borrow().last().copied(), Some(10_002));
    assert_eq!(
        menu_dispatch(&h.dom, "keydown", h.target("Close About"), Key::Escape),
        (true, false)
    );
    h.settle();
    assert_eq!(
        h.focuses.borrow().last().copied(),
        Some(h.invoker("About & legal information"))
    );
    assert_eq!(*h.capture.active.get().unwrap().peek(), None);
}

#[test]
fn direct_overlay_switch_keeps_invoker_and_external_close_preserves_outside_focus() {
    let _guard = crate::test_event_converter_guard();
    let mut h = OverlayFocusHarness::new();
    h.click("Settings");
    h.mount("Close Settings", 10_001);
    menu_dispatch(&h.dom, "focusin", h.target("Close Settings"), ());
    h.active(Some(MenuOverlayKind::About));
    h.mount("Close About", 10_002);
    h.click("Close About");
    assert_eq!(
        h.focuses.borrow().last().copied(),
        Some(h.invoker("Settings")),
        "programmatic switching retains the original registered utility"
    );
    h.click("Settings");
    h.mount("Close Settings", 10_003);
    menu_dispatch(&h.dom, "focusin", h.target("Close Settings"), ());
    menu_dispatch(&h.dom, "focusout", h.target("Close Settings"), ());
    let before = h.focuses.borrow().clone();
    h.active(None);
    assert_eq!(
        *h.focuses.borrow(),
        before,
        "external close after outside focus cannot restore the utility"
    );
}

#[test]
fn overlay_veto_and_replaced_invoker_do_not_receive_focus() {
    let _guard = crate::test_event_converter_guard();
    let mut h = OverlayFocusHarness::new();
    h.capture.veto.set(true);
    h.click("Settings");
    assert_eq!(*h.capture.active.get().unwrap().peek(), None);
    assert!(h.focuses.borrow().is_empty());
    h.capture.veto.set(false);
    h.click("Settings");
    h.mount("Close Settings", 10_001);
    menu_dispatch(&h.dom, "click", h.target("Close Settings"), ());
    let replacement = MenuTestMount {
        id: 20_001,
        focuses: h.focuses.clone(),
        ..Default::default()
    };
    replacement.ready.set(true);
    menu_dispatch(&h.dom, "mounted", h.target("Settings"), replacement);
    h.settle();
    assert_eq!(
        h.focuses.borrow().last().copied(),
        Some(10_001),
        "a replaced invoker invalidates the captured return"
    );
}

#[test]
fn queued_overlay_autofocus_and_return_cancel_on_new_visibility_or_owner_teardown() {
    let _guard = crate::test_event_converter_guard();
    let mut h = OverlayFocusHarness::new();
    h.click("Settings");
    let target = h.target("Close Settings");
    let mount = MenuTestMount {
        id: 10_001,
        focuses: h.focuses.clone(),
        ..Default::default()
    };
    mount.ready.set(true);
    menu_dispatch(&h.dom, "mounted", target, mount);
    h.dom
        .in_scope(ScopeId::ROOT, || h.capture.active.get().unwrap().set(None));
    h.settle();
    assert!(
        h.focuses.borrow().is_empty(),
        "closed overlay rejects queued autofocus"
    );
    h.click("Settings");
    h.mount("Close Settings", 10_002);
    let context = h.context();
    menu_dispatch(&h.dom, "click", h.target("Close Settings"), ());
    h.dom.in_scope(ScopeId::ROOT, || {
        h.capture.visible.get().unwrap().set(false)
    });
    h.settle();
    assert!(
        context.focus.try_peek().is_err()
            || context.focus.try_peek().is_ok_and(|state| !state.alive)
    );
    assert_eq!(h.focuses.borrow().last().copied(), Some(10_002));
}
