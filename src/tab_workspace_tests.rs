// Actual VDOM close/forwarding tests; native key activation/focus/overflow are verified in the gallery probe.
use crate::{TabPanel, TabSet};

#[derive(Clone)]
struct WorkspaceConfig {
    items: Vec<TabItem>,
    manual: bool,
    rounded: bool,
    localized: bool,
    close_prefix: String,
    overflow: bool,
    more_label: String,
}
impl Default for WorkspaceConfig {
    fn default() -> Self {
        let mut a = TabItem::new("a", "Inspection");
        a.closable = true;
        let mut b = TabItem::new("b", "Reference");
        b.closable = true;
        b.disabled = true;
        Self {
            items: vec![a, b, TabItem::new("c", "Checklist")],
            manual: false,
            rounded: false,
            localized: false,
            close_prefix: "Schließen".into(),
            overflow: false,
            more_label: "Alle Dokumente".into(),
        }
    }
}
#[derive(Default)]
struct WorkspaceObservation {
    config: Cell<Option<Signal<WorkspaceConfig>>>,
    closes: RefCell<Vec<usize>>,
    key_closes: RefCell<Vec<String>>,
    selected: RefCell<Vec<Option<usize>>>,
    key_selected: RefCell<Vec<Option<String>>>,
    requests: Cell<usize>,
}
#[derive(Clone, Default)]
struct WorkspaceFixture(Rc<WorkspaceObservation>);
fn workspace_config(fixture: &WorkspaceFixture) -> WorkspaceConfig {
    let config = use_signal(WorkspaceConfig::default);
    fixture.0.config.set(Some(config));
    config()
}
fn close_formatter(config: &WorkspaceConfig) -> Option<Callback<TabItem, String>> {
    let prefix = config.close_prefix.clone();
    config
        .localized
        .then(|| Callback::new(move |item: TabItem| format!("{prefix} {}", item.label)))
}
fn strip_fixture(fixture: WorkspaceFixture) -> Element {
    let config = workspace_config(&fixture);
    let close = fixture.clone();
    let before = fixture.clone();
    rsx! {Tabs {
        close_label:close_formatter(&config),items:config.items,rounded:config.rounded,
        keyboard_navigation:if config.manual {TabKeyboardNavigation::Manual}else{TabKeyboardNavigation::Automatic},
        id_prefix:"workspace-test",onclose:move |index|close.0.closes.borrow_mut().push(index),
        onbeforechange:move |_|before.0.requests.set(before.0.requests.get()+1),
        onselectionchange:move |index|fixture.0.selected.borrow_mut().push(index),
    }}
}
#[component]
fn RetainedWorkspaceBody() -> Element {
    let mut count = use_signal(|| 0usize);
    rsx! {button {id:String::from("workspace-local"),type:"button",onclick:move |_|count+=1,"Local work {count}"}}
}
fn set_fixture(fixture: WorkspaceFixture) -> Element {
    let config = workspace_config(&fixture);
    let close = fixture.clone();
    let before = fixture.clone();
    rsx! {TabSet {
        close_label:close_formatter(&config),items:config.items.clone(),overflow_menu:config.overflow,aria_label_more_tabs:config.more_label,
        keyboard_navigation:if config.manual {TabKeyboardNavigation::Manual}else{TabKeyboardNavigation::Automatic},
        onclose:move |key|close.0.key_closes.borrow_mut().push(key),
        onbeforechange:move |_|before.0.requests.set(before.0.requests.get()+1),
        onselectionchange:move |key|fixture.0.key_selected.borrow_mut().push(key),
        for item in config.items {
            TabPanel {key:"{item.key}",tab_key:item.key.clone(),
                if item.key=="a" {RetainedWorkspaceBody {}}else{"{item.label} body"}
            }
        }
    }}
}
#[derive(Clone)]
struct WorkspaceKey(Key);
impl dioxus::html::ModifiersInteraction for WorkspaceKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for WorkspaceKey {
    fn key(&self) -> Key {
        self.0.clone()
    }
    fn code(&self) -> dioxus::html::Code {
        dioxus::html::Code::Unidentified
    }
    fn location(&self) -> dioxus::html::Location {
        dioxus::html::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct WorkspaceConverter;
macro_rules! workspace_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for WorkspaceConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;workspace_converter!(@data $group,event)})*
        }
    };
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(WorkflowMouse)};
    (@data Keyboard,$event:ident)=>{dioxus::html::KeyboardData::new($event.downcast::<WorkspaceKey>().unwrap().clone())};
    (@data $other:ident,$event:ident)=>{panic!("Unexpected tab workspace event")};
}
dioxus::html::with_html_event_groups!(workspace_converter);
#[derive(Default)]
struct WorkspaceTargets {
    ids: BTreeMap<String, dioxus::core::ElementId>,
    labels: BTreeMap<String, dioxus::core::ElementId>,
}
impl WorkspaceTargets {
    fn collect(&mut self, mutations: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } => {
                    if name == "id" {
                        self.ids.insert(value, current);
                    } else if name == "aria-label" {
                        self.labels.insert(value, current);
                    }
                }
                _ => {}
            }
        }
    }
    fn tab(&self, key: &str) -> dioxus::core::ElementId {
        *self
            .ids
            .iter()
            .find(|(id, _)| id.ends_with(&format!("-tab-{}", crate::composition::tab_key_id(key))))
            .unwrap()
            .1
    }
}
fn workspace_setup(fixture: WorkspaceFixture, keyed: bool) -> (VirtualDom, WorkspaceTargets) {
    let app: fn(WorkspaceFixture) -> Element = if keyed { set_fixture } else { strip_fixture };
    let mut dom = VirtualDom::new_with_props(app, fixture);
    let mut targets = WorkspaceTargets::default();
    targets.collect(dom.rebuild_to_vec());
    workspace_settle(&mut dom, &mut targets);
    (dom, targets)
}
fn workspace_settle(dom: &mut VirtualDom, targets: &mut WorkspaceTargets) {
    for _ in 0..4 {
        targets.collect(dom.render_immediate_to_vec());
        dom.process_events();
    }
}
fn workspace_dispatch(
    dom: &VirtualDom,
    event: &str,
    id: dioxus::core::ElementId,
    key: Option<Key>,
) -> bool {
    let payload: Box<dyn std::any::Any> = key.map_or_else(
        || Box::new(()) as Box<dyn std::any::Any>,
        |key| Box::new(WorkspaceKey(key)),
    );
    let event_data = Event::new(
        Rc::new(dioxus::html::PlatformEventData::new(payload)) as Rc<dyn std::any::Any>,
        true,
    );
    dom.runtime().handle_event(event, event_data.clone(), id);
    event_data.default_action_enabled()
}

