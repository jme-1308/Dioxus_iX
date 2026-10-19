use super::*;
struct TestMouse;
impl dioxus::html::InteractionLocation for TestMouse {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        Default::default()
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for TestMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for TestMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for TestMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for TestMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct TestForm(String);
impl dioxus::html::HasFileData for TestForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for TestForm {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        Vec::new()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct TestKeyboard(Key, Modifiers);
impl dioxus::html::ModifiersInteraction for TestKeyboard {
    fn modifiers(&self) -> Modifiers {
        self.1
    }
}
impl dioxus::html::HasKeyboardData for TestKeyboard {
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
struct TestFocus;
impl dioxus::html::HasFocusData for TestFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone)]
struct TestVisible {
    intersecting: bool,
    bounding: dioxus::html::geometry::PixelsRect,
    intersection: dioxus::html::geometry::PixelsRect,
}
impl TestVisible {
    fn new(intersecting: bool, clip: [f64; 4]) -> Self {
        use dioxus::html::geometry::{PixelsRect, PixelsSize, euclid::Point2D};
        let rect = |[top, bottom, left, right]: [f64; 4]| {
            PixelsRect::new(
                Point2D::new(left, top),
                PixelsSize::new(right - left, bottom - top),
            )
        };
        Self {
            intersecting,
            bounding: rect([0.0, 40.0, 0.0, 100.0]),
            intersection: rect(clip),
        }
    }
}
impl dioxus::html::HasVisibleData for TestVisible {
    fn get_bounding_client_rect(
        &self,
    ) -> dioxus::html::VisibleResult<dioxus::html::geometry::PixelsRect> {
        Ok(self.bounding)
    }
    fn get_intersection_rect(
        &self,
    ) -> dioxus::html::VisibleResult<dioxus::html::geometry::PixelsRect> {
        Ok(self.intersection)
    }
    fn get_root_bounds(&self) -> dioxus::html::VisibleResult<dioxus::html::geometry::PixelsRect> {
        Ok(self.bounding)
    }
    fn is_intersecting(&self) -> dioxus::html::VisibleResult<bool> {
        Ok(self.intersecting)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct Converter;
macro_rules! converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for Converter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*
        }
    };
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(TestMouse)};
    (@data Keyboard,$event:ident)=>{{let (key,modifiers)=$event.downcast::<(Key,Modifiers)>().unwrap();dioxus::html::KeyboardData::new(TestKeyboard(key.clone(),*modifiers))}};
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(TestForm($event.downcast::<String>().unwrap().clone()))};
    (@data Visible,$event:ident)=>{dioxus::html::VisibleData::new($event.downcast::<TestVisible>().unwrap().clone())};
    (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(TestFocus)};
    (@data Mounted,$event:ident)=>{if let Some(mock)=$event.downcast::<TriggerFocusMount>() {MountedData::new(mock.clone())}else{MountedData::new(())}};
    (@data $other:ident,$event:ident)=>{panic!("Unexpected Dropdown test event")};
}
dioxus::html::with_html_event_groups!(converter);
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    payload: impl std::any::Any,
) -> (bool, bool) {
    let data: Rc<dyn std::any::Any> =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload)));
    let event = Event::new(
        data,
        !matches!(name, "focus" | "blur" | "mounted" | "visible"),
    );
    dom.runtime().handle_event(name, event.clone(), id);
    (!event.default_action_enabled(), event.propagates())
}
#[derive(Clone)]
struct Fixture {
    context: Rc<RefCell<Option<DropdownContext>>>,
    linked: Rc<Cell<Option<DropdownHandle>>>,
    update: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    visible: bool,
    mode: DropdownNavigationMode,
    layer: bool,
    blocked: bool,
    loading: bool,
    linked_blocked: bool,
    focus_checked: bool,
    no_focus: bool,
    no_trap: bool,
    alpha_id: Option<String>,
    placement: DropdownPlacement,
    suppress_visibility: bool,
    suppress_automatic: bool,
    linked_veto: bool,
    deep_veto: bool,
    veto_open: bool,
    veto_close: bool,
    close_after_open: bool,
    requests: Vec<(bool, bool)>,
    changed: Vec<bool>,
    listener_revision: usize,
    deliveries: Vec<(usize, bool, bool)>,
    item_requests: usize,
    item_veto: bool,
    item_keep_open: bool,
    item_prevent_click: bool,
    close_behavior: DropdownCloseBehavior,
    standalone_requests: usize,
    standalone_mouse: usize,
    item_focus_events: usize,
    item_blur_events: usize,
    pending_drop: Option<Rc<Cell<bool>>>,
    pending_started: Rc<Cell<bool>>,
    native_clicks: usize,
    mounts: usize,
    drops: usize,
}
impl Default for Fixture {
    fn default() -> Self {
        Self {
            context: Rc::new(RefCell::new(None)),
            linked: Rc::new(Cell::new(None)),
            update: None,
            visible: true,
            mode: DropdownNavigationMode::default(),
            layer: false,
            blocked: false,
            loading: false,
            linked_blocked: false,
            focus_checked: false,
            no_focus: false,
            no_trap: false,
            alpha_id: None,
            placement: DropdownPlacement::default(),
            suppress_visibility: false,
            suppress_automatic: false,
            linked_veto: false,
            deep_veto: false,
            veto_open: false,
            veto_close: false,
            close_after_open: false,
            requests: vec![],
            changed: vec![],
            listener_revision: 0,
            deliveries: vec![],
            item_requests: 0,
            item_veto: false,
            item_keep_open: false,
            item_prevent_click: false,
            close_behavior: DropdownCloseBehavior::Both,
            standalone_requests: 0,
            standalone_mouse: 0,
            item_focus_events: 0,
            item_blur_events: 0,
            pending_drop: None,
            pending_started: Rc::new(Cell::new(false)),
            native_clicks: 0,
            mounts: 0,
            drops: 0,
        }
    }
}
impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.context, &other.context)
    }
}
struct PendingRead(Rc<Cell<bool>>);
impl Drop for PendingRead {
    fn drop(&mut self) {
        self.0.set(true);
    }
}
#[component]
fn Stateful(config: Rc<RefCell<Fixture>>) -> Element {
    let context = use_context::<DropdownContext>();
    *config.borrow().context.borrow_mut() = Some(context.clone());
    use_hook(|| config.borrow_mut().mounts += 1);
    use_hook(|| {
        if let Some(flag) = config.borrow().pending_drop.clone() {
            let started = config.borrow().pending_started.clone();
            let task = spawn(async move {
                let _read = PendingRead(flag);
                started.set(true);
                std::future::pending::<()>().await;
            });
            *context.focus_task.borrow_mut() = Some(task);
        }
    });
    let dropped = config.clone();
    use_drop(move || dropped.borrow_mut().drops += 1);
    let mut count = use_signal(|| 0_usize);
    // This small fixture indexes dynamic anchor edits, not static template
    // prototypes. Emit the input marker with its own assigned listener ID.
    let field_label = String::from("Editing field");
    rsx! {div{input {value:"uncontrolled initial","data-ix-dropdown-label":field_label,onkeydown:move |_|{}}button{r#type:"button","data-count-button":true.to_string(),onclick:move |_|count+=1,"Local {count}"}}}
}
fn fixture(config: Rc<RefCell<Fixture>>) -> Element {
    let settings = config.borrow().clone();
    config.borrow_mut().update = Some(dioxus::core::schedule_update());
    let linked = use_dropdown();
    settings.linked.set(Some(linked));
    let before = config.clone();
    let changed = config.clone();
    let item = config.clone();
    let click = config.clone();
    let focused = config.clone();
    let blurred = config.clone();
    let standalone_item = config.clone();
    let standalone_mouse = config.clone();
    let linked_before = config.clone();
    let deep_before = config.clone();
    rsx! {div {
        if settings.visible {
        Dropdown {label:"Menu",close_behavior:settings.close_behavior,navigation_mode:settings.mode,enable_top_layer:settings.layer,disabled:settings.blocked,loading:settings.loading,focus_checked_item:settings.focus_checked,disable_focus_handling:settings.no_focus,disable_focus_trap:settings.no_trap,placement:settings.placement,suppress_trigger_visibility_check:settings.suppress_visibility,suppress_automatic_placement:settings.suppress_automatic,
            onbeforechange:move |event:DropdownChangeEvent|{let veto={let mut state=before.borrow_mut();state.requests.push((event.open,event.previous));if event.open{state.veto_open}else{state.veto_close}};if veto{event.prevent_default();}},
            onchange:move |open:bool|{let (close,capture)={let mut state=changed.borrow_mut();state.changed.push(open);(state.close_after_open&&open,state.context.clone())};let context=capture.borrow().clone();if let Some(context)=context{changed.borrow_mut().deliveries.push((settings.listener_revision,open,*context.open.peek()));if close{context.request(false,Reason::Trigger,false);}}},
            Stateful {config:config.clone()}
            DropdownItem {label:"Alpha",id:settings.alpha_id.clone(),keep_open:settings.item_keep_open,onfocus:move |_|focused.borrow_mut().item_focus_events+=1,onblur:move |_|blurred.borrow_mut().item_blur_events+=1,onitemclick:move |event:DropdownItemClickEvent|{let veto={let mut state=item.borrow_mut();state.item_requests+=1;state.item_veto};if veto{event.prevent_default();}},onclick:move |event:MouseEvent|{let mut state=click.borrow_mut();state.native_clicks+=1;if state.item_prevent_click{event.prevent_default();}}}
            DropdownItem {label:"Disabled",disabled:true}
            DropdownItem {label:"Linked",dropdown:linked,checked:true}
            DropdownSubmenu {label:"Explicit",DropdownItem {label:"Persistent nested"}}
        }
        DropdownTrigger {dropdown:linked,aria_label:"External trigger","External trigger"}
        Dropdown {label:"Detached",controller:linked,disabled:settings.linked_blocked,focus_checked_item:true,onbeforechange:move |event:DropdownChangeEvent|{if event.open&&linked_before.borrow().linked_veto{event.prevent_default();}},DropdownItem {label:"Detached leaf"} DropdownItem {label:"Checked detached",checked:true} DropdownSubmenu {label:"Deep",onbeforechange:move |event:DropdownChangeEvent|{if event.open&&deep_before.borrow().deep_veto{event.prevent_default();}},DropdownItem {label:"Deep first"} DropdownItem {label:"Deep last"}}}
        DropdownItem {label:"Standalone",onitemclick:move |event:DropdownItemClickEvent|{let veto={let mut state=standalone_item.borrow_mut();state.standalone_requests+=1;state.item_veto};if veto{event.prevent_default();}},onclick:move |_|standalone_mouse.borrow_mut().standalone_mouse+=1}
        }
    }}
}
struct Harness {
    dom: VirtualDom,
    config: Rc<RefCell<Fixture>>,
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    listeners: BTreeMap<String, Vec<dioxus::core::ElementId>>,
}
impl Harness {
    fn new() -> Self {
        Self::with_settings(Fixture::default())
    }
    fn with_settings(settings: Fixture) -> Self {
        Self::with_document(settings, None)
    }
    fn with_document(
        settings: Fixture,
        document: Option<Rc<dyn dioxus::document::Document>>,
    ) -> Self {
        dioxus::html::set_event_converter(Box::new(Converter));
        let config = Rc::new(RefCell::new(settings));
        let mut dom = VirtualDom::new_with_props(fixture, config.clone());
        if let Some(document) = document {
            dom = dom.with_root_context(document);
        }
        let edits = dom.rebuild_to_vec();
        let mut result = Self {
            dom,
            config,
            attrs: BTreeMap::new(),
            listeners: BTreeMap::new(),
        };
        result.collect(edits);
        let mounted = result.listeners.get("mounted").cloned().unwrap_or_default();
        for id in mounted {
            dispatch(&result.dom, "mounted", id, ());
        }
        result.settle();
        result
    }
    fn collect(&mut self, mutations: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute { name, value, .. } => {
                    let text = match value {
                        dioxus::core::AttributeValue::Text(value) => value,
                        dioxus::core::AttributeValue::Bool(value) => value.to_string(),
                        dioxus::core::AttributeValue::Int(value) => value.to_string(),
                        dioxus::core::AttributeValue::None => {
                            if let Some(attrs) = self.attrs.get_mut(&current.raw()) {
                                attrs.remove(&name);
                            }
                            continue;
                        }
                        _ => continue,
                    };
                    self.attrs
                        .entry(current.raw())
                        .or_default()
                        .insert(name, text);
                }
                dioxus::core::Mutation::NewEventListener { name } => {
                    self.listeners.entry(name).or_default().push(current)
                }
                _ => {}
            }
        }
    }
    fn render_now(&mut self) {
        for _ in 0..4 {
            let edits = self.dom.render_immediate_to_vec();
            self.collect(edits);
        }
    }
    /// Run the real framework effects after rendering component updates.
    fn completion_effects(&mut self) {
        self.render_now();
    }
    fn settle(&mut self) {
        for _ in 0..4 {
            self.render_now();
            self.completion_effects();
        }
    }
    fn update(&mut self) {
        let update = self.config.borrow().update.clone().unwrap();
        update();
        self.settle();
    }
    fn context(&self) -> DropdownContext {
        self.config
            .borrow()
            .context
            .borrow()
            .as_ref()
            .unwrap()
            .clone()
    }
    fn target(&self, label: &str) -> dioxus::core::ElementId {
        let (id, _) = self
            .attrs
            .iter()
            .find(|(_, attrs)| {
                attrs.get("aria-label").is_some_and(|value| value == label)
                    && (attrs.get("data-ix-overlay-root").is_some()
                        || attrs.get("data-ix-overlay-trigger").is_some())
                    || attrs
                        .get("data-ix-dropdown-label")
                        .is_some_and(|value| value == label)
            })
            .unwrap_or_else(|| panic!("Missing {label}"));
        dioxus::core::ElementId::from_raw(*id)
    }
    fn click(&mut self, label: &str) -> (bool, bool) {
        let result = dispatch(&self.dom, "click", self.target(label), ());
        self.settle();
        result
    }
    fn key(&mut self, label: &str, key: Key) -> (bool, bool) {
        if label == "Menu" {
            dispatch(&self.dom, "focus", self.target(label), ());
        }
        let result = dispatch(
            &self.dom,
            "keydown",
            self.target(label),
            (key, Modifiers::empty()),
        );
        self.settle();
        result
    }
    fn visible(&mut self, label: &str, fact: TestVisible) {
        dispatch(&self.dom, "visible", self.target(label), fact);
        self.settle();
    }
    fn html(&self) -> String {
        dioxus_ssr::render(&self.dom)
    }
}

fn item_tag(html: &str, id: &str) -> String {
    let at = html.find(&format!("id=\"{id}\"")).unwrap();
    let start = html[..at].rfind('<').unwrap();
    let end = at + html[at..].find('>').unwrap() + 1;
    html[start..end].to_string()
}

fn unrelated_trigger_target(h: &Harness) -> (Rc<Cell<bool>>, Rc<RefCell<Vec<Change>>>) {
    let owner = h.context().node.borrow().clone().unwrap();
    let runtime = owner.runtime().unwrap();
    let open = Rc::new(Cell::new(true));
    let seen = Rc::new(RefCell::new(Vec::new()));
    let read = open.clone();
    let apply = open.clone();
    let before = seen.clone();
    owner
        .with_owner(|| {
            runtime.register(
                "actual-trigger-unrelated-target".into(),
                1,
                OverlayKind::Dropdown,
                CloseBehavior::Both,
                overlay_runtime::Callbacks {
                    read: Rc::new(move || Some(read.get())),
                    before: Rc::new(move |change| {
                        before.borrow_mut().push(change);
                        true
                    }),
                    apply: Rc::new(move |change| apply.set(change.open)),
                    key: None,
                },
            );
        })
        .unwrap();
    (open, seen)
}

#[test]
fn dioxus_keys_keep_navigation_after_read_only_order_observation() {
    let _guard = crate::test_event_converter_guard();
    for mode in [
        DropdownNavigationMode::ActiveDescendant,
        DropdownNavigationMode::RovingTabindex,
    ] {
        let mut h = Harness::with_settings(Fixture {
            mode,
            ..Fixture::default()
        });
        h.key("Menu", Key::ArrowDown);
        let context = h.context();
        let node = context.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        let order = context.entries.borrow().keys().copied().collect::<Vec<_>>();
        dispatch(&h.dom, "input", transport, serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe",
            "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,"discovery_ready":true,"root_capture":"typed-current-root","items":order}]}).to_string());
        assert!(h.key("Menu", Key::End).0);
        assert_eq!(
            context.entries.borrow()[&context.state.borrow().active.unwrap()].label,
            "Explicit"
        );
        assert!(h.key("Alpha", Key::Home).0);
        assert_eq!(
            context.entries.borrow()[&context.state.borrow().active.unwrap()].label,
            "Alpha"
        );
    }
}

#[test]
fn dioxus_submenu_open_veto_keeps_navigation_with_the_visible_owner() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        focus_checked: true,
        deep_veto: true,
        ..Fixture::default()
    });
    h.key("Menu", Key::ArrowDown);
    h.key("Menu", Key::ArrowRight);
    let root = h.context();
    let child = h.config.borrow().linked.get().unwrap().context();
    assert!(*root.open.peek() && *child.open.peek());
    h.key("Menu", Key::End);
    h.key("Menu", Key::ArrowRight);
    let grand = child
        .entries
        .borrow()
        .values()
        .find(|entry| entry.label == "Deep")
        .unwrap()
        .submenu
        .unwrap()
        .context();
    assert!(!*grand.open.peek());
    h.key("Menu", Key::Home);
    assert_eq!(
        child.entries.borrow()[&child.state.borrow().active.unwrap()].label,
        "Detached leaf"
    );
    assert!(
        !*grand.open.peek(),
        "ordinary navigation never retries a vetoed submenu"
    );
}

