use super::*;
use std::{cell::RefCell, collections::BTreeMap};
#[derive(Clone)]
struct Config {
    items: Vec<GroupEntry>,
    item: Option<Option<String>>,
    header: Option<bool>,
    open: Option<bool>,
    suppress_header: bool,
    expand_header: bool,
    suppress_escape: bool,
}
impl Default for Config {
    fn default() -> Self {
        let mut items: Vec<_> = [
            ("motor", "Motor"),
            ("pump", "Pump"),
            ("archive", "Archived"),
            ("suppressed", "Suppressed"),
        ]
        .into_iter()
        .map(|(key, text)| {
            let mut entry = GroupEntry::new(key, text);
            entry.aria_label = Some(format!("Select {text}"));
            entry
        })
        .collect();
        items[2].disabled = true;
        items[3].suppress_selection = true;
        Self {
            items,
            item: None,
            header: None,
            open: None,
            suppress_header: false,
            expand_header: false,
            suppress_escape: false,
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    requests: RefCell<Vec<GroupChange>>,
    accepted: RefCell<Vec<GroupChange>>,
    veto: Cell<bool>,
    veto_header: Cell<bool>,
    actions: Cell<usize>,
    mounts: Cell<usize>,
}
impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn Preparation(data: Rc<Fixture>) -> Element {
    let mut note = use_signal(|| String::from("Prepare inspection"));
    rsx! {input{id:String::from("group-note"),value:note(),oninput:move|event|note.set(event.value()),onmounted:move|_|data.mounts.set(data.mounts.get()+1)}}
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let request = data.clone();
    let expand = data.clone();
    let select = data.clone();
    let item = data.clone();
    let action = data.clone();
    rsx! {Group{id:"managed-group",header:"Inspection",subheader:"Local assets",items:config.items,selected_item:config.item,selected:config.header,expanded:config.open,default_expanded:true,
        suppress_header_selection:config.suppress_header,expand_on_header_click:config.expand_header,suppress_escape:config.suppress_escape,
        aria_label_select:"Select inspections",aria_label_expand:"Toggle assets",
        onrequest:move|event:GroupChangeRequest|{
            request.requests.borrow_mut().push(event.change.clone());
            if request.veto.get()||request.veto_header.get()&&matches!(event.change,GroupChange::Selected(_)){
                event.clone().prevent_default();assert!(event.default_prevented());
            }
        },
        onexpand:move|value|expand.accepted.borrow_mut().push(GroupChange::Expanded(value)),
        onselect:move|value|select.accepted.borrow_mut().push(GroupChange::Selected(value)),
        onitemselect:move|value|item.accepted.borrow_mut().push(GroupChange::ItemSelected(value)),
        header_content:rsx!{span{"4 assets"}},
        header_actions:rsx!{button{id:String::from("group-action"),r#type:"button",onclick:move|_|action.actions.set(action.actions.get()+1),"Inspect"}},
        footer:rsx!{Preparation{data:data.clone()}},
        GroupItem{children:rsx!{button{id:String::from("handled-escape"),r#type:"button",onkeydown:move|event:KeyboardEvent|if event.key()==Key::Escape{event.prevent_default();},"Inner control"}}}
    }}
}
type FocusLog = Rc<RefCell<Vec<String>>>;
#[derive(Clone)]
struct FocusMount {
    label: String,
    calls: FocusLog,
}
impl dioxus::html::RenderedElementBacking for FocusMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        self.calls.borrow_mut().push(self.label.clone());
        Box::pin(async { Ok(()) })
    }
}
#[derive(Clone)]
struct RawKey(Key);
impl dioxus::html::ModifiersInteraction for RawKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for RawKey {
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
struct RawForm(String);
impl dioxus::html::HasFileData for RawForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for RawForm {
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
struct Mouse;
impl dioxus::html::InteractionLocation for Mouse {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for Mouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for Mouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for Mouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for Mouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data { let _=event;converter!(@data $group,event) })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
        (@data Mounted,$event:ident) => { MountedData::new($event.downcast::<FocusMount>().unwrap().clone()) };
        (@data Keyboard,$event:ident) => { dioxus::html::KeyboardData::new($event.downcast::<RawKey>().unwrap().clone()) };
        (@data Form,$event:ident) => { dioxus::html::FormData::new(RawForm($event.downcast::<String>().unwrap().clone())) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected group test event") };
    }
dioxus::html::with_html_event_groups!(converter);
#[derive(Default)]
struct Targets {
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    mounted: Vec<dioxus::core::ElementId>,
}
impl Targets {
    fn collect(&mut self, changes: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in changes.edits {
            match edit {
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
                dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                    self.mounted.push(current)
                }
                _ => {}
            }
        }
    }
    fn attr(&self, name: &str, value: &str) -> dioxus::core::ElementId {
        dioxus::core::ElementId::from_raw(
            self.attrs
                .iter()
                .find_map(|(id, attrs)| {
                    (attrs.get(name).map(String::as_str) == Some(value)).then_some(*id)
                })
                .unwrap_or_else(|| panic!("Missing {name}={value}")),
        )
    }
}
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    data: Box<dyn std::any::Any>,
) -> bool {
    let event = Event::new(
        Rc::new(dioxus::html::PlatformEventData::new(data)) as Rc<dyn std::any::Any>,
        name != "mounted",
    );
    dom.runtime().handle_event(name, event.clone(), id);
    event.default_action_enabled()
}
fn settle(dom: &mut VirtualDom, targets: &mut Targets) {
    for _ in 0..4 {
        targets.collect(dom.render_immediate_to_vec());
        dom.process_events();
    }
}
fn refresh(dom: &mut VirtualDom, targets: &mut Targets) {
    dom.mark_dirty(ScopeId::APP);
    settle(dom, targets);
}

fn setup(data: Rc<Fixture>) -> (VirtualDom, Targets, FocusLog) {
    let mut dom = VirtualDom::new_with_props(fixture, data);
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    let calls = Rc::new(RefCell::new(Vec::new()));
    for id in targets.mounted.clone() {
        let label = targets
            .attrs
            .get(&id.raw())
            .and_then(|attrs| attrs.get("aria-label"))
            .cloned()
            .unwrap_or_default();
        dispatch(
            &dom,
            "mounted",
            id,
            Box::new(FocusMount {
                label,
                calls: calls.clone(),
            }),
        );
    }
    settle(&mut dom, &mut targets);
    (dom, targets, calls)
}
fn tag<'a>(html: &'a str, name: &str, value: &str) -> &'a str {
    html.split('<')
        .map(|tag| tag.split('>').next().unwrap())
        .find(|tag| tag.contains(&format!("{name}=\"{value}\"")))
        .unwrap_or_else(|| panic!("Missing {name}={value}: {html}"))
}
fn attribute<'a>(tag: &'a str, name: &str) -> &'a str {
    tag.split(&format!("{name}=\""))
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
}
fn true_attribute(tag: &str, name: &str) -> bool {
    tag.trim_end_matches('/')
        .split_ascii_whitespace()
        .any(|attribute| {
            attribute == name
                || attribute == format!("{name}=true")
                || attribute == format!("{name}=\"true\"")
        })
}
fn selected(dom: &VirtualDom, label: &str) -> bool {
    true_attribute(
        tag(&dioxus_ssr::render(dom), "aria-label", label),
        "aria-pressed",
    )
}
fn open(dom: &VirtualDom) -> bool {
    let html = dioxus_ssr::render(dom);
    let root = html
        .split('<')
        .map(|tag| tag.split('>').next().unwrap())
        .find(|tag| tag.starts_with("button ") && tag.contains("class=\"ix-group-expand\""))
        .unwrap();
    true_attribute(root, "aria-expanded")
}
fn click(dom: &mut VirtualDom, targets: &mut Targets, label: &str) {
    dispatch(
        dom,
        "click",
        targets.attr("aria-label", label),
        Box::new(()),
    );
    settle(dom, targets);
}
#[test]
fn groups_managed_selection_toggles_clears_and_follows_keys_through_reorder_and_removal() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets, _) = setup(data.clone());
    click(&mut dom, &mut targets, "Select Motor");
    assert!(selected(&dom, "Select Motor"));
    click(&mut dom, &mut targets, "Select Motor");
    assert!(!selected(&dom, "Select Motor"));
    click(&mut dom, &mut targets, "Select Pump");
    let count = data.requests.borrow().len();
    click(&mut dom, &mut targets, "Select Archived");
    click(&mut dom, &mut targets, "Select Suppressed");
    assert_eq!(data.requests.borrow().len(), count);
    assert!(selected(&dom, "Select Pump"));
    data.config.borrow_mut().items.reverse();
    refresh(&mut dom, &mut targets);
    assert!(selected(&dom, "Select Pump"));
    click(&mut dom, &mut targets, "Select inspections");
    assert!(selected(&dom, "Select inspections"));
    assert!(!selected(&dom, "Select Pump"));
    assert_eq!(
        data.accepted.borrow().as_slice(),
        [
            GroupChange::ItemSelected(Some("motor".into())),
            GroupChange::ItemSelected(None),
            GroupChange::ItemSelected(Some("pump".into())),
            GroupChange::Selected(true),
            GroupChange::ItemSelected(None)
        ]
    );
    click(&mut dom, &mut targets, "Select Pump");
    assert!(
        selected(&dom, "Select inspections"),
        "row choice does not invent header selection"
    );
    let count = data.accepted.borrow().len();
    data.config
        .borrow_mut()
        .items
        .retain(|entry| entry.id != "pump");
    refresh(&mut dom, &mut targets);
    assert!(
        !tag(&dioxus_ssr::render(&dom), "class", "ix-group-header-row")
            .contains("data-item-selected")
    );
    assert_eq!(data.accepted.borrow().len(), count);
    let mut pump = GroupEntry::new("pump", "Pump");
    pump.aria_label = Some("Select Pump".into());
    data.config.borrow_mut().items.push(pump);
    refresh(&mut dom, &mut targets);
    assert!(!selected(&dom, "Select Pump"));
    assert_eq!(data.accepted.borrow().len(), count);
}
#[test]
fn groups_controlled_held_and_vetoed_requests_keep_authoritative_selection() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().item = Some(None);
    data.config.borrow_mut().header = Some(false);
    let (mut dom, mut targets, _) = setup(data.clone());
    click(&mut dom, &mut targets, "Select Motor");
    assert!(!selected(&dom, "Select Motor"));
    assert_eq!(
        data.accepted.borrow().as_slice(),
        [GroupChange::ItemSelected(Some("motor".into()))]
    );
    data.veto.set(true);
    click(&mut dom, &mut targets, "Select Pump");
    assert_eq!(data.accepted.borrow().len(), 1);
    assert_eq!(data.requests.borrow().len(), 2);
    data.veto.set(false);
    data.config.borrow_mut().item = Some(Some("motor".into()));
    refresh(&mut dom, &mut targets);
    assert!(selected(&dom, "Select Motor"));
    click(&mut dom, &mut targets, "Select inspections");
    assert!(!selected(&dom, "Select inspections"));
    assert!(selected(&dom, "Select Motor"));
    assert_eq!(
        &data.accepted.borrow()[1..],
        [GroupChange::Selected(true), GroupChange::ItemSelected(None)]
    );
    data.veto.set(true);
    let count = data.accepted.borrow().len();
    click(&mut dom, &mut targets, "Select inspections");
    assert_eq!(data.accepted.borrow().len(), count);
    assert!(selected(&dom, "Select Motor"));
    data.veto.set(false);
    data.config.borrow_mut().item = Some(Some("archive".into()));
    refresh(&mut dom, &mut targets);
    assert!(
        selected(&dom, "Select Archived"),
        "explicit disabled selection may display"
    );
    data.config.borrow_mut().item = Some(Some("missing".into()));
    refresh(&mut dom, &mut targets);
    assert!(
        !tag(&dioxus_ssr::render(&dom), "class", "ix-group-header-row")
            .contains("data-item-selected")
    );
    assert_eq!(data.accepted.borrow().len(), count);
    data.config.borrow_mut().expand_header = true;
    data.veto_header.set(true);
    refresh(&mut dom, &mut targets);
    click(&mut dom, &mut targets, "Select inspections");
    assert!(!open(&dom));
    assert_eq!(
        data.accepted.borrow().last(),
        Some(&GroupChange::Expanded(false)),
        "combined header changes are independently gated"
    );
}
#[test]
fn groups_escape_focus_requires_actual_close_and_preserves_retained_footer_and_aria() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().open = Some(true);
    let (mut dom, mut targets, calls) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    let toggle = tag(&html, "aria-label", "Toggle assets");
    let panel = attribute(toggle, "aria-controls").to_string();
    let header = tag(&html, "aria-label", "Select inspections");
    assert_eq!(
        attribute(tag(&html, "id", &panel), "aria-labelledby"),
        attribute(header, "id")
    );
    assert!(tag(&html, "id", &panel).contains("role=\"region\""));
    dispatch(
        &dom,
        "input",
        targets.attr("id", "group-note"),
        Box::new(String::from("Tools staged")),
    );
    dispatch(
        &dom,
        "click",
        targets.attr("id", "group-action"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    assert_eq!(data.actions.get(), 1);
    assert!(data.requests.borrow().is_empty());
    let root = targets.attr("id", "managed-group");
    dispatch(&dom, "keydown", root, Box::new(RawKey(Key::Escape)));
    settle(&mut dom, &mut targets);
    assert!(open(&dom));
    assert!(calls.borrow().is_empty());
    assert_eq!(
        data.accepted.borrow().last(),
        Some(&GroupChange::Expanded(false))
    );
    data.config.borrow_mut().open = Some(false);
    refresh(&mut dom, &mut targets);
    assert!(
        calls.borrow().is_empty(),
        "held Escape must not retain a future focus intent"
    );
    data.config.borrow_mut().open = Some(true);
    refresh(&mut dom, &mut targets);
    data.veto.set(true);
    let count = data.accepted.borrow().len();
    dispatch(&dom, "keydown", root, Box::new(RawKey(Key::Escape)));
    settle(&mut dom, &mut targets);
    assert!(open(&dom));
    assert_eq!(data.accepted.borrow().len(), count);
    assert!(calls.borrow().is_empty());
    data.veto.set(false);
    let count = data.requests.borrow().len();
    data.config.borrow_mut().suppress_escape = true;
    refresh(&mut dom, &mut targets);
    dispatch(&dom, "keydown", root, Box::new(RawKey(Key::Escape)));
    settle(&mut dom, &mut targets);
    assert_eq!(data.requests.borrow().len(), count);
    data.config.borrow_mut().suppress_escape = false;
    refresh(&mut dom, &mut targets);
    dispatch(
        &dom,
        "keydown",
        targets.attr("id", "handled-escape"),
        Box::new(RawKey(Key::Escape)),
    );
    settle(&mut dom, &mut targets);
    assert_eq!(data.requests.borrow().len(), count);
    assert!(open(&dom));
    data.config.borrow_mut().open = None;
    refresh(&mut dom, &mut targets);
    dispatch(&dom, "keydown", root, Box::new(RawKey(Key::Escape)));
    settle(&mut dom, &mut targets);
    assert!(!open(&dom));
    assert_eq!(calls.borrow().as_slice(), ["Toggle assets"]);
    assert!(tag(&dioxus_ssr::render(&dom), "id", &panel).contains("hidden"));
    click(&mut dom, &mut targets, "Toggle assets");
    assert!(open(&dom));
    let html = dioxus_ssr::render(&dom);
    assert!(tag(&html, "id", "group-note").contains("value=\"Tools staged\""));
    assert_eq!(data.mounts.get(), 1);
    assert_eq!(
        attribute(tag(&html, "aria-label", "Toggle assets"), "aria-controls"),
        panel
    );
}
#[test]
fn groups_legacy_custom_rows_and_disclosure_header_mode_keep_native_ownership() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().suppress_header = true;
    let (mut dom, mut targets, _) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    let header = html
        .split('<')
        .map(|tag| tag.split('>').next().unwrap())
        .find(|tag| tag.contains("class=\"ix-group-header\""))
        .unwrap();
    assert!(header.contains("aria-label=\"Toggle assets\""));
    assert!(!header.contains("aria-pressed"));
    let header_id = attribute(header, "id").to_string();
    dispatch(&dom, "click", targets.attr("id", &header_id), Box::new(()));
    settle(&mut dom, &mut targets);
    assert!(!open(&dom));
    assert_eq!(
        data.accepted.borrow().as_slice(),
        [GroupChange::Expanded(false)]
    );
    let mut legacy = VirtualDom::new(
        || rsx! {crate::layout::Group{crate::layout::GroupItem{text:"Manual",selected:true,icon:"check",aria_label_icon:"Reviewed"}crate::GroupItem{suppress_selection:true,selected:true,text:"Suppressed",aria_label:"Suppressed row"}crate::GroupItem{children:rsx!{button{"Custom action"}}}}},
    );
    legacy.rebuild_in_place();
    let html = dioxus_ssr::render(&legacy);
    assert!(html.contains("aria-label=\"Select group\""));
    assert!(html.contains("aria-label=\"Expand group\""));
    assert!(html.contains("aria-label=\"Reviewed\""));
    assert!(html.contains("ix-group-custom-item"));
    assert!(!tag(&html, "aria-label", "Suppressed row").contains("aria-pressed"));
    assert!(!true_attribute(
        tag(&html, "aria-label", "Suppressed row"),
        "data-selected"
    ));
    let mut depth = 0;
    for tag in html.split('<').skip(1) {
        if tag.starts_with("button ") || tag.starts_with("button>") {
            depth += 1;
            assert_eq!(depth, 1, "custom rows never nest native controls");
        } else if tag.starts_with("/button>") {
            depth -= 1;
        }
    }
    assert_eq!(depth, 0);
}