#[test]
fn tab_workspace_delete_requests_close_without_mutating_selection_or_consumer_items() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(WorkspaceConverter));
    let fixture = WorkspaceFixture::default();
    let (mut dom, mut targets) = workspace_setup(fixture.clone(), false);
    assert!(!workspace_dispatch(
        &dom,
        "keydown",
        targets.tab("a"),
        Some(Key::Delete)
    ));
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(*fixture.0.closes.borrow(), [0]);
    assert!(fixture.0.selected.borrow().is_empty());
    assert_eq!(fixture.0.requests.get(), 0);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("role=\"tab\"").count(), 3);
    assert_eq!(
        html.matches("aria-selected=\"true\"").count() + html.matches("aria-selected=true").count(),
        1
    );
    assert!(
        workspace_dispatch(&dom, "keydown", targets.tab("b"), Some(Key::Delete)),
        "disabled synthetic delivery must not close"
    );
    assert!(
        workspace_dispatch(&dom, "keydown", targets.tab("c"), Some(Key::Delete)),
        "nonclosable tabs retain the key default"
    );
    workspace_dispatch(&dom, "click", targets.labels["Close Reference"], None);
    assert_eq!(
        *fixture.0.closes.borrow(),
        [0],
        "disabled closebutton also guards synthetic dispatch"
    );
}