#[test]
fn directly_focused_dialog_navigates_but_blurred_dialog_preserves_nested_field_keys() {
    let _guard = crate::test_event_converter_guard();
    for mode in [
        DropdownNavigationMode::ActiveDescendant,
        DropdownNavigationMode::RovingTabindex,
    ] {
        let mut h = Harness::with_settings(Fixture {
            mode,
            ..Fixture::default()
        });
        h.click("Menu");
        let context = h.context();
        let dialog_id = context.host_id();
        let dialog = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("id") == Some(&dialog_id))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        dispatch(&h.dom, "focus", dialog, ());
        for (key, label) in [(Key::Home, "Alpha"), (Key::End, "Explicit")] {
            assert!(dispatch(&h.dom, "keydown", dialog, (key, Modifiers::empty())).0);
            h.settle();
            assert_eq!(
                context.entries.borrow()[&context.state.borrow().active.unwrap()].label,
                label
            );
        }
        let current_dialog = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("id") == Some(&dialog_id))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        assert_eq!(
            current_dialog, dialog,
            "dialog identity stays current through item navigation"
        );
        let field = h.target("Editing field");
        assert_ne!(
            field, dialog,
            "the fixture must address the nested input itself"
        );
        dispatch(&h.dom, "blur", current_dialog, ());
        assert!(
            !dispatch(
                &h.dom,
                "keydown",
                current_dialog,
                (Key::Home, Modifiers::empty())
            )
            .0,
            "{mode:?}: blur releases direct dialog keyboard ownership"
        );
        dispatch(&h.dom, "focus", field, ());
        let selected = context.state.borrow().active;
        for key in [Key::Home, Key::End, Key::ArrowDown, Key::ArrowUp] {
            assert!(
                !h.key("Editing field", key).0,
                "{mode:?}: a descendant field retains its native editing default"
            );
            assert_eq!(
                context.state.borrow().active,
                selected,
                "the dialog no longer owns field navigation"
            );
        }
        assert!(*context.open.peek());
    }
}

#[test]
fn ordinary_content_fields_keep_editing_keys_and_escape_uses_dioxus_dismissal() {
    let _guard = crate::test_event_converter_guard();
    for mode in [
        DropdownNavigationMode::ActiveDescendant,
        DropdownNavigationMode::RovingTabindex,
    ] {
        let mut h = Harness::with_settings(Fixture {
            mode,
            ..Fixture::default()
        });
        h.click("Menu");
        let context = h.context();
        let before = context.state.borrow().active;
        for key in [
            Key::Home,
            Key::End,
            Key::ArrowDown,
            Key::ArrowLeft,
            Key::Enter,
            Key::Character(" ".into()),
            Key::Character("x".into()),
        ] {
            assert!(
                !h.key("Editing field", key).0,
                "ordinary fields keep native editing/activation"
            );
            assert_eq!(context.state.borrow().active, before);
            assert!(*context.open.peek());
        }
        h.key("Editing field", Key::Escape);
        assert!(!*context.open.peek());
    }
}

#[test]
fn disabled_focus_and_prevented_original_dioxus_keys_leave_navigation_unchanged() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    h.click("Menu");
    let context = h.context();
    let before = context.state.borrow().active;
    let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new((
        Key::End,
        Modifiers::empty(),
    ))));
    let event = Event::new(data, true);
    event.prevent_default();
    h.dom
        .runtime()
        .handle_event("keydown", event, h.target("Menu"));
    h.settle();
    assert_eq!(context.state.borrow().active, before);
    h.config.borrow_mut().no_focus = true;
    h.update();
    assert!(!h.key("Menu", Key::Home).0);
    assert_eq!(context.state.borrow().active, before);
}

#[test]
fn actual_mouse_triggers_dismiss_others_despite_visibility_veto_but_public_handles_do_not() {
    let _guard = crate::test_event_converter_guard();
    for label in ["Menu", "External trigger", "Linked"] {
        let mut h = Harness::with_settings(Fixture {
            veto_open: true,
            linked_veto: true,
            ..Fixture::default()
        });
        let (other, seen) = unrelated_trigger_target(&h);
        let linked = h.config.borrow().linked.get().unwrap();
        h.dom.in_runtime(|| {
            linked.open();
            linked.close(false);
            linked.toggle();
        });
        h.settle();
        assert!(
            other.get(),
            "public handle requests must not acquire the trigger phase"
        );
        assert!(seen.borrow().is_empty());
        // Actual component listener dispatch is isolated from native layout
        // and hierarchy here; the unrelated target is a registered owner.
        h.click(label);
        assert!(
            !other.get(),
            "{label}: own visibility veto does not cancel other dismissal"
        );
        assert_eq!(seen.borrow().len(), 1);
        assert_eq!(seen.borrow()[0].reason, Reason::TriggerOther);
        assert!(!*h.context().open.peek());
        assert!(!linked.is_open());
    }
}

#[test]
fn closed_ordinary_trigger_keys_run_others_but_open_navigation_and_submenu_delegation_do_not() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    let (other, seen) = unrelated_trigger_target(&h);
    h.key("Menu", Key::ArrowDown);
    assert!(!other.get());
    assert_eq!(seen.borrow().len(), 1);
    other.set(true);
    h.key("Menu", Key::Home);
    assert!(
        other.get(),
        "already open navigation has no toggle-controller phase"
    );
    h.dom
        .in_runtime(|| h.context().request(false, Reason::Trigger, false));
    h.settle();
    h.context().state.borrow_mut().anchor_in_dropdown_item = true;
    h.key("Menu", Key::Enter);
    assert!(
        other.get(),
        "anchor-submenu delegation has no ordinary trigger phase"
    );
    assert_eq!(seen.borrow().len(), 1);
}

#[test]
fn original_prevented_component_click_skips_own_toggle_and_other_dismissal() {
    let _guard = crate::test_event_converter_guard();
    for label in ["Menu", "External trigger", "Linked"] {
        let mut h = Harness::new();
        let (other, seen) = unrelated_trigger_target(&h);
        let data: Rc<dyn std::any::Any> =
            Rc::new(dioxus::html::PlatformEventData::new(Box::new(())));
        let event = Event::new(data, true);
        event.prevent_default();
        h.dom
            .runtime()
            .handle_event("click", event, h.target(label));
        h.settle();
        assert!(!*h.context().open.peek());
        assert!(!h.config.borrow().linked.get().unwrap().is_open());
        assert!(other.get());
        assert!(seen.borrow().is_empty());
    }
}

#[test]
fn dioxus_visible_geometry_updates_placement_and_full_visibility_resets_it() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        placement: DropdownPlacement::BottomEnd,
        suppress_automatic: true,
        ..Fixture::default()
    });
    h.click("Menu");
    let host = |h: &Harness| item_tag(&h.html(), &h.context().host_id());
    h.visible("Menu", TestVisible::new(true, [0.0, 20.0, 0.0, 100.0]));
    assert!(host(&h).contains("data-placement=\"top-end\""));
    h.visible("Menu", TestVisible::new(true, [0.0, 40.0, 0.0, 100.0]));
    assert!(host(&h).contains("data-placement=\"bottom-end\""));
    assert!(host(&h).contains("position-try-fallbacks:none"));
    h.config.borrow_mut().suppress_automatic = false;
    h.update();
    assert!(!host(&h).contains("position-try-fallbacks:none"));
    assert!(*h.context().open.peek());
    assert_eq!(h.config.borrow().requests, [(true, false)]);
    assert_eq!(
        h.config.borrow().mounts,
        1,
        "visibility does not remount content"
    );
}

#[test]
fn dioxus_visible_dismissal_honors_veto_suppression_and_preopen_cache() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    h.config.borrow_mut().veto_close = true;
    h.click("Menu");
    h.visible("Menu", TestVisible::new(false, [0.0; 4]));
    assert!(*h.context().open.peek());
    assert_eq!(h.config.borrow().requests, [(true, false), (false, true)]);
    h.config.borrow_mut().veto_close = false;
    h.visible("Menu", TestVisible::new(false, [0.0; 4]));
    assert!(!*h.context().open.peek());

    let mut preopen = Harness::new();
    preopen.visible("Menu", TestVisible::new(false, [0.0; 4]));
    assert!(preopen.config.borrow().requests.is_empty());
    preopen.click("Menu");
    assert!(
        !*preopen.context().open.peek(),
        "ordinary opening applies the current mounted anchor's cached visibility"
    );

    let mut cached = Harness::with_settings(Fixture {
        suppress_visibility: true,
        ..Fixture::default()
    });
    cached.visible("Menu", TestVisible::new(false, [0.0; 4]));
    assert!(
        cached.config.borrow().requests.is_empty(),
        "closed visibility only updates the owned cache"
    );
    cached.click("Menu");
    assert!(
        *cached.context().open.peek(),
        "suppression permits ordinary opening"
    );
    cached.config.borrow_mut().suppress_visibility = false;
    cached.update();
    assert!(
        !*cached.context().open.peek(),
        "current pre-open facts apply when checking is enabled"
    );
    cached.visible("Menu", TestVisible::new(true, [0.0, 40.0, 0.0, 100.0]));
    cached.click("Menu");
    assert!(*cached.context().open.peek());
}