#[derive(Default)]
struct ComposedItemFixture {
    selected: Cell<bool>,
    disabled: Cell<bool>,
    suppressed: Cell<bool>,
    hold: Cell<bool>,
    requests: Cell<usize>,
    actions: Cell<usize>,
    mounts: Cell<usize>,
    label: RefCell<Option<Signal<String>>>,
}
impl PartialEq for ComposedItemFixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn ComposedItemLabel(data: Rc<ComposedItemFixture>) -> Element {
    use_hook(|| data.mounts.set(data.mounts.get() + 1));
    let label = use_signal(|| String::from("Motor 001"));
    *data.label.borrow_mut() = Some(label);
    rsx! {span{id:String::from("composed-item-label"),"{label()}"}}
}
fn composed_item_fixture(data: Rc<ComposedItemFixture>) -> Element {
    let request = data.clone();
    let action = data.clone();
    let legacy_request = data.clone();
    let mut note = use_signal(|| String::from("Prepare tools"));
    rsx! {
        GroupItem {id:"composed-item",aria_label:"Select composed asset",text:"Source text",icon:"check",secondary_text:"Details",
            selectable_children:true,selected:data.selected.get(),disabled:data.disabled.get(),suppress_selection:data.suppressed.get(),
            onselect:move|_|{
                request.requests.set(request.requests.get()+1);
                if !request.hold.get(){request.selected.set(!request.selected.get());}
            },
            ComposedItemLabel{data:data.clone()}
        }
        GroupItem {id:"independent-item",aria_label:"Independent controls",selected:true,disabled:true,suppress_selection:true,
            onselect:move|_|legacy_request.requests.set(legacy_request.requests.get()+1),
            input{id:String::from("independent-item-note"),value:note(),oninput:move|event|note.set(event.value())}
            button{id:String::from("independent-item-action"),r#type:"button",onclick:move|_|action.actions.set(action.actions.get()+1),"Save note"}
        }
        GroupItem {id:"plain-item",text:"Plain row",selected:true}
        GroupItem {id:"plain-opt-in",text:"Plain opt-in",selectable_children:true}
        GroupItem {id:"only-composed-item",selectable_children:true,span{"Only composed label"}}
    }
}
fn composed_item_setup(data: Rc<ComposedItemFixture>) -> (VirtualDom, Targets) {
    let mut dom = VirtualDom::new_with_props(composed_item_fixture, data);
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    settle(&mut dom, &mut targets);
    (dom, targets)
}
fn assert_composed_label_retained(dom: &VirtualDom, data: &ComposedItemFixture) {
    assert!(dioxus_ssr::render(dom).contains("Motor 001 · inspected"));
    assert_eq!(
        data.mounts.get(),
        1,
        "controlled row props retain the composed label scope"
    );
}

#[test]
fn group_item_composed_labels_use_controlled_native_selection_and_retain_local_state() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(ComposedItemFixture::default());
    data.hold.set(true);
    let (mut dom, mut targets) = composed_item_setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    let start = html.find("id=\"composed-item\"").unwrap();
    let body = &html[start..start + html[start..].find("</button>").unwrap()];
    assert!(tag(&html, "id", "composed-item").starts_with("button "));
    assert!(body.find("ix-icon").unwrap() < body.find("Source text").unwrap());
    assert!(body.find("Source text").unwrap() < body.find("Details").unwrap());
    assert!(body.find("Details").unwrap() < body.find("Motor 001").unwrap());
    assert_eq!(
        attribute(tag(&html, "id", "composed-item"), "type"),
        "button"
    );
    dom.in_scope(ScopeId::APP, || {
        data.label
            .borrow()
            .unwrap()
            .set("Motor 001 · inspected".into())
    });
    settle(&mut dom, &mut targets);
    click(&mut dom, &mut targets, "Select composed asset");
    assert_eq!(data.requests.get(), 1);
    assert!(
        !selected(&dom, "Select composed asset"),
        "the caller can hold authoritative selection"
    );
    assert_composed_label_retained(&dom, &data);
    data.hold.set(false);
    click(&mut dom, &mut targets, "Select composed asset");
    refresh(&mut dom, &mut targets);
    assert_eq!(
        data.requests.get(),
        2,
        "one request per accepted activation"
    );
    assert!(selected(&dom, "Select composed asset"));
    data.selected.set(false);
    refresh(&mut dom, &mut targets);
    assert!(!selected(&dom, "Select composed asset"));
    assert_eq!(data.requests.get(), 2, "owner updates do not emit requests");
    assert_composed_label_retained(&dom, &data);
}

