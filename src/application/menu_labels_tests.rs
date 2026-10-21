use super::*;

fn translated() -> MenuLabels {
    MenuLabels {
        expand: "Utvid".into(),
        collapse: "Skjul".into(),
        settings: "Innstillinger".into(),
        about: "Om og juridisk informasjon".into(),
        toggle_theme: "Bytt tema".into(),
        collapse_navigation: "Skjul navigasjon".into(),
    }
}
#[derive(Default)]
struct LabelsCapture {
    labels: Cell<Option<Signal<MenuLabels>>>,
    expanded: Cell<Option<Signal<bool>>>,
    overlay: Cell<Option<Signal<Option<MenuOverlayKind>>>>,
    veto_settings: Cell<bool>,
    veto_expansion: Cell<bool>,
    settings_calls: Cell<usize>,
    about_calls: Cell<usize>,
    theme_calls: Cell<usize>,
    mounts: RefCell<BTreeMap<String, usize>>,
    drops: RefCell<BTreeMap<String, usize>>,
}
impl PartialEq for LabelsCapture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn LabelDraft(capture: Rc<LabelsCapture>, id: String) -> Element {
    use_hook({
        let capture = capture.clone();
        let id = id.clone();
        move || *capture.mounts.borrow_mut().entry(id).or_default() += 1
    });
    use_drop({
        let id = id.clone();
        move || *capture.drops.borrow_mut().entry(id).or_default() += 1
    });
    let mut draft = use_signal(|| String::from("Initial private note"));
    rsx! {input {id,value:draft(),oninput:move|event|draft.set(event.value())}}
}
fn labels_fixture(capture: Rc<LabelsCapture>) -> Element {
    let labels = use_signal(MenuLabels::default);
    let expanded = use_signal(|| false);
    let overlay = use_signal(|| None);
    capture.labels.set(Some(labels));
    capture.expanded.set(Some(expanded));
    capture.overlay.set(Some(overlay));
    let settings = capture.clone();
    let about = capture.clone();
    let theme = capture.clone();
    let opening = capture.clone();
    let expansion = capture.clone();
    rsx! {Application {force_breakpoint:ApplicationBreakpoint::Small,menu_expanded:expanded,
        header:rsx!{ApplicationHeader {name:"Menu label fixture",menu_labels:labels()}},
        menu:rsx!{Menu {labels:labels(),active_overlay:overlay,enable_toggle_theme:true,
            onbeforeopensettings:move|event:MenuOpenEvent|if opening.veto_settings.get(){event.prevent_default();},
            onbeforeexpandchange:move|event:MenuExpandEvent|if expansion.veto_expansion.get(){event.prevent_default();},
            onopensettings:move |_|settings.settings_calls.set(settings.settings_calls.get()+1),
            onopenabout:move |_|about.about_calls.set(about.about_calls.get()+1),
            ontoggletheme:move |_|theme.theme_calls.set(theme.theme_calls.get()+1),
            settings:rsx!{MenuSettings {label:"Independent settings heading",aria_label_close_button:"Close independent settings",LabelDraft {capture:capture.clone(),id:"settings-label-note"}}},
            about:rsx!{MenuAbout {label:"Independent about heading",aria_label_close_button:"Close independent about","About content"}},
            MenuItem {label:"Primary navigation",close_on_select:false}
            LabelDraft {capture:capture.clone(),id:"menu-label-note"}
        }},
        "Application content"
    }}
}
struct LabelsHarness {
    dom: VirtualDom,
    capture: Rc<LabelsCapture>,
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    pending: Vec<dioxus::core::ElementId>,
    mounted: Vec<dioxus::core::ElementId>,
    focuses: Rc<RefCell<Vec<usize>>>,
}
impl LabelsHarness {
    fn new() -> Self {
        dioxus::html::set_event_converter(Box::new(MenuTestConverter));
        let capture = Rc::new(LabelsCapture::default());
        let mut dom = VirtualDom::new_with_props(labels_fixture, capture.clone());
        let edits = dom.rebuild_to_vec();
        let mut result = Self {
            dom,
            capture,
            attrs: BTreeMap::new(),
            pending: Vec::new(),
            mounted: Vec::new(),
            focuses: Default::default(),
        };
        result.collect(edits);
        result.settle();
        result
    }
    fn collect(&mut self, edits: dioxus::core::Mutations) {
        use dioxus::core::Mutation;
        let mut current = None;
        for edit in edits.edits {
            match edit {
                Mutation::SetId { id } => {
                    self.attrs.insert(id.raw(), BTreeMap::new());
                    current = Some(id);
                }
                Mutation::PushId { id } => current = Some(id),
                Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } => {
                    if let Some(id) = current {
                        self.attrs.entry(id.raw()).or_default().insert(name, value);
                    }
                }
                Mutation::NewEventListener { name } if name == "mounted" => self
                    .pending
                    .push(current.expect("registered native control")),
                Mutation::SetAttribute { .. }
                | Mutation::SetText { .. }
                | Mutation::NewEventListener { .. }
                | Mutation::RemoveEventListener { .. } => {}
                _ => current = None,
            }
        }
    }
    fn settle(&mut self) {
        for _ in 0..8 {
            let edits = self.dom.render_immediate_to_vec();
            self.collect(edits);
            for id in std::mem::take(&mut self.pending) {
                let mount = MenuTestMount {
                    id: id.raw(),
                    focuses: self.focuses.clone(),
                    ..Default::default()
                };
                mount.ready.set(true);
                menu_dispatch(&self.dom, "mounted", id, mount);
                self.mounted.push(id);
            }
            self.dom.process_events();
        }
    }
    fn target(&self, name: &str, value: &str) -> dioxus::core::ElementId {
        dioxus::core::ElementId::from_raw(
            *self
                .attrs
                .iter()
                .find(|(_, attrs)| attrs.get(name).is_some_and(|actual| actual == value))
                .unwrap_or_else(|| panic!("missing {name}={value}"))
                .0,
        )
    }
    fn marker(&self, name: &str) -> dioxus::core::ElementId {
        dioxus::core::ElementId::from_raw(
            *self
                .attrs
                .iter()
                .find(|(_, attrs)| attrs.contains_key(name))
                .unwrap_or_else(|| panic!("missing {name}"))
                .0,
        )
    }
    fn click(&mut self, id: dioxus::core::ElementId) {
        menu_dispatch(&self.dom, "click", id, ());
        self.settle();
    }
    fn named_click(&mut self, label: &str) {
        self.click(self.target("aria-label", label));
    }
    fn text(&mut self, id: &str, value: &str) {
        menu_dispatch(&self.dom, "input", self.target("id", id), value.to_string());
        self.settle();
    }
    fn expanded(&self) -> bool {
        *self.capture.expanded.get().unwrap().peek()
    }
    fn overlay(&self) -> Option<MenuOverlayKind> {
        *self.capture.overlay.get().unwrap().peek()
    }
}
#[test]
fn menu_control_label_defaults_keep_original_native_actions_and_vetoes() {
    let _guard = crate::test_event_converter_guard();
    let mut h = LabelsHarness::new();
    let own = h.marker("data-menu-expansion-control");
    let header = h.marker("data-header-menu-toggle");
    assert_eq!(h.attrs[&own.raw()]["aria-label"], "Expand");
    assert_eq!(h.attrs[&header.raw()]["aria-label"], "Expand");
    h.click(own);
    assert!(h.expanded());
    assert_eq!(h.attrs[&own.raw()]["aria-label"], "Collapse");
    assert_eq!(h.attrs[&header.raw()]["aria-label"], "Collapse");
    assert!(dioxus_ssr::render(&h.dom).contains("class=\"ix-menu-label\">Collapse</span>"));
    for label in [
        "Settings",
        "About & legal information",
        "Toggle theme",
        "Collapse navigation",
    ] {
        h.target("aria-label", label);
    }
    h.named_click("Toggle theme");
    assert_eq!(h.capture.theme_calls.get(), 1);
    h.capture.veto_settings.set(true);
    h.named_click("Settings");
    assert_eq!(h.overlay(), None);
    assert_eq!(h.capture.settings_calls.get(), 0);
    h.capture.veto_settings.set(false);
    h.named_click("Settings");
    assert_eq!(h.overlay(), Some(MenuOverlayKind::Settings));
    assert_eq!(h.capture.settings_calls.get(), 1);
}
#[test]
fn live_menu_labels_update_current_controls_without_replacing_open_overlay_or_child_drafts() {
    let _guard = crate::test_event_converter_guard();
    let mut h = LabelsHarness::new();
    let own = h.marker("data-menu-expansion-control");
    let header = h.marker("data-header-menu-toggle");
    let settings = h.target("aria-label", "Settings");
    let about = h.target("aria-label", "About & legal information");
    let theme = h.target("aria-label", "Toggle theme");
    h.click(own);
    h.text("menu-label-note", "Retained navigation note");
    h.named_click("Settings");
    h.text("settings-label-note", "Retained settings note");
    let menu_note = h.target("id", "menu-label-note");
    let settings_note = h.target("id", "settings-label-note");
    let mounted = h.mounted.len();
    h.dom
        .in_runtime(|| h.capture.labels.get().unwrap().set(translated()));
    h.settle();
    assert!(h.expanded());
    assert_eq!(h.overlay(), Some(MenuOverlayKind::Settings));
    assert_eq!(h.marker("data-menu-expansion-control"), own);
    assert_eq!(h.marker("data-header-menu-toggle"), header);
    assert_eq!(h.attrs[&own.raw()]["aria-label"], "Skjul");
    assert_eq!(h.attrs[&header.raw()]["aria-label"], "Skjul");
    assert_eq!(h.target("aria-label", "Innstillinger"), settings);
    assert_eq!(h.target("aria-label", "Om og juridisk informasjon"), about);
    assert_eq!(h.target("aria-label", "Bytt tema"), theme);
    assert_eq!(h.target("id", "menu-label-note"), menu_note);
    assert_eq!(h.target("id", "settings-label-note"), settings_note);
    assert_eq!(
        h.mounted.len(),
        mounted,
        "label changes do not replace native controls or open overlay content"
    );
    assert_eq!(h.capture.mounts.borrow().get("menu-label-note"), Some(&1));
    assert_eq!(
        h.capture.mounts.borrow().get("settings-label-note"),
        Some(&1)
    );
    assert!(h.capture.drops.borrow().is_empty());
    let html = dioxus_ssr::render(&h.dom);
    assert!(html.contains("Retained navigation note") && html.contains("Retained settings note"));
    assert!(
        html.contains("class=\"ix-menu-label\">Skjul</span>")
            && html.contains("Independent settings heading")
    );
    h.target("aria-label", "Close independent settings");
    h.target("aria-label", "Skjul navigasjon");
    h.named_click("Close independent settings");
    assert_eq!(h.overlay(), None);
    h.named_click("Om og juridisk informasjon");
    assert_eq!(h.capture.about_calls.get(), 1);
    assert_eq!(h.overlay(), Some(MenuOverlayKind::About));
    h.named_click("Close independent about");
    h.named_click("Bytt tema");
    assert_eq!(h.capture.theme_calls.get(), 1);
    h.capture.veto_expansion.set(true);
    h.click(header);
    assert!(h.expanded());
    h.capture.veto_expansion.set(false);
    h.click(header);
    assert!(!h.expanded());
    assert_eq!(h.attrs[&own.raw()]["aria-label"], "Utvid");
    assert_eq!(h.attrs[&header.raw()]["aria-label"], "Utvid");
    h.click(header);
    assert!(h.expanded());
    h.named_click("Skjul navigasjon");
    assert!(!h.expanded());
}