#[test]
fn external_and_linked_library_triggers_deliver_visibility_from_the_current_mount() {
    let _guard = crate::test_event_converter_guard();
    for label in ["External trigger", "Linked"] {
        let mut h = Harness::new();
        let mock = mount_trigger_mock(&mut h, label);
        if label == "Linked" {
            h.click("Menu");
        }
        h.visible(label, TestVisible::new(true, [0.0, 40.0, 0.0, 100.0]));
        h.click(label);
        let linked = h.config.borrow().linked.get().unwrap();
        assert!(linked.is_open());
        if label == "Linked" {
            assert!(
                item_tag(&h.html(), &linked.context().host_id())
                    .contains("data-placement=\"right-start\"")
            );
        }
        let focused = mock.calls.borrow().clone();
        h.visible(label, TestVisible::new(false, [0.0; 4]));
        assert!(!linked.is_open());
        assert_eq!(
            *mock.calls.borrow(),
            focused,
            "visibility dismissal does not steal independent focus"
        );
    }
}

#[test]
fn dropdown_persistent_tree_keeps_scope_and_listeners_across_open_mode_and_layer() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    let alpha = h.target("Alpha");
    let context = h.context();
    assert!(!*context.open.peek());
    assert!(
        h.html().contains("Persistent nested"),
        "closed nested content remains mounted"
    );
    assert_eq!((h.config.borrow().mounts, h.config.borrow().drops), (1, 0));
    h.click("Menu");
    assert!(*context.open.peek());
    let count = *h
        .listeners
        .get("click")
        .unwrap()
        .iter()
        .find(|id| {
            h.attrs
                .get(&id.raw())
                .is_some_and(|a| a.get("data-count-button").is_some())
        })
        .unwrap();
    assert_eq!(
        dispatch(&h.dom, "click", count, ()),
        (false, false),
        "ordinary button defaults remain available after content dismissal"
    );
    h.settle();
    assert!(h.html().contains("Local 1"));
    // Content dismissal keeps the button default and its component scope.
    assert!(!*context.open.peek());
    h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
    h.update();
    h.config.borrow_mut().layer = true;
    h.update();
    h.click("Menu");
    h.click("Menu");
    assert_eq!(h.target("Alpha"), alpha);
    assert!(h.html().contains("Local 1"));
    assert_eq!((h.config.borrow().mounts, h.config.borrow().drops), (1, 0));
    assert!(h.html().contains("popover=\"manual\""));
    h.config.borrow_mut().visible = false;
    h.update();
    assert_eq!(h.config.borrow().drops, 1);
    assert!(
        !context.request(true, Reason::Trigger, false),
        "detached owner rejects stale controller work"
    );
}
#[test]
fn dropdown_actual_visibility_requests_veto_retry_and_reentrant_change_are_ordered() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    let context = h.context();
    let host = *h
        .attrs
        .iter()
        .find(|(_, attrs)| attrs.get("data-ix-overlay-host") == Some(&context.node_id()))
        .unwrap()
        .0;
    assert_eq!(
        h.attrs[&host].get("inert").map(String::as_str),
        Some(""),
        "closed host uses boolean attribute presence"
    );
    h.config.borrow_mut().veto_open = true;
    let result = h.click("Menu");
    assert!(
        !result.0,
        "visibility veto is distinct from original click cancellation"
    );
    assert!(!*context.open.peek());
    assert!(h.config.borrow().changed.is_empty());
    h.config.borrow_mut().veto_open = false;
    h.click("Menu");
    assert!(*context.open.peek());
    assert!(
        !h.attrs[&host].contains_key("inert"),
        "open host removes inert instead of serializing inert=false"
    );
    h.config.borrow_mut().veto_close = true;
    h.click("Menu");
    assert!(*context.open.peek());
    h.config.borrow_mut().veto_close = false;
    h.click("Menu");
    assert!(!*context.open.peek());
    assert_eq!(
        h.attrs[&host].get("inert").map(String::as_str),
        Some(""),
        "closing restores inert presence on same persistent host"
    );
    assert_eq!(
        h.config.borrow().requests,
        [(true, false), (true, false), (false, true), (false, true)]
    );
    h.config.borrow_mut().close_after_open = true;
    h.click("Menu");
    assert!(
        !*context.open.peek(),
        "nested command's final state survives outer callback"
    );
    assert_eq!(h.config.borrow().changed, [true, false, true, false]);
}
#[test]
fn content_click_policy_preserves_native_defaults_and_never_toggles_the_trigger() {
    let _guard = crate::test_event_converter_guard();
    for behavior in [
        DropdownCloseBehavior::Both,
        DropdownCloseBehavior::Inside,
        DropdownCloseBehavior::Outside,
        DropdownCloseBehavior::None,
    ] {
        let mut h = Harness::with_settings(Fixture {
            close_behavior: behavior,
            ..Fixture::default()
        });
        assert_eq!(h.click("Menu"), (false, false));
        assert!(*h.context().open.peek());
        assert_eq!(
            h.click("Alpha"),
            (false, false),
            "ordinary item defaults remain available: {behavior:?}"
        );
        let stays_open = matches!(
            behavior,
            DropdownCloseBehavior::Outside | DropdownCloseBehavior::None
        );
        assert_eq!(
            *h.context().open.peek(),
            stays_open,
            "inside policy: {behavior:?}"
        );
        assert_eq!(h.config.borrow().item_requests, 1);
        assert_eq!(h.config.borrow().native_clicks, 1);
        if stays_open {
            h.click("Menu");
        }
        assert!(!*h.context().open.peek());
        let requests = h.config.borrow().requests.len();
        assert_eq!(h.click("Alpha"), (false, false));
        assert!(
            !*h.context().open.peek(),
            "closed content is never a trigger"
        );
        assert_eq!(h.config.borrow().requests.len(), requests);
    }
}

#[test]
fn keep_open_and_consumer_prevention_retain_the_open_owner() {
    let _guard = crate::test_event_converter_guard();
    for keep_open in [false, true] {
        let mut h = Harness::with_settings(Fixture {
            item_keep_open: keep_open,
            item_prevent_click: !keep_open,
            ..Fixture::default()
        });
        h.click("Menu");
        assert_eq!(h.click("Alpha"), (true, false));
        assert!(*h.context().open.peek());
        assert_eq!(h.config.borrow().requests, [(true, false)]);
        assert_eq!(h.config.borrow().item_requests, 1);
        assert_eq!(h.config.borrow().native_clicks, 1);
    }
    let mut h = Harness::with_settings(Fixture {
        veto_close: true,
        ..Fixture::default()
    });
    h.click("Menu");
    assert_eq!(
        h.click("Alpha"),
        (false, false),
        "visibility veto does not cancel the original click default"
    );
    assert!(*h.context().open.peek());
    assert_eq!(h.config.borrow().requests, [(true, false), (false, true)]);
    h.config.borrow_mut().veto_close = false;
    assert_eq!(h.click("Alpha"), (false, false));
    assert!(
        !*h.context().open.peek(),
        "the next content click can retry a vetoed close"
    );
}

#[test]
fn dropdown_linked_clicks_toggle_only_their_child_and_selection_closes_current_tree() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    let root = h.context();
    let child = h.config.borrow().linked.get().unwrap();
    // Supply read-only current ancestry for this renderer without a DOM observer.
    let root_node = root.node.borrow().clone().unwrap();
    let child_node = child.context().node.borrow().clone().unwrap();
    let runtime = root_node.runtime().unwrap();
    let transport = h
        .attrs
        .iter()
        .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
        .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
        .unwrap();
    dispatch(&h.dom, "input", transport, serde_json::json!({
        "owner":runtime.owner_id(),"sequence":1,"kind":"observe",
        "nodes":[
            {"id":root_node.id(),"generation":root_node.generation(),"connected":true,"discovery_ready":true},
            {"id":child_node.id(),"generation":child_node.generation(),"connected":true,"discovery_ready":true,"trigger_ancestors":[root_node.id()]}
        ]
    }).to_string());
    h.settle();
    h.click("Menu");
    let result = h.click("Linked");
    assert_eq!(
        result,
        (false, false),
        "linked clicks stop virtual bubbling while keeping native defaults"
    );
    assert!(*root.open.peek() && child.is_open());
    h.click("Linked");
    assert!(
        *root.open.peek() && !child.is_open(),
        "the second click only closes its own child"
    );
    h.click("Linked");
    assert!(*root.open.peek() && child.is_open());
    assert_eq!(h.click("Detached leaf"), (false, false));
    assert!(
        !*root.open.peek() && !child.is_open(),
        "child selection dismisses the current linked hierarchy"
    );
    h.click("Menu");
    h.config.borrow_mut().item_veto = true;
    let result = h.click("Alpha");
    assert!(
        !result.0,
        "internal item notification does not cancel original click"
    );
    assert_eq!(h.config.borrow().item_requests, 1);
    assert_eq!(h.config.borrow().native_clicks, 1);
    assert!(
        !*root.open.peek(),
        "content dismissal is independent of notification cancellation"
    );
    h.click("Menu");
    let result = h.click("Disabled");
    assert!(result.0);
    assert!(
        *root.open.peek(),
        "disabled content cannot dismiss its owner"
    );
    assert_eq!(h.config.borrow().item_requests, 1);
    let enter = h.key("Standalone", Key::Enter);
    let space = h.key("Standalone", Key::Character(" ".into()));
    assert!(
        !enter.0 && !space.0,
        "notification veto does not prevent original standalone keys"
    );
    assert_eq!(h.config.borrow().standalone_requests, 2);
    assert_eq!(
        h.config.borrow().standalone_mouse,
        0,
        "keyboard notification never fabricates MouseEvent"
    );
}
fn mixed_inventory_facts() -> InventoryFacts {
    serde_json::from_value(serde_json::json!({
        "items_host":"mixed-items-host",
        "hosts":[
            {"item":"actual-category","kind":"category","target":"mixed-category","capture":"actual-category","registration_key":"category-key","source_focusable":false},
            {"item":"actual-menu","kind":"menu","target":"mixed-menu","capture":"actual-menu","registration_key":"menu-key"},
            {"item":"actual-native","kind":"native-opt-in","target":"mixed-native","capture":"actual-native","source_focusable":false,"roving_opt_in":true}
        ],
        "navigation_order":["actual-category","actual-menu","actual-native"],
        "query_order":["actual-category","actual-menu","actual-native"]
    })).unwrap()
}
struct InventoryMount {
    target: String,
    focuses: Rc<RefCell<Vec<String>>>,
}
impl dioxus::html::RenderedElementBacking for InventoryMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        self.focuses.borrow_mut().push(self.target.clone());
        Box::pin(async { Ok(()) })
    }
}
fn mount_inventory(h: &Harness, facts: &InventoryFacts) -> Rc<RefCell<Vec<String>>> {
    let focuses = Rc::new(RefCell::new(Vec::new()));
    let runtime = h
        .context()
        .node
        .borrow()
        .as_ref()
        .unwrap()
        .runtime()
        .unwrap();
    for host in &facts.hosts {
        let key = if host.registration_key.is_empty() {
            &host.capture
        } else {
            &host.registration_key
        };
        runtime.register_item_cue(key.clone(), host.kind);
        runtime.register_item_mount(
            key,
            &host.target,
            Rc::new(MountedData::new(InventoryMount {
                target: host.target.clone(),
                focuses: focuses.clone(),
            })),
        );
    }
    focuses
}
fn key_event(key: &str, modifiers: Modifiers) -> KeyboardEvent {
    let key = match key {
        "Home" => Key::Home,
        "End" => Key::End,
        "ArrowDown" => Key::ArrowDown,
        "ArrowUp" => Key::ArrowUp,
        "ArrowLeft" => Key::ArrowLeft,
        "ArrowRight" => Key::ArrowRight,
        "Enter" => Key::Enter,
        "Escape" => Key::Escape,
        "Tab" => Key::Tab,
        other => Key::Character(other.into()),
    };
    Event::new(
        Rc::new(dioxus::html::KeyboardData::new(TestKeyboard(
            key, modifiers,
        ))),
        true,
    )
}
fn registered_inventory_key_from(
    h: &Harness,
    sequence: u64,
    key: &str,
    facts: &InventoryFacts,
    focused: Option<String>,
) -> bool {
    let context = h.context();
    let node = context.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let mode = context.state.borrow().settings.mode;
    let transport = h
        .attrs
        .iter()
        .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
        .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
        .unwrap();
    dispatch(&h.dom, "input", transport, serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":"observe",
        "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,"open":true,"discovery_ready":true,"inventory":facts}]}).to_string());
    h.dom.in_runtime(|| {
        let event = key_event(key, Modifiers::empty());
        let on_trigger = mode == DropdownNavigationMode::ActiveDescendant && focused.is_none();
        node.dioxus_key_facts(&event, on_trigger, focused, true)
            .is_some_and(|facts| node.dioxus_key(facts).prevent)
    })
}
fn registered_inventory_key(h: &Harness, sequence: u64, key: &str, facts: &InventoryFacts) -> bool {
    registered_inventory_key_from(h, sequence, key, facts, None)
}

#[test]
fn registered_category_inventory_ignores_legacy_focus_and_retains_projection_seam() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    h.click("Menu");
    let context = h.context();
    let legacy = context.entries.borrow().keys().next().copied().unwrap();
    context.state.borrow_mut().active = Some(legacy);
    let facts = mixed_inventory_facts();
    let focuses = mount_inventory(&h, &facts);
    assert!(registered_inventory_key(&h, 1, "Home", &facts));
    let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
    assert_eq!(runtime.pending_item_focus_count(), 1);
    h.settle();
    assert_eq!(focuses.borrow().as_slice(), ["mixed-category"]);
    assert_eq!(
        context.active_dom_id(),
        None,
        "projected parent is not an outer AD marker/ARIA alias"
    );
    assert!(registered_inventory_key(&h, 2, "ArrowDown", &facts));
    h.settle();
    assert_eq!(
        focuses.borrow().as_slice(),
        ["mixed-category", "mixed-category"],
        "no current host marker means Down selects the first Category again"
    );
    assert!(
        registered_inventory_key(&h, 3, "Enter", &facts),
        "source activation callback prevents Enter even without any active host"
    );
    assert!(runtime.pending_item_focus_count() == 0);
}