#[test]
fn group_item_composed_disabled_and_suppressed_guards_preserve_native_key_defaults() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(ComposedItemFixture::default());
    let (mut dom, mut targets) = composed_item_setup(data.clone());
    dom.in_scope(ScopeId::APP, || {
        data.label
            .borrow()
            .unwrap()
            .set("Motor 001 · inspected".into())
    });
    data.disabled.set(true);
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(true_attribute(
        tag(&html, "id", "composed-item"),
        "disabled"
    ));
    click(&mut dom, &mut targets, "Select composed asset");
    assert_eq!(
        data.requests.get(),
        0,
        "synthetic clicks still honor the native disabled row guard"
    );
    data.disabled.set(false);
    data.suppressed.set(true);
    data.selected.set(true);
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    let row = tag(&html, "id", "composed-item");
    assert!(!true_attribute(row, "disabled"));
    assert!(!row.contains("aria-pressed"));
    assert!(!true_attribute(row, "data-selected"));
    assert!(
        !row.contains("tabindex"),
        "suppressed native button keeps its ordinary focusability"
    );
    let id = targets.attr("id", "composed-item");
    for key in [Key::Enter, Key::Character(" ".into())] {
        assert!(
            dispatch(&dom, "keydown", id, Box::new(RawKey(key))),
            "native button activation is not intercepted"
        );
    }
    click(&mut dom, &mut targets, "Select composed asset");
    assert_eq!(data.requests.get(), 0);
    assert_composed_label_retained(&dom, &data);
    data.suppressed.set(false);
    refresh(&mut dom, &mut targets);
    assert!(selected(&dom, "Select composed asset"));
    click(&mut dom, &mut targets, "Select composed asset");
    refresh(&mut dom, &mut targets);
    assert_eq!(data.requests.get(), 1);
    assert!(!selected(&dom, "Select composed asset"));
    assert_composed_label_retained(&dom, &data);
}

