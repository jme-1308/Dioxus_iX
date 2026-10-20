//! Stable-key group selection and retained native disclosure composition.
use crate::Icon;
use dioxus::prelude::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_GROUP: AtomicUsize = AtomicUsize::new(1);

/// Managed group row. IDs must be unique and stable within a group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupEntry {
    pub id: String,
    pub text: String,
    pub secondary_text: Option<String>,
    pub icon: Option<String>,
    pub aria_label: Option<String>,
    pub aria_label_icon: Option<String>,
    pub disabled: bool,
    pub suppress_selection: bool,
}
impl GroupEntry {
    /// Construct an enabled selectable row.
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            secondary_text: None,
            icon: None,
            aria_label: None,
            aria_label_icon: None,
            disabled: false,
            suppress_selection: false,
        }
    }
}
/// A requested disclosure, header selection or managed item selection change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupChange {
    Expanded(bool),
    Selected(bool),
    ItemSelected(Option<String>),
}
/// Cancel synchronously to preserve state and suppress accepted callbacks.
/// Clones share cancellation for this single request.
#[derive(Clone, Debug)]
pub struct GroupChangeRequest {
    pub change: GroupChange,
    prevented: Rc<Cell<bool>>,
}
impl GroupChangeRequest {
    fn new(change: GroupChange) -> Self {
        Self {
            change,
            prevented: Rc::new(Cell::new(false)),
        }
    }
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

/// Collapsible group with optional stable-key managed rows and retained content.
/// `selected_item: None` uses local selection; `Some(None)` is controlled clear.
/// Unknown keys are not displayed; removed/suppressed local keys are silently
/// pruned. Explicit selected disabled entries may display selection, but cannot
/// be activated. Suppressed entries never display a selection cue.
///
/// Each admitted change passes `onrequest` before state and accepted callbacks.
/// An accepted header selection also clears current managed selection atomically.
/// Composed children/controls remain independently owned; custom header content
/// sits beside the native select button. Set `suppress_escape` while a nested
/// consumer-owned popup handles Escape; no DOM or menu discovery is performed.
#[component]
pub fn Group(
    #[props(default)] header: String,
    #[props(default=VNode::empty())] children: Element,
    #[props(default)] subheader: Option<String>,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
    #[props(default)] selected: Option<bool>,
    #[props(default)] onselect: EventHandler<bool>,
    #[props(default)] suppress_header_selection: bool,
    #[props(default)] expand_on_header_click: bool,
    #[props(default)] header_actions: Option<Element>,
    #[props(default)] items: Vec<GroupEntry>,
    #[props(default)] selected_item: Option<Option<String>>,
    #[props(default)] default_selected_item: Option<String>,
    #[props(default)] onitemselect: EventHandler<Option<String>>,
    #[props(default)] onrequest: EventHandler<GroupChangeRequest>,
    #[props(default)] header_content: Option<Element>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] aria_label_select: Option<String>,
    #[props(default)] aria_label_expand: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] suppress_escape: bool,
) -> Element {
    let mut local_open = use_signal(|| default_expanded);
    let mut local_selected = use_signal(|| false);
    let mut local_item = use_signal(|| default_selected_item);
    let open = expanded.unwrap_or(local_open());
    let is_selected = selected.unwrap_or(local_selected());
    let controlled_item = selected_item.is_some();
    let requested_item = selected_item.clone().unwrap_or_else(&*local_item);
    let selected_key = requested_item.filter(|key| {
        items
            .iter()
            .any(|entry| &entry.id == key && !entry.suppress_selection)
    });
    let live_keys: Vec<_> = items
        .iter()
        .filter(|entry| !entry.suppress_selection)
        .map(|entry| entry.id.clone())
        .collect();
    use_effect(use_reactive(
        (&live_keys, &controlled_item),
        move |(keys, controlled)| {
            if !controlled
                && local_item
                    .peek()
                    .as_ref()
                    .is_some_and(|key| !keys.contains(key))
            {
                local_item.set(None);
            }
        },
    ));
    let prefix = use_hook(|| format!("ix-group-{}", NEXT_GROUP.fetch_add(1, Ordering::Relaxed)));
    let header_id = format!("{prefix}-header");
    let expand_id = format!("{prefix}-expand");
    let panel_id = format!("{prefix}-content");
    let mut disclosure = use_signal(|| None::<Rc<MountedData>>);
    let mut pending_escape = use_signal(|| false);
    let current_open = use_hook(|| Rc::new(Cell::new(open)));
    current_open.set(open);
    use_effect(use_reactive((&open,), move |(open,)| {
        if pending_escape() {
            pending_escape.set(false);
            if !open && let Some(node) = disclosure.peek().clone() {
                let current = current_open.clone();
                spawn(async move {
                    if !current.get() {
                        let _ = node.set_focus(true).await;
                    }
                });
            }
        }
    }));
    let current_key = selected_key.clone();
    let request_items = items.clone();
    let request = use_callback(move |change: GroupChange| {
        if let GroupChange::ItemSelected(Some(key)) = &change
            && !request_items
                .iter()
                .any(|entry| &entry.id == key && !entry.disabled && !entry.suppress_selection)
        {
            return false;
        }
        let event = GroupChangeRequest::new(change.clone());
        onrequest.call(event.clone());
        if event.default_prevented() {
            return false;
        }
        match change {
            GroupChange::Expanded(value) => {
                if expanded.is_none() {
                    local_open.set(value);
                }
                onexpand.call(value);
            }
            GroupChange::Selected(value) => {
                if selected.is_none() {
                    local_selected.set(value);
                }
                if current_key.is_some() && !controlled_item {
                    local_item.set(None);
                }
                onselect.call(value);
                if current_key.is_some() {
                    onitemselect.call(None);
                }
            }
            GroupChange::ItemSelected(value) => {
                if !controlled_item {
                    local_item.set(value.clone());
                }
                onitemselect.call(value);
            }
        }
        true
    });
    let select_label = aria_label_select
        .filter(|label| !label.trim().is_empty())
        .unwrap_or_else(|| {
            if header.trim().is_empty() {
                "Select group".into()
            } else {
                header.clone()
            }
        });
    let expand_label = aria_label_expand
        .filter(|label| !label.trim().is_empty())
        .unwrap_or_else(|| {
            format!(
                "{} {}",
                if open { "Collapse" } else { "Expand" },
                if header.trim().is_empty() {
                    "group"
                } else {
                    &header
                }
            )
        });
    let header_label = if suppress_header_selection {
        expand_label.clone()
    } else {
        select_label
    };
    rsx! {section{id,class:"ix-group {class}",
        onkeydown:move|event:KeyboardEvent|{
            if event.key()==Key::Escape&&open&&!suppress_escape&&event.default_action_enabled()
                && request.call(GroupChange::Expanded(false)){
                event.prevent_default();event.stop_propagation();pending_escape.set(true);
            }
        },
        div{class:"ix-group-header-row","data-selected":is_selected,"data-item-selected":selected_key.clone(),
            button{id:header_id.clone(),class:"ix-group-header",r#type:"button","aria-label":header_label,
                "aria-pressed":(!suppress_header_selection).then(||is_selected.to_string()),
                "aria-expanded":(suppress_header_selection||expand_on_header_click).then(||open.to_string()),
                "aria-controls":(suppress_header_selection||expand_on_header_click).then(||panel_id.clone()),
                onclick:move|_|{
                    if !suppress_header_selection{request.call(GroupChange::Selected(!is_selected));}
                    if suppress_header_selection||expand_on_header_click{request.call(GroupChange::Expanded(!open));}
                },
                span{strong{"{header}"}if let Some(text)=subheader{small{"{text}"}}}
            }
            if let Some(content)=header_content{div{class:"ix-group-header-content",{content}}}
            {header_actions}
            button{id:expand_id,class:"ix-group-expand",r#type:"button","aria-label":expand_label,"aria-expanded":open,"aria-controls":panel_id.clone(),
                onmounted:move|event|disclosure.set(Some(event.data())),onclick:move|_|{request.call(GroupChange::Expanded(!open));},
                span{class:"ix-chevron","data-open":open,Icon{name:"chevron-down"}}
            }
        }
        div{id:panel_id,class:"ix-group-content",role:"region","aria-labelledby":header_id,hidden:!open,
            for entry in items {
                div{key:"{entry.id}",class:"ix-group-entry","data-group-item":entry.id.clone(),
                    GroupItem{text:entry.text,secondary_text:entry.secondary_text,icon:entry.icon,aria_label:entry.aria_label,aria_label_icon:entry.aria_label_icon,
                        selected:selected_key.as_ref()==Some(&entry.id),disabled:entry.disabled,suppress_selection:entry.suppress_selection,
                        onselect:{let key=entry.id.clone();let current=selected_key.clone();move|_|{request.call(GroupChange::ItemSelected(if current.as_ref()==Some(&key){None}else{Some(key.clone())}));}}
                    }
                }
            }
            {children}
            if let Some(footer)=footer{div{class:"ix-group-footer",{footer}}}
        }
    }}
}

/// Native selectable row, or a nonselectable custom-content row when `children`
/// is supplied. Custom controls own their handlers and disabled state. Suppressed
/// standard rows remain focusable but have no pressed state, cue or callback.
#[component]
pub fn GroupItem(
    #[props(default)] text: String,
    #[props(default)] children: Option<Element>,
    #[props(default)] secondary_text: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] selected: bool,
    #[props(default)] disabled: bool,
    #[props(default)] onselect: EventHandler<()>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] aria_label_icon: Option<String>,
    #[props(default)] suppress_selection: bool,
) -> Element {
    if let Some(children) = children {
        return rsx! {div{id,class:"ix-group-item ix-group-custom-item {class}","aria-label":aria_label,{children}}};
    }
    rsx! {button{id,class:"ix-group-item {class}",r#type:"button",disabled,
        "aria-label":aria_label,"aria-pressed":(!suppress_selection).then(||selected.to_string()),"data-selected":selected&&!suppress_selection,
        onclick:move|_|{if !disabled&&!suppress_selection{onselect.call(());}},
        if let Some(icon)=icon{Icon{name:icon,label:aria_label_icon}}span{"{text}"}
        if let Some(text)=secondary_text{span{class:"ix-group-secondary","{text}"}}
    }}
}

#[cfg(test)]
mod tests {
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
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
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
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => current = id,
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
}