fn rendered_cue_host(h: &Harness, label: &str) -> crate::dropdown_inventory::HostFacts {
    let attributes = &h.attrs[&h.target(label).raw()];
    let id = &attributes["id"];
    serde_json::from_value(serde_json::json!({
        "item":format!("rendered-{id}"),"capture":format!("rendered-{id}"),
        "kind":"dropdown","target":id,"registration_key":attributes["data-ix-item-key"],
        "source_focusable":true
    }))
    .unwrap()
}
fn cue_tag<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('>')
        .find(|tag| tag.contains(&format!(" id=\"{id}\"")))
        .unwrap_or_else(|| panic!("missing cue target {id}"))
}

#[test]
fn direct_roving_item_focus_preserves_keyboard_position_before_inventory() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        mode: DropdownNavigationMode::RovingTabindex,
        ..Fixture::default()
    });
    h.key("Menu", Key::ArrowDown);
    let context = h.context();
    let entry = |label: &str| {
        context
            .entries
            .borrow()
            .iter()
            .find(|(_, entry)| entry.label == label)
            .map(|(id, _)| *id)
            .unwrap()
    };
    assert!(
        context
            .node
            .borrow()
            .as_ref()
            .unwrap()
            .inventory()
            .is_none()
    );
    assert_eq!(context.state.borrow().active, Some(entry("Alpha")));
    dispatch(&h.dom, "focus", h.target("Alpha"), ());
    h.settle();
    h.key("Alpha", Key::ArrowDown);
    assert_eq!(
        context.state.borrow().active,
        Some(entry("Linked")),
        "Down continues after the focused item and skips Disabled"
    );
}

#[test]
fn direct_item_focus_and_blur_render_owned_aria_before_native_inventory() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        alpha_id: Some("direct-focus-alpha".into()),
        ..Fixture::default()
    });
    h.click("Menu");
    let context = h.context();
    let node = context.node.borrow().as_ref().unwrap().clone();
    let runtime = node.runtime().unwrap();
    let sibling = h.config.borrow().linked.get().unwrap();
    h.dom.in_runtime(|| sibling.open());
    h.settle();
    assert!(node.inventory().is_none());
    let transport = h
        .attrs
        .iter()
        .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
        .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
        .unwrap();
    dispatch(
        &h.dom,
        "input",
        transport,
        serde_json::json!({
            "owner":runtime.owner_id(),"document":format!("direct-focus-{}",runtime.owner_id()),
            "sequence":1,"kind":"modality","input_kind":"keydown","key":"Tab","nodes":[]
        })
        .to_string(),
    );
    dispatch(&h.dom, "focus", h.target("Alpha"), ());
    h.settle();
    assert_eq!(h.config.borrow().item_focus_events, 1);
    let html = h.html();
    assert!(cue_tag(&html, "direct-focus-alpha").contains("ix-focused"));
    assert!(
        cue_tag(&html, &context.trigger_id())
            .contains("aria-activedescendant=\"direct-focus-alpha\"")
    );
    assert_eq!(
        sibling.active_descendant(),
        None,
        "an open sibling cannot claim this item"
    );
    assert!(!cue_tag(&html, &sibling.context().trigger_id()).contains("aria-activedescendant"));
    dispatch(&h.dom, "blur", h.target("Alpha"), ());
    h.settle();
    assert_eq!(h.config.borrow().item_blur_events, 1);
    assert!(runtime.item_visual_key().is_none());
    assert!(!cue_tag(&h.html(), "direct-focus-alpha").contains("ix-focused"));
    assert!(!cue_tag(&h.html(), &context.trigger_id()).contains("aria-activedescendant"));
    assert_eq!(
        context.state.borrow().active,
        context
            .entries
            .borrow()
            .iter()
            .find(|(_, entry)| entry.label == "Alpha")
            .map(|(id, _)| *id),
        "blur clears the cue while retaining the keyboard position"
    );
    dispatch(&h.dom, "focus", h.target("Alpha"), ());
    h.config.borrow_mut().visible = false;
    h.update();
    assert!(
        runtime.item_visual_key().is_none(),
        "unmount releases the focused registration"
    );
}

#[test]
fn mounted_navigation_cues_render_classes_root_aria_and_survive_props_without_inheriting_replacement()
 {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        alpha_id: Some("rendered-cue-alpha".into()),
        ..Fixture::default()
    });
    h.click("Menu");
    let context = h.context();
    let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
    let host = rendered_cue_host(&h, "Alpha");
    let facts = InventoryFacts {
        items_host: ItemRef("rendered-cue-items".into()),
        hosts: vec![host.clone()],
        navigation_order: vec![host.item.clone()],
        query_order: vec![host.item.clone()],
        slots: None,
    };
    assert!(registered_inventory_key(&h, 1, "Home", &facts));
    h.settle();
    let html = h.html();
    let item = cue_tag(&html, &host.target);
    assert!(item.contains("ix-focused") && item.contains("data-active=\"true\""));
    assert!(
        cue_tag(&html, &context.trigger_id())
            .contains("aria-activedescendant=\"rendered-cue-alpha\"")
    );
    h.config.borrow_mut().listener_revision += 1;
    h.update();
    assert!(cue_tag(&h.html(), &host.target).contains("ix-focused"));
    h.config.borrow_mut().visible = false;
    h.update();
    assert!(runtime.item_visual_target().is_none());
    h.config.borrow_mut().visible = true;
    h.update();
    let html = h.html();
    let replacement = cue_tag(&html, &host.target);
    assert!(replacement.contains("data-active=\"false\"") && !replacement.contains("ix-focused"));
}

#[test]
fn linked_open_cue_and_typed_trigger_aria_follow_current_rust_submenu_state() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    h.click("Menu");
    h.click("Linked");
    let child = h.config.borrow().linked.get().unwrap();
    let context = child.context();
    let node = context.node.borrow().as_ref().unwrap().clone();
    let runtime = node.runtime().unwrap();
    let host = rendered_cue_host(&h, "Detached leaf");
    let transport = h
        .attrs
        .iter()
        .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
        .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
        .unwrap();
    dispatch(&h.dom,"input",transport,serde_json::json!({
        "owner":runtime.owner_id(),"document":format!("rendered-linked-cue-{}",runtime.owner_id()),
        "sequence":1,"kind":"modality","input_kind":"keydown","key":"Tab",
        "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,
            "inventory":{"items_host":"rendered-child-items","hosts":[host],"navigation_order":[host.item],"query_order":[host.item]}}]
    }).to_string());
    assert!(
        runtime.keyboard_mode(),
        "the observed document enables keyboard focus cues"
    );
    h.dom.in_runtime(|| runtime.mark_item_visual(&host));
    h.settle();
    assert_eq!(
        child.active_descendant().as_deref(),
        Some(host.target.as_str())
    );
    let html = h.html();
    let linked_id = h.attrs[&h.target("Linked").raw()]["id"].clone();
    let linked = cue_tag(&html, &linked_id);
    assert!(linked.contains("ix-dropdown-submenu-trigger-active"));
    assert!(linked.contains(&format!("aria-activedescendant=\"{}\"", host.target)));
    assert!(
        cue_tag(&html, &context.trigger_id())
            .contains(&format!("aria-activedescendant=\"{}\"", host.target))
    );
    h.dom.in_runtime(|| child.close(false));
    h.settle();
    assert!(child.active_descendant().is_none());
    let html = h.html();
    let linked = cue_tag(&html, &linked_id);
    assert!(
        !linked.contains("ix-dropdown-submenu-trigger-active")
            && !linked.contains("aria-activedescendant")
    );
}

#[test]
fn registered_roving_inventory_projects_logical_tab_order_before_render() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        mode: DropdownNavigationMode::RovingTabindex,
        ..Fixture::default()
    });
    h.click("Menu");
    let context = h.context();
    let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
    let facts = mixed_inventory_facts();
    let focuses = mount_inventory(&h, &facts);
    for (index, key) in ["Home", "ArrowDown", "ArrowDown"].into_iter().enumerate() {
        assert!(registered_inventory_key(&h, index as u64 + 1, key, &facts));
    }
    assert_eq!(runtime.pending_item_focus_count(), 1);
    assert!(
        !registered_inventory_key(&h, 4, "Enter", &facts),
        "logical tabindex selects the custom control before rendering; native Enter passes through"
    );
    h.settle();
    assert_eq!(focuses.borrow().as_slice(), ["mixed-native"]);
    assert!(runtime.pending_item_focus_count() == 0);
}
#[test]
fn rapid_navigation_commits_the_latest_mounted_target_without_foreign_commands() {
    let _guard = crate::test_event_converter_guard();
    for mode in [
        DropdownNavigationMode::ActiveDescendant,
        DropdownNavigationMode::RovingTabindex,
    ] {
        let mut h = Harness::with_settings(Fixture {
            mode,
            ..Fixture::default()
        });
        h.click("Menu");
        let context = h.context();
        let node = context.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let order = context.entries.borrow().keys().copied().collect::<Vec<_>>();
        dispatch(&h.dom, "focus", h.target("Menu"), ());
        for key in [Key::End, Key::Home, Key::End] {
            assert!(
                dispatch(
                    &h.dom,
                    "keydown",
                    h.target("Menu"),
                    (key, Modifiers::empty())
                )
                .0
            );
        }
        assert_eq!(runtime.pending_item_focus_count(), 1);
        h.settle();
        assert_eq!(
            context.active_dom_id(),
            Some(format!("ix-dropdown-item-{}", order.last().unwrap()))
        );
        assert!(runtime.pending_item_focus_count() == 0);
        h.click("Menu");
        assert!(!*context.open.peek());
        assert!(
            runtime.pending_item_focus_count() == 0,
            "dismissal leaves no stale focus attempt"
        );
    }
}
#[test]
fn dropdown_actual_keyboard_flags_distinguish_active_descendant_and_roving() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    let context = h.context();
    let result = h.key("Menu", Key::ArrowDown);
    assert!(result.0);
    assert!(*context.open.peek());
    let first = context.state.borrow().active.unwrap();
    assert_eq!(context.entries.borrow()[&first].label, "Alpha");
    assert_eq!(
        context.state.borrow().focus_target,
        "none",
        "source default retains trigger focus"
    );
    let result = h.key("Menu", Key::End);
    assert!(result.0);
    let last = context.state.borrow().active.unwrap();
    assert_eq!(context.entries.borrow()[&last].label, "Explicit");
    let result = h.key("Menu", Key::ArrowRight);
    assert!(!result.0, "submenu Right leaves original default enabled");
    let forward = context.state.borrow().forward.unwrap();
    h.dom.in_runtime(|| forward.close(false));
    h.settle();
    let next = h.key("Menu", Key::ArrowDown);
    assert!(next.0);
    assert_eq!(
        context.entries.borrow()[&context.state.borrow().active.unwrap()].label,
        "Alpha",
        "closed child no longer owns parent item queries"
    );
    h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
    h.update();
    let result = h.key("Menu", Key::ArrowDown);
    assert!(result.0);
    assert_eq!(
        context.state.borrow().focus_target,
        "none",
        "source item focus is independently queued, not the latest attribute"
    );
    let result = h.key("Menu", Key::Character("e".into()));
    assert!(result.0);
    let active = context.state.borrow().active.unwrap();
    assert_eq!(
        context.entries.borrow()[&active].label,
        "Explicit",
        "explicit roving preserves existing typeahead adapter"
    );
    let target = h.target("Menu");
    let previous = context.state.borrow().active;
    let shortcut = dispatch(
        &h.dom,
        "keydown",
        target,
        (Key::Character("a".into()), Modifiers::CONTROL),
    );
    h.settle();
    assert!(!shortcut.0);
    assert_eq!(
        context.state.borrow().active,
        previous,
        "roving typeahead adapter leaves browser shortcuts unchanged"
    );
    assert!(
        !h.html().contains("aria-activedescendant"),
        "roving has no focus proxy aria attribute"
    );
}

#[test]
fn completion_effects_keep_each_payload_and_use_current_committed_listener() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    let context = h.context();
    h.dom.in_runtime(|| {
        for value in [true, false, true] {
            assert!(context.request(value, Reason::Trigger, false));
        }
    });
    assert!(
        h.config.borrow().changed.is_empty(),
        "request admission is synchronous, completion is not"
    );
    h.config.borrow_mut().listener_revision = 7;
    h.config.borrow().update.clone().unwrap()();
    h.completion_effects();
    assert!(*context.open.peek());
    assert_eq!(h.config.borrow().changed, [true, false, true]);
    assert_eq!(
        h.config.borrow().deliveries,
        [(7, true, true), (7, false, true), (7, true, true)]
    );
    h.completion_effects();
    assert_eq!(h.config.borrow().changed.len(), 3, "effects deliver once");
}