#[test]
fn group_item_default_custom_controls_stay_independent_and_plain_rows_keep_their_anatomy() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(ComposedItemFixture::default());
    let (mut dom, mut targets) = composed_item_setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    let legacy = tag(&html, "id", "independent-item");
    assert!(legacy.starts_with("div "));
    assert!(legacy.contains("ix-group-custom-item"));
    assert!(!legacy.contains("aria-pressed"));
    assert!(
        !legacy.contains("disabled"),
        "legacy arbitrary controls keep consumer-owned disabled behavior"
    );
    assert!(tag(&html, "id", "plain-item").starts_with("button "));
    assert!(true_attribute(
        tag(&html, "id", "plain-item"),
        "aria-pressed"
    ));
    assert!(html.contains("<span>Plain row</span>"));
    assert!(html.contains("<span>Plain opt-in</span>"));
    let only = html.find("id=\"only-composed-item\"").unwrap();
    let only_body = &html[only..only + html[only..].find("</button>").unwrap()];
    assert!(
        !only_body.contains("<span></span>"),
        "content-only labels do not create an empty flex item"
    );
    let mut depth = 0;
    for tag in html.split('<').skip(1) {
        if tag.starts_with("button ") || tag.starts_with("button>") {
            depth += 1;
            assert_eq!(
                depth, 1,
                "independent native controls must never be nested in the selectable button"
            );
        } else if tag.starts_with("/button>") {
            depth -= 1;
        }
    }
    assert_eq!(depth, 0);
    dispatch(
        &dom,
        "input",
        targets.attr("id", "independent-item-note"),
        Box::new(String::from("Tools staged")),
    );
    dispatch(
        &dom,
        "click",
        targets.attr("id", "independent-item-action"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    assert_eq!(data.actions.get(), 1);
    assert_eq!(
        data.requests.get(),
        0,
        "independent controls do not invoke row selection"
    );
    assert_eq!(
        attribute(
            tag(&dioxus_ssr::render(&dom), "id", "independent-item-note"),
            "value"
        ),
        "Tools staged"
    );
}