#[test]
fn tab_workspace_native_activation_defaults_and_disabled_navigation_stay_separate_from_close() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(WorkspaceConverter));
    let fixture = WorkspaceFixture::default();
    let (mut dom, mut targets) = workspace_setup(fixture.clone(), false);
    // A native button owns Enter/Space activation; VDOM key delivery must not fabricate its click.
    for key in [Key::Enter, Key::Character(" ".into())] {
        assert!(workspace_dispatch(
            &dom,
            "keydown",
            targets.tab("a"),
            Some(key)
        ));
    }
    assert!(fixture.0.selected.borrow().is_empty());
    assert!(!workspace_dispatch(
        &dom,
        "keydown",
        targets.tab("a"),
        Some(Key::ArrowRight)
    ));
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(*fixture.0.selected.borrow(), [Some(2)]);
    let mut config = fixture.0.config.get().unwrap();
    dom.in_runtime(|| config.write().manual = true);
    workspace_settle(&mut dom, &mut targets);
    assert!(!workspace_dispatch(
        &dom,
        "keydown",
        targets.tab("c"),
        Some(Key::ArrowLeft)
    ));
    assert_eq!(
        *fixture.0.selected.borrow(),
        [Some(2)],
        "manual arrows do not activate"
    );
    workspace_dispatch(&dom, "click", targets.tab("a"), None);
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(*fixture.0.selected.borrow(), [Some(2), Some(0)]);
    assert!(fixture.0.closes.borrow().is_empty());
}

#[test]
fn tab_workspace_localized_close_label_tracks_live_items_and_rounded_delete_still_requests() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(WorkspaceConverter));
    let fixture = WorkspaceFixture::default();
    let (mut dom, mut targets) = workspace_setup(fixture.clone(), false);
    assert!(dioxus_ssr::render(&dom).contains("aria-label=\"Close Inspection\""));
    let mut config = fixture.0.config.get().unwrap();
    dom.in_runtime(|| {
        let mut config = config.write();
        config.localized = true;
        config.items[0].label = "Motorprüfung".into();
    });
    workspace_settle(&mut dom, &mut targets);
    assert!(dioxus_ssr::render(&dom).contains("aria-label=\"Schließen Motorprüfung\""));
    workspace_dispatch(
        &dom,
        "click",
        targets.labels["Schließen Motorprüfung"],
        None,
    );
    assert_eq!(*fixture.0.closes.borrow(), [0]);
    dom.in_runtime(|| {
        let mut config = config.write();
        config.rounded = true;
        config.close_prefix = "Fermer".into();
    });
    workspace_settle(&mut dom, &mut targets);
    assert!(!dioxus_ssr::render(&dom).contains("ix-tab-close"));
    workspace_dispatch(&dom, "keydown", targets.tab("a"), Some(Key::Delete));
    assert_eq!(*fixture.0.closes.borrow(), [0, 0]);
    dom.in_runtime(|| config.write().items[0].disabled = true);
    workspace_settle(&mut dom, &mut targets);
    workspace_dispatch(&dom, "keydown", targets.tab("a"), Some(Key::Delete));
    assert_eq!(*fixture.0.closes.borrow(), [0, 0]);
}

#[test]
fn tab_workspace_keyed_set_forwards_overflow_labels_and_close_requests_while_panels_stay_retained()
{
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(WorkspaceConverter));
    let fixture = WorkspaceFixture::default();
    let (mut dom, mut targets) = workspace_setup(fixture.clone(), true);
    let mut config = fixture.0.config.get().unwrap();
    dom.in_runtime(|| {
        let mut config = config.write();
        config.overflow = true;
        config.localized = true;
    });
    workspace_settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("ix-tabs-overflow"));
    assert!(html.contains("aria-label=\"Alle Dokumente\""));
    assert!(html.contains("aria-label=\"Schließen Inspection\""));
    workspace_dispatch(&dom, "click", targets.ids["workspace-local"], None);
    workspace_settle(&mut dom, &mut targets);
    workspace_dispatch(&dom, "click", targets.tab("c"), None);
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(*fixture.0.key_selected.borrow(), [Some("c".into())]);
    assert!(dioxus_ssr::render(&dom).contains("Local work 1"));
    workspace_dispatch(&dom, "keydown", targets.tab("a"), Some(Key::Delete));
    assert_eq!(*fixture.0.key_closes.borrow(), ["a"]);
    assert_eq!(*fixture.0.key_selected.borrow(), [Some("c".into())]);
    dom.in_runtime(|| {
        let mut config = config.write();
        config.items.reverse();
        config.more_label = "Tous les documents".into();
    });
    workspace_settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("aria-label=\"Tous les documents\""));
    assert!(html.contains("Local work 1"));
    workspace_dispatch(&dom, "click", targets.labels["Schließen Inspection"], None);
    assert_eq!(
        *fixture.0.key_closes.borrow(),
        ["a", "a"],
        "close index translates using current order"
    );
}