#[test]
fn completion_effect_allows_reentrant_state_change_without_borrow_conflict() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        close_after_open: true,
        ..Default::default()
    });
    let context = h.context();
    h.dom
        .in_runtime(|| assert!(context.request(true, Reason::Trigger, false)));
    assert!(h.config.borrow().changed.is_empty());
    h.settle();
    assert_eq!(h.config.borrow().changed, [true, false]);
    assert!(!*context.open.peek());
    assert_eq!(h.config.borrow().requests, [(true, false), (false, true)]);
    assert_eq!(
        h.config.borrow().deliveries,
        [(0, true, true), (0, false, false)]
    );
}

#[test]
fn dropped_component_effect_cannot_call_the_replacement_listener() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    let old = h.context();
    let runtime = old.node.borrow().clone().unwrap().runtime().unwrap();
    h.dom
        .in_runtime(|| assert!(old.request(true, Reason::Trigger, false)));
    assert_eq!(runtime.pending_completions(), 1);
    h.config.borrow_mut().visible = false;
    h.update();
    assert_eq!(runtime.pending_completions(), 0);
    assert!(h.config.borrow().changed.is_empty());
    h.config.borrow_mut().visible = true;
    h.config.borrow_mut().listener_revision = 9;
    h.update();
    let current = h.context();
    assert_ne!(current.id, old.id, "a new component has a new owner");
    assert!(!old.request(true, Reason::Trigger, false));
    h.dom
        .in_runtime(|| assert!(current.request(true, Reason::Trigger, false)));
    assert!(h.config.borrow().changed.is_empty());
    h.settle();
    assert_eq!(h.config.borrow().deliveries, [(9, true, true)]);
}

#[test]
fn typed_tab_wraps_registered_controls_and_preserves_native_defaults() {
    let _guard = crate::test_event_converter_guard();
    for (inside, index, shift, registered, no_trap, expected) in [
        (true, 2, false, true, false, Some("mixed-category")),
        (true, 0, true, true, false, Some("mixed-native")),
        (true, 1, false, true, false, None),
        (false, 2, false, true, false, None),
        (true, 2, false, false, false, None),
        (true, 2, false, true, true, None),
    ] {
        let mut h = Harness::with_settings(Fixture {
            no_trap,
            ..Default::default()
        });
        h.click("Menu");
        let context = h.context();
        let node = context.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let facts = mixed_inventory_facts();
        let focuses = Rc::new(RefCell::new(Vec::new()));
        let focusable_ids = facts
            .hosts
            .iter()
            .map(|host| format!("{}-control", host.target))
            .collect::<Vec<_>>();
        if registered {
            for (host, focus_id) in facts.hosts.iter().zip(&focusable_ids) {
                let key = if host.registration_key.is_empty() {
                    &host.capture
                } else {
                    &host.registration_key
                };
                runtime.register_item_cue(key.clone(), host.kind);
                runtime.register_item_control_mount(
                    key,
                    &host.target,
                    focus_id,
                    Rc::new(MountedData::new(InventoryMount {
                        target: host.target.clone(),
                        focuses: focuses.clone(),
                    })),
                );
            }
        }
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        let before = h.config.borrow().requests.clone();
        let report = serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe","nodes":[{
            "id":node.id(),"generation":node.generation(),"connected":true,"discovery_ready":true,
            "open":true,"focus_count":3,"focused_index":index,"focusable_ids":focusable_ids,"inventory":facts
        }]});
        dispatch(&h.dom, "input", transport, report.to_string());
        let revision_before = *context.revision.peek();
        let prevented = h.dom.in_runtime(|| {
            let event = key_event(
                "Tab",
                if shift {
                    Modifiers::SHIFT
                } else {
                    Modifiers::empty()
                },
            );
            let focused = inside.then(|| focusable_ids[index].clone());
            let facts = node
                .dioxus_key_facts(&event, !inside, focused, inside)
                .unwrap();
            node.dioxus_key(facts).prevent
        });
        if expected.is_some() {
            assert!(
                *context.revision.peek() > revision_before,
                "admitted Tab focus wakes the renderer before any manual settling"
            );
        }
        h.settle();
        assert_eq!(prevented, expected.is_some(), "{report}");
        assert_eq!(
            focuses.borrow().last().map(String::as_str),
            expected,
            "{report}"
        );
        assert_eq!(*context.open.peek(), !no_trap);
        if !no_trap {
            assert_eq!(
                h.config.borrow().requests,
                before,
                "wrapping does not request visibility changes"
            );
        }
        assert_eq!(runtime.pending_item_focus_count(), 0);
    }
}

#[test]
fn roving_typeahead_focuses_authored_item_id_and_dismissal_cancels_pending_mount() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        mode: DropdownNavigationMode::RovingTabindex,
        alpha_id: Some("authored-alpha".into()),
        ..Default::default()
    });
    h.click("Menu");
    let mock = mount_trigger_mock(&mut h, "Alpha");
    assert!(h.key("Menu", Key::Character("a".into())).0);
    assert_eq!(*mock.calls.borrow(), ["Alpha"]);
    let context = h.context();
    let selected = context.state.borrow().active.unwrap();
    assert_eq!(
        context.entries.borrow()[&selected].host_id,
        "authored-alpha"
    );
    let html = h.html();
    assert!(!html.contains("data-ix-overlay-focus-target"));
    assert!(!html.contains("data-ix-overlay-focus-version"));
    mock.pending.set(true);
    mock.started.set(false);
    mock.dropped.set(false);
    h.key("Menu", Key::Character("a".into()));
    assert!(mock.started.get());
    assert!(!mock.dropped.get());
    h.dom
        .in_runtime(|| assert!(context.request(false, Reason::Outside, false)));
    h.settle();
    assert!(
        mock.dropped.get(),
        "dismissal releases the pending mounted focus attempt"
    );
    mock.pending.set(false);
    mock.waiter.borrow_mut().take().unwrap().wake();
    h.settle();
    assert_eq!(*mock.calls.borrow(), ["Alpha"]);
}

#[test]
fn dropdown_actual_request_and_surface_drop_cancel_owned_pending_work() {
    let _guard = crate::test_event_converter_guard();
    let flag = Rc::new(Cell::new(false));
    let mut h = Harness::with_settings(Fixture {
        pending_drop: Some(flag.clone()),
        ..Default::default()
    });
    assert!(
        h.config.borrow().pending_started.get(),
        "owned pending future must actually begin"
    );
    assert!(!flag.get());
    h.click("Menu");
    assert!(*h.context().open.peek());
    assert!(
        flag.get(),
        "accepted request cancels older focus work before state notifications"
    );
    let context = h.context();
    h.config.borrow_mut().visible = false;
    h.update();
    assert!(context.node.borrow().is_none());
    assert!(!context.state.borrow().surface);
    assert!(!context.request(true, Reason::Keyboard, false));
}

#[test]
fn virtual_item_enter_and_space_deliver_one_registered_rust_action_in_both_modes() {
    let _guard = crate::test_event_converter_guard();
    for mode in [
        DropdownNavigationMode::ActiveDescendant,
        DropdownNavigationMode::RovingTabindex,
    ] {
        let mut h = Harness::with_settings(Fixture {
            mode,
            ..Fixture::default()
        });
        for (index, key) in [Key::Enter, Key::Character(" ".into())]
            .into_iter()
            .enumerate()
        {
            if !*h.context().open.peek() {
                h.click("Menu");
            }
            h.key("Menu", Key::Home);
            assert_eq!(
                h.context().entries.borrow()[&h.context().state.borrow().active.unwrap()].label,
                "Alpha"
            );
            h.key("Menu", key);
            assert_eq!(
                h.config.borrow().item_requests,
                index + 1,
                "one original key delivers one typed item action"
            );
            assert_eq!(
                h.config.borrow().native_clicks,
                0,
                "keyboard action does not fabricate a MouseEvent"
            );
        }
    }
}

#[test]
fn dropdown_linked_activation_keys_present_and_forward_without_click_or_key_cancellation() {
    let _guard = crate::test_event_converter_guard();
    for key in [Key::ArrowRight, Key::Enter, Key::Character(" ".into())] {
        let mut h = Harness::with_settings(Fixture {
            focus_checked: true,
            ..Default::default()
        });
        let parent = h.context();
        let child = h.config.borrow().linked.get().unwrap();
        h.key("Menu", Key::ArrowDown);
        let active = parent.state.borrow().active.unwrap();
        assert_eq!(parent.entries.borrow()[&active].label, "Linked");
        let result = h.key("Menu", key.clone());
        assert!(
            !result.0,
            "linked activation leaves the original key unprevented"
        );
        assert!(child.is_open());
        assert!(parent.state.borrow().forward == Some(child));
        let child_context = child.context();
        // This harness dispatches mounted events with unit-backed handles,
        // while NoOpDocument cannot return a native inventory. Opening
        // therefore completes with the Dioxus trigger focus intent.
        assert_eq!(child_context.state.borrow().focus_target, "trigger");
        assert!(child_context.state.borrow().focus_pending.is_none());
        // Repeated keys must never manufacture a click/toggle on the
        // already open trigger after the opening focus request completes.
        h.key("Menu", Key::Enter);
        h.key("Menu", Key::Character(" ".into()));
        assert!(child.is_open());
        assert_eq!(h.config.borrow().native_clicks, 0);
        h.key("Menu", Key::Home);
        h.key("Menu", Key::ArrowDown);
        let selected = child_context.state.borrow().active.unwrap();
        assert_eq!(
            child_context.entries.borrow()[&selected].label,
            "Checked detached",
            "forwarded navigation uses child-owned facts, not root IDs"
        );
        assert!(
            parent
                .state
                .borrow()
                .order
                .iter()
                .all(|id| parent.entries.borrow().contains_key(id)),
            "forwarded facts cannot overwrite root order"
        );
        h.key("Menu", Key::End);
        h.key("Menu", Key::ArrowRight);
        h.key("Menu", Key::End);
        let deepest = parent.navigation_owner(false);
        assert!(deepest.id != parent.id && deepest.id != child_context.id);
        let selected = deepest.state.borrow().active.unwrap();
        assert_eq!(deepest.entries.borrow()[&selected].label, "Deep last");
        assert_eq!(
            parent.active_dom_id(),
            Some(format!("ix-dropdown-item-{selected}")),
            "deep projection and ARIA use the same Rust owner"
        );
    }
    let mut h = Harness::with_settings(Fixture {
        focus_checked: true,
        linked_veto: true,
        ..Default::default()
    });
    let parent = h.context();
    let child = h.config.borrow().linked.get().unwrap();
    h.key("Menu", Key::ArrowDown);
    let result = h.key("Menu", Key::ArrowRight);
    assert!(!result.0);
    assert!(!child.is_open());
    assert!(
        parent.state.borrow().forward == Some(child),
        "veto does not undo source forward intent"
    );
    assert_eq!(
        child.context().state.borrow().focus_target,
        "none",
        "veto cannot publish a native focus target"
    );
}
#[test]
fn dropdown_configuration_updates_live_blocked_triggers_and_open_mode_focus_intent() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    let context = h.context();
    h.click("Menu");
    h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
    h.update();
    assert_eq!(
        context.state.borrow().focus_pending,
        Some(FocusRequest {
            last: false,
            allow_checked: true
        }),
        "open roving transition initializes current first/checked via DOM order read"
    );
    h.config.borrow_mut().mode = DropdownNavigationMode::ActiveDescendant;
    h.update();
    assert_eq!(context.state.borrow().focus_target, "trigger");
    h.click("Menu");
    h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
    h.update();
    assert_eq!(context.state.borrow().focus_target, "none");
    assert_eq!(
        context.state.borrow().focus_pending,
        None,
        "closed mode change does not start focus"
    );
    h.config.borrow_mut().linked_blocked = true;
    h.update();
    let linked = h.config.borrow().linked.get().unwrap();
    let html = h.html();
    let external = html
        .split("ix-dropdown-external-trigger")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(
        external
            .split_ascii_whitespace()
            .any(|token| token == "disabled" || token.starts_with("disabled="))
            && external.contains("aria-expanded=\"false\"")
    );
    assert!(
        h.attrs[&h.target("Linked").raw()]
            .get("aria-disabled")
            .is_some_and(|value| value == "true")
    );
    h.click("Linked");
    h.click("External trigger");
    assert!(!linked.is_open());
    h.config.borrow_mut().linked_blocked = false;
    h.update();
    h.click("External trigger");
    assert!(linked.is_open(), "re-enabling reaches the same controller");
    h.config.borrow_mut().loading = true;
    h.update();
    assert!(
        h.attrs[&h.target("Menu").raw()]
            .get("aria-busy")
            .is_some_and(|value| value == "true")
    );
}
#[test]
fn dropdown_checked_first_last_and_disabled_focus_key_flags_follow_source() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        focus_checked: true,
        ..Default::default()
    });
    let context = h.context();
    h.key("Menu", Key::End);
    let active = context.state.borrow().active.unwrap();
    assert_eq!(
        context.entries.borrow()[&active].label,
        "Explicit",
        "last ignores checked preference"
    );
    h.click("Menu");
    h.key("Menu", Key::ArrowDown);
    let active = context.state.borrow().active.unwrap();
    assert_eq!(
        context.entries.borrow()[&active].label,
        "Linked",
        "first can honor checked preference"
    );
    h.click("Menu");
    h.config.borrow_mut().no_focus = true;
    h.update();
    let result = h.key("Menu", Key::ArrowDown);
    assert!(
        result.0 && !result.1,
        "disabled focus handling prevents and stops opening key"
    );
    assert!(context.state.borrow().active.is_none());
    h.click("Menu");
    h.config.borrow_mut().no_focus = false;
    h.update();
    h.click("Menu");
    assert!(context.state.borrow().active.is_none());
    let enter = h.key("Menu", Key::Enter);
    let space = h.key("Menu", Key::Character(" ".into()));
    assert!(
        enter.0 && space.0,
        "open ordinary trigger activation cancels even without active item"
    );
    assert!(*context.open.peek());
}
#[test]
fn dropdown_order_selection_uses_current_owned_candidates_and_handles_unknown_current() {
    let entries = [
        (1, "Alpha", false),
        (2, "Disabled", true),
        (3, "Gamma", false),
        (4, "Delta", false),
    ]
    .into_iter()
    .map(|(id, label, disabled)| {
        (
            id,
            MenuEntry {
                host_id: format!("ix-dropdown-item-{id}"),
                label: label.into(),
                disabled,
                checked: false,
                mounted: None,
                submenu: None,
            },
        )
    })
    .collect();
    assert_eq!(
        menu_target(&entries, &[4, 1, 2, 3, 99], Some(4), "ArrowDown"),
        Some(1)
    );
    assert_eq!(
        menu_target(&entries, &[4, 1, 2, 3], Some(1), "ArrowUp"),
        Some(4)
    );
    assert_eq!(
        menu_target(&entries, &[3, 2, 1], None, "ArrowDown"),
        Some(3)
    );
    assert_eq!(menu_target(&entries, &[3, 2, 1], None, "ArrowUp"), Some(1));
    assert_eq!(menu_target(&entries, &[2, 99], None, "Home"), None);
    assert_eq!(menu_target(&entries, &[4, 1, 2, 3], Some(4), "g"), Some(3));
}
#[derive(Clone, Default)]
struct TriggerFocusMount {
    name: String,
    calls: Rc<RefCell<Vec<String>>>,
    pending: Rc<Cell<bool>>,
    started: Rc<Cell<bool>>,
    dropped: Rc<Cell<bool>>,
    waiter: Rc<RefCell<Option<std::task::Waker>>>,
}
struct TriggerFocusAttempt(Rc<Cell<bool>>);
impl Drop for TriggerFocusAttempt {
    fn drop(&mut self) {
        self.0.set(true);
    }
}
impl dioxus::html::RenderedElementBacking for TriggerFocusMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let mock = self.clone();
        let guard = TriggerFocusAttempt(self.dropped.clone());
        Box::pin(async move {
            let _guard = guard;
            mock.started.set(true);
            std::future::poll_fn(|cx| {
                if mock.pending.get() {
                    *mock.waiter.borrow_mut() = Some(cx.waker().clone());
                    std::task::Poll::Pending
                } else {
                    std::task::Poll::Ready(())
                }
            })
            .await;
            mock.calls.borrow_mut().push(mock.name.clone());
            Ok(())
        })
    }
}
fn mount_trigger_mock(h: &mut Harness, label: &str) -> TriggerFocusMount {
    let mock = TriggerFocusMount {
        name: label.into(),
        ..Default::default()
    };
    dispatch(&h.dom, "mounted", h.target(label), mock.clone());
    h.settle();
    mock
}
#[test]
fn mounted_trigger_escape_and_explicit_close_run_as_dioxus_effects_in_both_modes() {
    let _guard = crate::test_event_converter_guard();
    for mode in [
        DropdownNavigationMode::ActiveDescendant,
        DropdownNavigationMode::RovingTabindex,
    ] {
        let mut h = Harness::with_settings(Fixture {
            mode,
            ..Default::default()
        });
        let mock = mount_trigger_mock(&mut h, "Menu");
        let context = h.context();
        h.click("Menu");
        h.key("Menu", Key::Escape);
        assert!(!*context.open.peek());
        assert_eq!(*mock.calls.borrow(), ["Menu"]);
        h.click("Menu");
        h.dom
            .in_runtime(|| assert!(context.request(false, Reason::Trigger, true)));
        assert_eq!(
            mock.calls.borrow().len(),
            1,
            "admission never focuses synchronously"
        );
        h.settle();
        assert_eq!(*mock.calls.borrow(), ["Menu", "Menu"]);
        assert!(h.html().contains(r#"data-ix-dropdown-focus-owned="true""#));
    }
}
#[test]
fn shared_modality_cleanup_and_open_active_descendant_mode_change_focus_the_mount() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    let mock = mount_trigger_mock(&mut h, "Menu");
    h.click("Menu");
    let context = h.context();
    let node = context.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let transport = h
        .attrs
        .iter()
        .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
        .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
        .unwrap();
    dispatch(&h.dom, "input", transport, serde_json::json!({"owner":runtime.owner_id(),"document":format!("trigger-focus-test-{}", runtime.owner_id()),"sequence":1,
        "kind":"modality","input_kind":"keydown","key":"Tab",
        "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,"discovery_ready":true,
            "root_capture":"mounted-trigger-root","trigger_capture":"mounted-trigger-control"}]}).to_string());
    assert!(
        runtime.keyboard_mode(),
        "ordinary cleanup reads document modality, so the fixture must attach its document"
    );
    h.dom
        .in_runtime(|| assert!(context.request(false, Reason::AnchorVisibility, true)));
    h.settle();
    assert_eq!(*mock.calls.borrow(), ["Menu"]);
    h.click("Menu");
    h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
    h.update();
    h.config.borrow_mut().mode = DropdownNavigationMode::ActiveDescendant;
    h.update();
    assert!(*context.open.peek());
    assert_eq!(*mock.calls.borrow(), ["Menu", "Menu"]);
}
#[test]
fn mounted_trigger_outside_tab_and_stale_close_requests_preserve_the_destination() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        mode: DropdownNavigationMode::RovingTabindex,
        ..Default::default()
    });
    let mock = mount_trigger_mock(&mut h, "Menu");
    let context = h.context();
    h.click("Menu");
    h.dom
        .in_runtime(|| assert!(context.request(false, Reason::Outside, false)));
    h.settle();
    assert!(mock.calls.borrow().is_empty());
    h.click("Menu");
    h.key("Menu", Key::Tab);
    assert!(!*context.open.peek());
    assert!(mock.calls.borrow().is_empty());
    h.dom.in_runtime(|| {
        assert!(context.request(true, Reason::Trigger, false));
        assert!(context.request(false, Reason::Trigger, true));
        assert!(context.request(true, Reason::Trigger, false));
    });
    h.settle();
    assert!(
        mock.calls.borrow().is_empty(),
        "reopen cancels the queued close focus"
    );
    h.dom
        .in_runtime(|| assert!(context.request(false, Reason::Trigger, true)));
    let replacement = mount_trigger_mock(&mut h, "Menu");
    assert!(mock.calls.borrow().is_empty());
    assert!(
        replacement.calls.borrow().is_empty(),
        "a stale request cannot focus the replacement mount"
    );
}
#[test]
fn trigger_focus_cancels_pending_tasks_and_rejects_disabled_loading_and_dropped_surfaces() {
    let _guard = crate::test_event_converter_guard();
    for loading in [false, true] {
        let mut h = Harness::new();
        let mock = mount_trigger_mock(&mut h, "Menu");
        let context = h.context();
        h.click("Menu");
        h.dom
            .in_runtime(|| assert!(context.request(false, Reason::Trigger, true)));
        if loading {
            h.config.borrow_mut().loading = true;
        } else {
            h.config.borrow_mut().blocked = true;
        }
        h.update();
        assert!(mock.calls.borrow().is_empty());
    }
    let mut h = Harness::new();
    let mock = mount_trigger_mock(&mut h, "Menu");
    mock.pending.set(true);
    let context = h.context();
    h.click("Menu");
    h.dom
        .in_runtime(|| assert!(context.request(false, Reason::Trigger, true)));
    h.settle();
    assert!(mock.started.get());
    assert!(!mock.dropped.get());
    h.dom.in_runtime(|| context.invalidate_focus());
    assert!(
        mock.dropped.get(),
        "a newer focus epoch cancels the pending mounted operation"
    );
    mock.pending.set(false);
    mock.waiter.borrow_mut().take().unwrap().wake();
    h.settle();
    assert!(mock.calls.borrow().is_empty());
    let dropping = mount_trigger_mock(&mut h, "Menu");
    dropping.pending.set(true);
    h.click("Menu");
    h.dom
        .in_runtime(|| assert!(context.request(false, Reason::Trigger, true)));
    h.settle();
    assert!(dropping.started.get());
    h.config.borrow_mut().visible = false;
    h.update();
    assert!(dropping.dropped.get());
    assert!(dropping.calls.borrow().is_empty());
    assert!(!context.state.borrow().surface);
}
#[test]
fn linked_item_and_external_trigger_focus_use_the_current_registered_mount() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    let linked = h.config.borrow().linked.get().unwrap();
    let context = linked.context();
    let item = mount_trigger_mock(&mut h, "Linked");
    h.dom.in_runtime(|| {
        assert!(context.request(true, Reason::Keyboard, false));
        assert!(context.request(false, Reason::Left, true));
    });
    h.settle();
    assert_eq!(*item.calls.borrow(), ["Linked"]);
    let external = mount_trigger_mock(&mut h, "External trigger");
    h.dom.in_runtime(|| {
        assert!(context.request(true, Reason::Trigger, false));
        assert!(context.request(false, Reason::Trigger, true));
    });
    h.settle();
    assert_eq!(*external.calls.borrow(), ["External trigger"]);
    assert_eq!(item.calls.borrow().len(), 1);
}
#[derive(Clone, Default)]
struct CustomTriggerFixture(Rc<CustomTriggerState>);
#[derive(Default)]
struct CustomTriggerState {
    dropdown: Cell<Option<DropdownHandle>>,
    first: Cell<Option<Signal<bool>>>,
    second: Cell<Option<Signal<bool>>>,
    disabled: Cell<Option<Signal<bool>>>,
    loading: Cell<Option<Signal<bool>>>,
    veto: Cell<bool>,
    requests: Cell<usize>,
}
#[component]
fn CustomTriggerControl(dropdown: DropdownHandle, name: String) -> Element {
    let binding = use_dropdown_trigger_binding(dropdown);
    rsx! { button { id: name.clone(), "data-ix-dropdown-label":name, onmounted:binding.onmounted, onvisible:binding.onvisible, onkeydown:binding.onkeydown, onclick:binding.onclick, ..binding.attributes(), "Custom trigger" } }
}
fn custom_trigger_fixture(fixture: CustomTriggerFixture) -> Element {
    let dropdown = use_dropdown();
    let first = use_signal(|| true);
    let second = use_signal(|| true);
    let disabled = use_signal(|| false);
    let loading = use_signal(|| false);
    fixture.0.disabled.set(Some(disabled));
    fixture.0.loading.set(Some(loading));
    let before = fixture.clone();
    fixture.0.dropdown.set(Some(dropdown));
    fixture.0.first.set(Some(first));
    fixture.0.second.set(Some(second));
    rsx! {
        if first() { CustomTriggerControl { dropdown, name:"first" } }
        if second() { CustomTriggerControl { dropdown, name:"second" } }
        Dropdown { controller:dropdown, trigger:Some("second".into()), label:"Custom popup", disabled:disabled(),loading:loading(),
            onbeforechange:move |event:DropdownChangeEvent|{before.0.requests.set(before.0.requests.get()+1);if before.0.veto.get(){event.prevent_default();}},
            DropdownItem { label:"Custom child" } }
    }
}
#[test]
fn explicit_custom_trigger_clicks_honor_current_mount_veto_disabled_loading_and_remount() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let fixture = CustomTriggerFixture::default();
    let mut dom = VirtualDom::new_with_props(custom_trigger_fixture, fixture.clone());
    let edits = dom.rebuild_to_vec();
    let mut h = Harness {
        dom,
        config: Rc::new(RefCell::new(Fixture::default())),
        attrs: BTreeMap::new(),
        listeners: BTreeMap::new(),
    };
    h.collect(edits);
    for target in h.listeners.get("mounted").cloned().unwrap_or_default() {
        dispatch(&h.dom, "mounted", target, ());
    }
    h.settle();
    let handle = fixture.0.dropdown.get().unwrap();
    let context = handle.context();
    let tag = item_tag(&h.html(), "second");
    assert!(tag.contains(&format!(
        "data-ix-overlay-trigger=\"{}\"",
        context.node_id()
    )));
    assert!(
        tag.contains("anchor-name")
            && tag.contains(&format!("--ix-dropdown-anchor-{}", context.id))
    );
    assert!(!h.html().contains("data-ix-overlay-trigger-version"));
    h.click("first");
    assert!(!handle.is_open());
    assert_eq!(
        fixture.0.requests.get(),
        0,
        "an older mounted control cannot activate the current binding"
    );
    h.click("second");
    assert!(handle.is_open());
    h.click("second");
    assert!(!handle.is_open());
    assert_eq!(
        fixture.0.requests.get(),
        2,
        "each original click delivers one visibility request"
    );
    fixture.0.veto.set(true);
    h.click("second");
    assert!(!handle.is_open());
    assert_eq!(fixture.0.requests.get(), 3);
    fixture.0.veto.set(false);
    for loading in [false, true] {
        h.dom.in_runtime(|| {
            fixture.0.disabled.get().unwrap().set(!loading);
            fixture.0.loading.get().unwrap().set(loading);
        });
        h.settle();
        let before = fixture.0.requests.get();
        h.click("second");
        assert!(!handle.is_open());
        assert_eq!(
            fixture.0.requests.get(),
            before,
            "blocked controls do not request opening"
        );
    }
    h.dom
        .in_runtime(|| fixture.0.loading.get().unwrap().set(false));
    h.settle();
    mount_trigger_mock(&mut h, "second");
    h.click("second");
    assert!(
        handle.is_open(),
        "a fresh same-ID mounted handle activates normally"
    );
    let before = fixture.0.requests.get();
    let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new(())));
    let event = Event::new(data, true);
    event.prevent_default();
    h.dom
        .runtime()
        .handle_event("click", event, h.target("second"));
    h.settle();
    assert!(handle.is_open());
    assert_eq!(
        fixture.0.requests.get(),
        before,
        "consumer prevention skips the binding's action"
    );
    h.dom
        .in_runtime(|| fixture.0.first.get().unwrap().set(false));
    h.settle();
    h.click("second");
    assert!(
        !handle.is_open(),
        "old-scope cleanup preserves the current click binding"
    );
    h.dom
        .in_runtime(|| fixture.0.second.get().unwrap().set(false));
    h.settle();
    assert!(context.trigger.borrow().is_none());
    assert!(
        !h.html().contains("id=\"second\""),
        "the removed control leaves the rendered tree"
    );
    h.dom
        .in_runtime(|| fixture.0.second.get().unwrap().set(true));
    h.settle();
    let replacement = *h.listeners.get("mounted").unwrap().last().unwrap();
    assert!(
        context.trigger.borrow().is_none(),
        "rendering metadata does not enroll a mounted focus/click owner"
    );
    assert!(
        item_tag(&h.html(), "second").contains(&format!(
            "data-ix-overlay-trigger=\"{}\"",
            context.node_id()
        )),
        "the explicit binding renders current ancestry metadata before mounting"
    );
    let before = fixture.0.requests.get();
    dispatch(&h.dom, "click", replacement, ());
    h.settle();
    assert!(
        !handle.is_open(),
        "a descriptive ID without its current mounted binding cannot activate"
    );
    assert_eq!(fixture.0.requests.get(), before);
    dispatch(&h.dom, "mounted", replacement, ());
    dispatch(&h.dom, "click", replacement, ());
    h.settle();
    assert!(
        handle.is_open(),
        "a remounted keyed scope gets its own current click handler"
    );
    assert_eq!(fixture.0.requests.get(), before + 1);
    assert!(
        context.trigger.borrow().is_some(),
        "the remounted control owns the current handler"
    );
    assert!(item_tag(&h.html(), "second").contains(&format!(
        "data-ix-overlay-trigger=\"{}\"",
        context.node_id()
    )));
}

#[test]
fn registered_custom_trigger_paired_key_handler_opens_navigates_and_dismisses() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let fixture = CustomTriggerFixture::default();
    let mut dom = VirtualDom::new_with_props(custom_trigger_fixture, fixture.clone());
    let edits = dom.rebuild_to_vec();
    let mut h = Harness {
        dom,
        config: Rc::new(RefCell::new(Fixture::default())),
        attrs: BTreeMap::new(),
        listeners: BTreeMap::new(),
    };
    h.collect(edits);
    for target in h.listeners.get("mounted").cloned().unwrap_or_default() {
        dispatch(&h.dom, "mounted", target, ());
    }
    h.settle();
    let handle = fixture.0.dropdown.get().unwrap();
    assert!(h.key("second", Key::ArrowDown).0);
    assert!(handle.is_open());
    assert!(h.key("second", Key::End).0);
    let context = handle.context();
    assert_eq!(
        context.entries.borrow()[&context.state.borrow().active.unwrap()].label,
        "Custom child"
    );
    h.key("second", Key::Escape);
    assert!(!handle.is_open());
}

#[test]
fn custom_paired_visibility_uses_current_mount_and_scope_cleanup_releases_its_cache() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let fixture = CustomTriggerFixture::default();
    let mut dom = VirtualDom::new_with_props(custom_trigger_fixture, fixture.clone());
    let edits = dom.rebuild_to_vec();
    let mut h = Harness {
        dom,
        config: Rc::new(RefCell::new(Fixture::default())),
        attrs: BTreeMap::new(),
        listeners: BTreeMap::new(),
    };
    h.collect(edits);
    for target in h.listeners.get("mounted").cloned().unwrap_or_default() {
        dispatch(&h.dom, "mounted", target, ());
    }
    h.settle();
    let handle = fixture.0.dropdown.get().unwrap();
    let context = handle.context();
    h.visible("second", TestVisible::new(true, [0.0, 40.0, 0.0, 100.0]));
    h.dom.in_runtime(|| handle.open());
    h.settle();
    assert!(handle.is_open());
    h.visible("first", TestVisible::new(false, [0.0; 4]));
    assert!(
        handle.is_open(),
        "an older custom control cannot dismiss the current anchor"
    );
    let previous = context.trigger.borrow().as_ref().unwrap().node.clone();
    mount_trigger_mock(&mut h, "second");
    assert!(
        context
            .trigger
            .borrow()
            .as_ref()
            .unwrap()
            .visibility
            .is_none(),
        "replacement starts with its own visibility facts"
    );
    h.dom.in_runtime(|| {
        context.observe_trigger_visibility(
            &previous,
            TriggerVisibility {
                intersecting: false,
                geometry: None,
            },
        )
    });
    h.settle();
    assert!(
        handle.is_open(),
        "old mount geometry cannot affect its replacement"
    );
    h.dom
        .in_runtime(|| fixture.0.first.get().unwrap().set(false));
    h.settle();
    assert!(
        context.trigger.borrow().is_some(),
        "old-scope cleanup preserves the current mounted owner"
    );
    h.visible("second", TestVisible::new(false, [0.0; 4]));
    assert!(!handle.is_open());
    h.dom
        .in_runtime(|| fixture.0.second.get().unwrap().set(false));
    h.settle();
    assert!(context.trigger.borrow().is_none());
    drop(h);
    assert!(!context.state.borrow().alive);
    context.observe_trigger_visibility(
        &previous,
        TriggerVisibility {
            intersecting: false,
            geometry: None,
        },
    );
}

#[test]
fn custom_trigger_scope_cleanup_preserves_newer_registration_and_releases_removed_current_mount() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let fixture = CustomTriggerFixture::default();
    let mut dom = VirtualDom::new_with_props(custom_trigger_fixture, fixture.clone());
    let edits = dom.rebuild_to_vec();
    let mut h = Harness {
        dom,
        config: Rc::new(RefCell::new(Fixture::default())),
        attrs: BTreeMap::new(),
        listeners: BTreeMap::new(),
    };
    h.collect(edits);
    for target in h.listeners.get("mounted").cloned().unwrap_or_default() {
        dispatch(&h.dom, "mounted", target, ());
    }
    h.settle();
    let first = mount_trigger_mock(&mut h, "first");
    let second = mount_trigger_mock(&mut h, "second");
    let context = fixture.0.dropdown.get().unwrap().context();
    let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
    assert!(
        runtime.mounted_focus_target("second").is_some(),
        "the custom trigger hook also registers outside-Tab focus"
    );
    let previous_node = context.trigger.borrow().as_ref().unwrap().node.clone();
    let previous_second = second;
    let second = mount_trigger_mock(&mut h, "second");
    let current_node = context.trigger.borrow().as_ref().unwrap().node.clone();
    assert!(
        !Rc::ptr_eq(&previous_node, &current_node),
        "replacement admits the new mounted handle"
    );
    drop(previous_node);
    assert!(previous_second.calls.borrow().is_empty());
    let current = context.trigger.borrow().clone().unwrap();
    context.mount_trigger(current.node.clone(), current.blocked.clone());
    drop(current);
    h.settle();
    assert!(Rc::ptr_eq(
        &context.trigger.borrow().as_ref().unwrap().node,
        &current_node
    ));
    drop(current_node);
    h.dom
        .in_runtime(|| fixture.0.first.get().unwrap().set(false));
    h.settle();
    assert!(
        context.trigger.borrow().is_some(),
        "old-scope cleanup preserves the current mount"
    );
    h.dom.in_runtime(|| {
        assert!(context.request(true, Reason::Trigger, false));
        assert!(context.request(false, Reason::Trigger, true));
    });
    h.settle();
    assert!(first.calls.borrow().is_empty());
    assert_eq!(
        *second.calls.borrow(),
        ["second"],
        "unmounting the earlier binding cannot clear its replacement"
    );
    let target = Rc::downgrade(&context.trigger.borrow().as_ref().unwrap().node);
    h.dom.in_runtime(|| {
        assert!(context.request(true, Reason::Trigger, false));
        assert!(context.request(false, Reason::Trigger, true));
        fixture.0.second.get().unwrap().set(false);
    });
    h.settle();
    assert!(context.trigger.borrow().is_none());
    assert!(
        runtime.mounted_focus_target("second").is_none(),
        "removing the trigger component releases its Tab destination"
    );
    assert!(
        target.upgrade().is_none(),
        "removed custom scope releases its current handle"
    );
    assert_eq!(second.calls.borrow().len(), 1);
    let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
    h.dom.in_runtime(|| {
        assert!(context.request(true, Reason::Trigger, false));
        assert!(context.request(false, Reason::Trigger, true));
    });
    assert_eq!(
        runtime.pending_item_focus_count(),
        0,
        "an unmounted descriptive ID cannot enroll focus"
    );
    h.settle();
    assert_eq!(second.calls.borrow().len(), 1);
}
#[derive(Default)]
struct OpeningQuery {
    scope: Option<ScopeId>,
    reply: Option<serde_json::Value>,
    waiter: Option<std::task::Waker>,
    finished: bool,
}
#[derive(Default)]
struct OpeningDocument {
    owner: dioxus::signals::Owner,
    queries: RefCell<Vec<Rc<RefCell<OpeningQuery>>>>,
}
struct OpeningEvaluator(Rc<RefCell<OpeningQuery>>);
impl dioxus::document::Document for OpeningDocument {
    fn eval(&self, script: String) -> dioxus::document::Eval {
        if !script.contains("?.inventoryReaders?.get(owner)") {
            return dioxus::document::Document::eval(&dioxus::document::NoOpDocument, script);
        }
        let query = Rc::new(RefCell::new(OpeningQuery {
            scope: Some(dioxus::core::current_scope_id()),
            ..Default::default()
        }));
        self.queries.borrow_mut().push(query.clone());
        dioxus::document::Eval::new(
            self.owner
                .insert(Box::new(OpeningEvaluator(query)) as Box<dyn dioxus::document::Evaluator>),
        )
    }
}
impl dioxus::document::Evaluator for OpeningEvaluator {
    fn send(&self, value: serde_json::Value) -> Result<(), dioxus::document::EvalError> {
        if value["kind"] == "finish" {
            self.0.borrow_mut().finished = true;
        }
        Ok(())
    }
    fn poll_recv(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<serde_json::Value, dioxus::document::EvalError>> {
        let mut query = self.0.borrow_mut();
        if let Some(reply) = query.reply.take() {
            std::task::Poll::Ready(Ok(reply))
        } else {
            query.waiter = Some(cx.waker().clone());
            std::task::Poll::Pending
        }
    }
    fn poll_join(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<serde_json::Value, dioxus::document::EvalError>> {
        panic!("the inventory query must receive its acknowledged result");
    }
}
fn opening_reply(h: &mut Harness, query: &Rc<RefCell<OpeningQuery>>, facts: &InventoryFacts) {
    let waiter = {
        let mut query = query.borrow_mut();
        query.reply = Some(serde_json::json!({"value":facts,"error":null}));
        query.waiter.take()
    };
    if let Some(waiter) = waiter {
        waiter.wake();
    }
    h.settle();
}
#[test]
fn opening_uses_registered_first_last_checked_and_custom_controls_in_both_modes() {
    let _guard = crate::test_event_converter_guard();
    for mode in [
        DropdownNavigationMode::ActiveDescendant,
        DropdownNavigationMode::RovingTabindex,
    ] {
        let document = Rc::new(OpeningDocument::default());
        let mut h = Harness::with_document(
            Fixture {
                mode,
                focus_checked: true,
                ..Default::default()
            },
            Some(document.clone()),
        );
        let trigger = mount_trigger_mock(&mut h, "Menu");
        h.click("Menu");
        let context = h.context();
        let mut facts = mixed_inventory_facts();
        facts.hosts[1].checked_attribute = true;
        let extra = |name: &str, disabled: bool, hidden: bool, negative: bool| {
            let mut host = facts.hosts[1].clone();
            host.item = ItemRef(name.into());
            host.capture = name.into();
            host.target = name.into();
            host.registration_key = name.into();
            host.checked_attribute = false;
            host.disabled_attribute = disabled;
            host.hidden_attribute = hidden;
            host.tabindex_attribute = negative.then(|| "-1".into());
            host
        };
        let mut hosts = vec![
            extra("disabled-opening", true, false, false),
            extra("hidden-opening", false, true, false),
            extra("unregistered-opening", false, false, false),
            extra("negative-opening", false, false, true),
        ];
        hosts.extend(facts.hosts.clone());
        facts.hosts = hosts;
        facts.query_order = facts.hosts.iter().map(|host| host.item.clone()).collect();
        facts.navigation_order = facts.query_order.clone();
        let mut registered = facts.clone();
        registered
            .hosts
            .retain(|host| host.target != "unregistered-opening");
        let focuses = mount_inventory(&h, &registered);
        for (last, checked, target) in [
            (
                false,
                false,
                if mode == DropdownNavigationMode::ActiveDescendant {
                    "mixed-category"
                } else {
                    "negative-opening"
                },
            ),
            (true, true, "mixed-native"),
            (false, true, "mixed-menu"),
        ] {
            h.dom
                .in_runtime(|| context.focus_edge_with_checked(last, checked));
            h.settle();
            let query = document.queries.borrow().last().unwrap().clone();
            assert_eq!(
                query.borrow().scope,
                context
                    .node
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .with_owner(dioxus::core::current_scope_id),
                "query belongs to the surface owner"
            );
            opening_reply(&mut h, &query, &facts);
            assert_eq!(focuses.borrow().last().map(String::as_str), Some(target));
        }
        assert!(trigger.calls.borrow().is_empty());
        let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
        for host in &registered.hosts {
            runtime.unregister_item_cue(if host.registration_key.is_empty() {
                &host.capture
            } else {
                &host.registration_key
            });
        }
        h.dom.in_runtime(|| context.focus_edge(false));
        h.settle();
        let query = document.queries.borrow().last().unwrap().clone();
        opening_reply(&mut h, &query, &facts);
        assert_eq!(
            trigger.calls.borrow().len(),
            usize::from(mode == DropdownNavigationMode::ActiveDescendant),
            "no registered candidate uses Dioxus trigger focus in AD mode"
        );
    }
}
#[test]
fn opening_inventory_await_cancels_on_new_intent_close_mode_disabled_and_drop() {
    let _guard = crate::test_event_converter_guard();
    let document = Rc::new(OpeningDocument::default());
    let mut h = Harness::with_document(Fixture::default(), Some(document.clone()));
    h.click("Menu");
    let context = h.context();
    let facts = mixed_inventory_facts();
    let focuses = mount_inventory(&h, &facts);
    h.dom.in_runtime(|| context.focus_edge(false));
    h.settle();
    let first = document.queries.borrow().last().unwrap().clone();
    h.dom.in_runtime(|| context.focus_edge(true));
    h.settle();
    assert!(
        first.borrow().finished,
        "newer opening intent drops the old query channel"
    );
    let latest = document.queries.borrow().last().unwrap().clone();
    opening_reply(&mut h, &first, &facts);
    assert!(focuses.borrow().is_empty());
    opening_reply(&mut h, &latest, &facts);
    assert_eq!(*focuses.borrow(), ["mixed-native"]);
    h.dom.in_runtime(|| context.focus_submenu_entry());
    h.settle();
    let closing = document.queries.borrow().last().unwrap().clone();
    h.dom
        .in_runtime(|| assert!(context.request(false, Reason::Outside, false)));
    h.settle();
    assert!(closing.borrow().finished);
    opening_reply(&mut h, &closing, &facts);
    assert_eq!(focuses.borrow().len(), 1);
    h.dom.in_runtime(|| {
        assert!(context.request(true, Reason::Trigger, false));
        context.focus_edge(false);
    });
    h.settle();
    let old_mode = document.queries.borrow().last().unwrap().clone();
    h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
    h.update();
    assert!(old_mode.borrow().finished);
    opening_reply(&mut h, &old_mode, &facts);
    assert_eq!(focuses.borrow().len(), 1);
    let roving = document.queries.borrow().last().unwrap().clone();
    h.config.borrow_mut().no_focus = true;
    h.update();
    opening_reply(&mut h, &roving, &facts);
    assert_eq!(
        focuses.borrow().len(),
        1,
        "disabled focus handling rejects a query admitted earlier"
    );
    h.config.borrow_mut().no_focus = false;
    h.update();
    h.dom.in_runtime(|| context.focus_edge(false));
    h.settle();
    let dropping = document.queries.borrow().last().unwrap().clone();
    h.config.borrow_mut().visible = false;
    h.update();
    assert!(dropping.borrow().finished);
    opening_reply(&mut h, &dropping, &facts);
    assert_eq!(focuses.borrow().len(), 1);
}
#[test]
fn registered_native_control_keeps_ad_editing_defaults_while_escape_and_roving_still_route() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    h.click("Menu");
    let context = h.context();
    let facts = mixed_inventory_facts();
    let _focuses = mount_inventory(&h, &facts);
    let node = context.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    for (index, key) in ["Home", "End", "ArrowDown", "ArrowLeft", "Enter", " "]
        .into_iter()
        .enumerate()
    {
        assert!(
            !registered_inventory_key_from(
                &h,
                index as u64 + 1,
                key,
                &facts,
                Some("mixed-native".into())
            ),
            "registered form control keeps {key} default"
        );
        h.settle();
        assert!(*context.open.peek());
        assert_eq!(runtime.pending_item_focus_count(), 0);
    }
    h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
    h.update();
    assert!(
        registered_inventory_key(&h, 7, "ArrowDown", &facts),
        "roving mode still owns item arrow navigation"
    );
    h.settle();
    h.dom
        .in_runtime(|| assert!(context.request(false, Reason::Escape, true)));
    h.settle();
    assert!(!*context.open.peek());
}
#[test]
fn two_linked_rows_open_their_current_controllers_after_parent_is_already_open() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::new();
    h.click("Menu");
    let parent = h.context();
    let parent_node = parent.node.borrow().clone().unwrap();
    let runtime = parent_node.runtime().unwrap();
    let linked = h.config.borrow().linked.get().unwrap();
    let explicit = parent
        .entries
        .borrow()
        .values()
        .find(|entry| entry.label == "Explicit")
        .unwrap()
        .submenu
        .unwrap();
    let hosts = parent.entries.borrow().values().map(|entry| {
        let attributes = &h.attrs[&h.target(&entry.label).raw()];
        serde_json::json!({"item":entry.host_id,"capture":entry.host_id,"target":entry.host_id,
            "registration_key":attributes["data-ix-item-key"],"kind":"dropdown",
            "disabled_attribute":entry.disabled,"checked_attribute":entry.checked,
            "linked_trigger":entry.submenu.and_then(DropdownHandle::try_context).map(|context|context.node_id())})
    }).collect::<Vec<_>>();
    let order = hosts
        .iter()
        .map(|host| host["item"].clone())
        .collect::<Vec<_>>();
    let inventory: InventoryFacts =
        serde_json::from_value(serde_json::json!({"items_host":parent.host_id(),
        "hosts":hosts,"query_order":order,"navigation_order":order}))
        .unwrap();
    let mut nodes = vec![
        serde_json::json!({"id":parent_node.id(),"generation":parent_node.generation(),
        "connected":true,"discovery_ready":true,"inventory":inventory,"own_inventory":inventory}),
    ];
    for handle in [linked, explicit] {
        let child = handle.context();
        let node = child.node.borrow().clone().unwrap();
        nodes.push(
            serde_json::json!({"id":node.id(),"generation":node.generation(),"connected":true,
            "discovery_ready":true,"trigger_ancestors":[parent_node.id()]}),
        );
    }
    let transport = h
        .attrs
        .iter()
        .find(|(_, attributes)| {
            attributes.get("data-ix-overlay-transport") == Some(&runtime.owner_id())
        })
        .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
        .unwrap();
    dispatch(
        &h.dom,
        "input",
        transport,
        serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe","nodes":nodes})
            .to_string(),
    );
    h.settle();
    for handle in [linked, explicit] {
        let child_id = handle.context().node_id();
        let host = inventory
            .hosts
            .iter()
            .find(|host| host.linked_trigger.as_deref() == Some(child_id.as_str()))
            .unwrap();
        assert!(
            parent_node.linked_submenu(host) == Some(handle),
            "each row resolves its explicit current handle"
        );
    }
    h.key("Menu", Key::Home);
    h.key("Menu", Key::ArrowDown);
    h.key("Menu", Key::ArrowRight);
    assert!(
        linked.is_open() && !explicit.is_open(),
        "the Linked row opens its own controller"
    );
    h.dom.in_runtime(|| linked.close(false));
    h.settle();
    h.key("Menu", Key::End);
    h.key("Menu", Key::ArrowRight);
    assert!(
        !linked.is_open() && explicit.is_open(),
        "the second row does not reuse the first sibling's controller"
    );
    h.key("Menu", Key::Escape);
    assert!(
        !*parent.open.peek() && !linked.is_open() && !explicit.is_open(),
        "parent dismissal includes children admitted after opening"
    );
}

#[test]
fn external_ad_item_keys_navigate_current_mounts_and_return_to_parent() {
    let _guard = crate::test_event_converter_guard();
    let mut h = Harness::with_settings(Fixture {
        focus_checked: true,
        ..Default::default()
    });
    let parent = h.context();
    let child = h.config.borrow().linked.get().unwrap().context();
    assert!(!parent.state.borrow().external && child.state.borrow().external);
    let inventory = |context: &DropdownContext| {
        let hosts = context
            .entries
            .borrow()
            .values()
            .map(|entry| {
                let attrs = &h.attrs[&h.target(&entry.label).raw()];
                serde_json::json!({"item":entry.host_id,"capture":entry.host_id,
                "target":entry.host_id,"registration_key":attrs["data-ix-item-key"],
                "kind":"dropdown","disabled_attribute":entry.disabled,
                "checked_attribute":entry.checked,"visible_focus":entry.label=="Detached leaf",
                "linked_trigger":entry.submenu.and_then(DropdownHandle::try_context).map(|child|child.node_id())})
            })
            .collect::<Vec<_>>();
        let order = hosts
            .iter()
            .map(|host| host["item"].clone())
            .collect::<Vec<_>>();
        serde_json::from_value::<InventoryFacts>(serde_json::json!({"items_host":context.host_id(),
            "hosts":hosts,"query_order":order,"navigation_order":order}))
        .unwrap()
    };
    let parent_facts = inventory(&parent);
    let child_facts = inventory(&child);
    let parent_focus = mount_inventory(&h, &parent_facts);
    let child_focus = mount_inventory(&h, &child_facts);
    let parent_node = parent.node.borrow().clone().unwrap();
    let child_node = child.node.borrow().clone().unwrap();
    let runtime = parent_node.runtime().unwrap();
    let transport = h
        .attrs
        .iter()
        .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
        .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
        .unwrap();
    dispatch(&h.dom,"input",transport,serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe","nodes":[
        {"id":parent_node.id(),"generation":parent_node.generation(),"connected":true,"discovery_ready":true,
            "root_capture":"parent-root","mode":"active-descendant","inventory":parent_facts,"own_inventory":parent_facts},
        {"id":child_node.id(),"generation":child_node.generation(),"connected":true,"discovery_ready":true,
            "root_capture":"child-root","mode":"active-descendant","trigger_ancestors":[parent_node.id()],"inventory":child_facts}
    ]}).to_string());
    let trigger = parent_facts
        .hosts
        .iter()
        .find(|host| {
            parent
                .entries
                .borrow()
                .values()
                .any(|entry| entry.label == "Linked" && entry.host_id == host.target)
        })
        .unwrap();
    assert!(parent_node.linked_submenu(trigger) == h.config.borrow().linked.get());
    // Current admitted ancestry supplies the registered parent relationship.
    h.dom.in_runtime(|| {
        assert!(parent.request(true, Reason::Trigger, false));
        parent.open_submenu(trigger);
    });
    h.settle();
    assert!(*parent.open.peek() && *child.open.peek());
    assert!(h.key("Detached leaf", Key::ArrowDown).0);
    let next = child
        .entries
        .borrow()
        .values()
        .find(|entry| entry.label == "Checked detached")
        .unwrap()
        .host_id
        .clone();
    assert_eq!(child_focus.borrow().last(), Some(&next));
    let left = h.key("Checked detached", Key::ArrowLeft);
    assert!(
        !left.1,
        "successful child return stops ancestor key handling"
    );
    assert!(!*child.open.peek() && *parent.open.peek());
    assert!(parent.state.borrow().forward.is_none());
    assert_eq!(parent_focus.borrow().last(), Some(&trigger.target));

    let host = &child_facts.hosts[0];
    let binding = ItemHostBinding {
        key: host.registration_key.clone(),
        host_id: host.target.clone(),
        generic: true,
        runtime: runtime.clone(),
        context: Some(child.clone()),
    };
    let event = || {
        KeyboardEvent::new(
            Rc::new(dioxus::html::KeyboardData::new(TestKeyboard(
                Key::ArrowDown,
                Modifiers::empty(),
            ))),
            true,
        )
    };
    h.dom.in_runtime(|| {
        assert!(
            !binding.navigation_key(event()),
            "closed owner does not route"
        )
    });
    h.dom.in_runtime(|| {
        child.request(true, Reason::Trigger, false);
    });
    let unregistered = ItemHostBinding {
        key: "unregistered-control".into(),
        host_id: "unregistered-control".into(),
        ..binding.clone()
    };
    h.dom.in_runtime(|| {
        assert!(
            !unregistered.navigation_key(event()),
            "unregistered controls are not enrolled"
        )
    });
    let mut native_facts = child_facts.clone();
    native_facts.hosts[0].kind = ItemKind::NativeOptIn;
    dispatch(&h.dom,"input",transport,serde_json::json!({"owner":runtime.owner_id(),"sequence":2,"kind":"observe","nodes":[
        {"id":child_node.id(),"generation":child_node.generation(),"connected":true,"discovery_ready":true,
            "root_capture":"child-root","inventory":native_facts}
    ]}).to_string());
    h.dom.in_runtime(|| {
        assert!(
            !binding.navigation_key(event()),
            "registered native controls retain their editing route"
        )
    });
}
