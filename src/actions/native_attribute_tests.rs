//! Native metadata composition and component-owned action state through actual VDOM handlers.
use super::*;
use std::cell::Cell;
use std::collections::BTreeMap;

const IDS: [&str; 6] = [
    "native-action",
    "anchor-action",
    "icon-action",
    "toggle-action",
    "icon-toggle-action",
    "link-action",
];
fn tag<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .filter(|part| part.starts_with("button ") || part.starts_with("a "))
        .find(|part| attribute(part.split('>').next().unwrap(), "id") == Some(id))
        .unwrap_or_else(|| panic!("missing native action {id}"))
        .split('>')
        .next()
        .unwrap()
}
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        value.split_once('"').map(|(value, _)| value)
    } else {
        value.split_whitespace().next()
    }
}
#[test]
fn global_action_attributes_compile_and_render_on_all_six_native_paths() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Button {id:IDS[0],aria_details:"help",title:"Save work","data-action":"save","Save"}
            Button {id:IDS[1],href:"#manual",aria_details:"help",title:"Read manual","data-action":"manual","Manual"}
            IconButton {id:IDS[2],icon:"check",label:"Confirm",aria_details:"help",title:"Ignored raw title","data-action":"confirm"}
            ToggleButton {id:IDS[3],aria_details:"help",title:"Pin work","data-action":"pin",onpressedchange:move |_|{},"Pin"}
            IconToggleButton {id:IDS[4],icon:"star",label:"Favorite",aria_details:"help",title:"Ignored raw title","data-action":"favorite",onpressedchange:move |_|{}}
            LinkButton {id:IDS[5],url:"#reference",aria_details:"help",title:"Read reference","data-action":"reference","Reference"}
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    for (id, metadata) in
        IDS.into_iter()
            .zip(["save", "manual", "confirm", "pin", "favorite", "reference"])
    {
        let control = tag(&html, id);
        assert_eq!(attribute(control, "aria-details"), Some("help"));
        assert_eq!(attribute(control, "data-action"), Some(metadata));
        assert_eq!(attribute(control, "aria-disabled"), Some("false"));
        assert!(attribute(control, "class").unwrap().starts_with("ix-"));
    }
    assert_eq!(attribute(tag(&html, IDS[0]), "title"), Some("Save work"));
    assert_eq!(attribute(tag(&html, IDS[2]), "title"), Some("Confirm"));
    assert_eq!(attribute(tag(&html, IDS[4]), "title"), Some("Favorite"));
    assert_eq!(attribute(tag(&html, IDS[1]), "href"), Some("#manual"));
    assert_eq!(attribute(tag(&html, IDS[5]), "href"), Some("#reference"));
    assert_eq!(attribute(tag(&html, IDS[3]), "aria-pressed"), Some("false"));
    assert_eq!(attribute(tag(&html, IDS[4]), "aria-pressed"), Some("false"));
}

#[derive(Clone, Default)]
struct Config {
    metadata: Option<String>,
    attack: bool,
    disabled: bool,
    loading: bool,
    inert: bool,
    pressed: bool,
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    calls: [Cell<usize>; 6],
    requests: [Cell<Option<bool>>; 2],
    raw_calls: Cell<usize>,
    raw_mounts: Cell<usize>,
    typed_mounts: Cell<usize>,
    focus_calls: Cell<usize>,
    typed_mouse: Cell<usize>,
    child_mounts: Cell<usize>,
    child_drops: Cell<usize>,
    private_text: Cell<Option<Signal<String>>>,
}
impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn LocalLabel(data: Rc<Fixture>) -> Element {
    use_hook(|| data.child_mounts.set(data.child_mounts.get() + 1));
    use_drop({
        let data = data.clone();
        move || data.child_drops.set(data.child_drops.get() + 1)
    });
    let draft = use_signal(|| String::from("Private action label"));
    data.private_text.set(Some(draft));
    rsx! {span {"{draft}"}}
}
fn metadata(data: Rc<Fixture>, index: usize) -> Vec<Attribute> {
    let config = data.config.borrow().clone();
    let focus = data.clone();
    let mut attrs = vec![Attribute::new(
        "onfocus",
        dioxus::html::EventHandlerValue::into_platform_listener(EventHandler::new(
            move |_: FocusEvent| focus.focus_calls.set(focus.focus_calls.get() + 1),
        )),
        None,
        false,
    )];
    if index >= 3 {
        attrs.push(Attribute::new("id", IDS[index], None, false));
    }
    if let Some(value) = config.metadata {
        attrs.push(Attribute::new("aria-details", value.clone(), None, false));
        attrs.push(Attribute::new("data-live", value, None, false));
    }
    if config.attack {
        for (name, value) in [
            ("class", "rogue-class"),
            ("disabled", "false"),
            ("tabindex", "99"),
            ("type", "reset"),
            ("aria-disabled", "false"),
        ] {
            attrs.push(Attribute::new(name, value, None, false));
        }
        if index < 3 {
            attrs.push(Attribute::new("id", "rogue-id", None, false));
        }
        if index != 5 {
            for (name, value) in [
                ("aria-label", "rogue name"),
                ("aria-busy", "false"),
                ("aria-pressed", "true"),
            ] {
                attrs.push(Attribute::new(name, value, None, false));
            }
        }
        if index <= 1 {
            for (name, value) in [
                ("href", "#rogue"),
                ("target", "_blank"),
                ("rel", "opener"),
                ("form", "rogue-form"),
                ("inert", "false"),
                ("aria-describedby", "rogue-help"),
            ] {
                attrs.push(Attribute::new(name, value, None, false));
            }
        }
        if index == 2 {
            for (name, value) in [
                ("style", "display:none"),
                ("title", "rogue title"),
                ("inert", "false"),
                ("aria-expanded", "true"),
                ("aria-hidden", "true"),
            ] {
                attrs.push(Attribute::new(name, value, None, false));
            }
            let raw = data.clone();
            attrs.push(Attribute::new(
                "onmousedown",
                dioxus::html::EventHandlerValue::into_platform_listener(EventHandler::new(
                    move |_: MouseEvent| raw.raw_calls.set(raw.raw_calls.get() + 1),
                )),
                None,
                false,
            ));
        }
        if index == 4 {
            attrs.push(Attribute::new("title", "rogue title", None, false));
        }
        if index == 5 {
            for (name, value) in [
                ("href", "#rogue"),
                ("target", "_blank"),
                ("rel", "opener"),
                ("role", "menuitem"),
            ] {
                attrs.push(Attribute::new(name, value, None, false));
            }
        }
        let raw = data.clone();
        attrs.push(Attribute::new(
            "onclick",
            dioxus::html::EventHandlerValue::into_platform_listener(EventHandler::new(
                move |_: MouseEvent| raw.raw_calls.set(raw.raw_calls.get() + 1),
            )),
            None,
            false,
        ));
        let mounts = data.clone();
        attrs.push(Attribute::new(
            "onmounted",
            dioxus::html::EventHandlerValue::into_platform_listener(EventHandler::new(
                move |_: MountedEvent| mounts.raw_mounts.set(mounts.raw_mounts.get() + 1),
            )),
            None,
            false,
        ));
    }
    attrs
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let callbacks = std::array::from_fn::<_, 6, _>(|_| data.clone());
    let [button, anchor, icon, toggle, icon_toggle, link] = callbacks;
    let [button_mount, anchor_mount, icon_mount] = std::array::from_fn::<_, 3, _>(|_| data.clone());
    let mouse = data.clone();
    rsx! {
        Button {id:IDS[0],disabled:config.disabled,loading:config.loading,inert:config.inert,attributes:metadata(data.clone(),0),
            onmounted:move |_|button_mount.typed_mounts.set(button_mount.typed_mounts.get()+1),
            onclick:move |_|button.calls[0].set(button.calls[0].get()+1),LocalLabel {data:data.clone()}
        }
        Button {id:IDS[1],href:"#safe",disabled:config.disabled,loading:config.loading,inert:config.inert,attributes:metadata(data.clone(),1),
            onmounted:move |_|anchor_mount.typed_mounts.set(anchor_mount.typed_mounts.get()+1),
            onclick:move |_|anchor.calls[1].set(anchor.calls[1].get()+1),"Read manual"
        }
        IconButton {id:IDS[2],icon:"check",label:"Confirm work",disabled:config.disabled,loading:config.loading,inert:config.inert,attributes:metadata(data.clone(),2),
            onmounted:move |_|icon_mount.typed_mounts.set(icon_mount.typed_mounts.get()+1),
            onmousedown:move |_|mouse.typed_mouse.set(mouse.typed_mouse.get()+1),
            onclick:move |_|icon.calls[2].set(icon.calls[2].get()+1)
        }
        ToggleButton {pressed:config.pressed,disabled:config.disabled,loading:config.loading,attributes:metadata(data.clone(),3),
            onpressedchange:move |next|{toggle.calls[3].set(toggle.calls[3].get()+1);toggle.requests[0].set(Some(next));},"Pin work"
        }
        IconToggleButton {icon:"star",label:"Favorite work",pressed:config.pressed,disabled:config.disabled,loading:config.loading,attributes:metadata(data.clone(),4),
            onpressedchange:move |next|{icon_toggle.calls[4].set(icon_toggle.calls[4].get()+1);icon_toggle.requests[1].set(Some(next));}
        }
        LinkButton {url:"#reference",disabled:config.disabled,attributes:metadata(data.clone(),5),onclick:move |_|link.calls[5].set(link.calls[5].get()+1),"Reference"}
    }
}
struct Mouse;
impl dioxus::html::InteractionLocation for Mouse {
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
struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! action_attribute_converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*}) => {
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,_:&dioxus::html::PlatformEventData)->dioxus::html::$data {action_attribute_converter!(@data $group)})*}
    };
    (@data Mouse)=>{MouseData::new(Mouse)};
    (@data Focus)=>{FocusData::new(Focus)};
    (@data Mounted)=>{MountedData::new(())};
    (@data $other:ident)=>{panic!("Unexpected native action metadata event")};
}
dioxus::html::with_html_event_groups!(action_attribute_converter);
fn setup(data: Rc<Fixture>) -> (VirtualDom, BTreeMap<String, dioxus::core::ElementId>) {
    let mut dom = VirtualDom::new_with_props(fixture, data);
    let mut current = dioxus::core::ElementId::ROOT;
    let mut targets = BTreeMap::new();
    for edit in dom.rebuild_to_vec().edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" => {
                targets.insert(value, current);
            }
            _ => {}
        }
    }
    assert_eq!(targets.len(), 6);
    (dom, targets)
}
fn refresh(dom: &mut VirtualDom) {
    dom.mark_dirty(ScopeId::APP);
    for _ in 0..4 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
fn dispatch(dom: &VirtualDom, name: &str, id: dioxus::core::ElementId) -> bool {
    let event = Event::new(
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))) as Rc<dyn std::any::Any>,
        name != "mounted",
    );
    dom.runtime().handle_event(name, event.clone(), id);
    event.default_action_enabled()
}
#[test]
fn live_native_metadata_updates_and_removal_retain_control_handlers_and_child_scope() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, targets) = setup(data.clone());
    dom.in_scope(ScopeId::APP, || {
        data.private_text
            .get()
            .unwrap()
            .set("Retained private draft".into())
    });
    for value in [Some("help-one"), Some("help-two"), None] {
        data.config.borrow_mut().metadata = value.map(str::to_string);
        refresh(&mut dom);
        let html = dioxus_ssr::render(&dom);
        for id in IDS {
            let control = tag(&html, id);
            assert_eq!(attribute(control, "aria-details"), value);
            assert_eq!(attribute(control, "data-live"), value);
            dispatch(&dom, "focus", targets[id]);
        }
        assert!(html.contains("Retained private draft"));
        assert_eq!(data.child_mounts.get(), 1);
        assert_eq!(data.child_drops.get(), 0);
    }
    assert_eq!(
        data.focus_calls.get(),
        18,
        "unowned focus handlers reach all six retained controls"
    );
    drop(dom);
    assert_eq!(data.child_drops.get(), 1);
}
#[test]
fn raw_owned_attributes_cannot_replace_identity_state_or_activation_and_mount_handlers() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().attack = true;
    let (mut dom, targets) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    assert!(
        !html.contains("rogue"),
        "owned raw values are absent, even when optional typed props are None"
    );
    for id in IDS {
        let control = tag(&html, id);
        assert_eq!(attribute(control, "aria-disabled"), Some("false"));
        assert_ne!(attribute(control, "class"), Some("rogue-class"));
        dispatch(&dom, "mounted", targets[id]);
        assert!(dispatch(&dom, "click", targets[id]));
    }
    assert_eq!(
        data.typed_mounts.get(),
        3,
        "Button/anchor/IconButton own mounted callbacks"
    );
    assert_eq!(
        data.raw_mounts.get(),
        3,
        "toggle/link mount events are unowned and forwarded"
    );
    assert_eq!(data.calls.each_ref().map(Cell::get), [1; 6]);
    assert_eq!(data.requests.each_ref().map(Cell::get), [Some(true); 2]);
    assert_eq!(data.raw_calls.get(), 0);
    assert_eq!(attribute(tag(&html, IDS[0]), "type"), Some("button"));
    assert_eq!(attribute(tag(&html, IDS[0]), "aria-label"), None);
    assert_eq!(attribute(tag(&html, IDS[0]), "aria-describedby"), None);
    assert_eq!(attribute(tag(&html, IDS[1]), "href"), Some("#safe"));
    assert_eq!(attribute(tag(&html, IDS[1]), "target"), Some("_self"));
    assert_eq!(
        attribute(tag(&html, IDS[2]), "aria-label"),
        Some("Confirm work")
    );
    assert_eq!(attribute(tag(&html, IDS[2]), "style"), None);
    assert_eq!(attribute(tag(&html, IDS[2]), "aria-expanded"), None);
    assert_eq!(attribute(tag(&html, IDS[2]), "aria-hidden"), None);
    assert_eq!(attribute(tag(&html, IDS[3]), "aria-pressed"), Some("false"));
    assert_eq!(
        attribute(tag(&html, IDS[4]), "aria-label"),
        Some("Favorite work")
    );
    assert_eq!(attribute(tag(&html, IDS[5]), "href"), Some("#reference"));
    assert_eq!(attribute(tag(&html, IDS[5]), "role"), None);
    dispatch(&dom, "mousedown", targets[IDS[2]]);
    assert_eq!(data.typed_mouse.get(), 1);
    assert_eq!(data.raw_calls.get(), 0);
    for loading in [false, true] {
        {
            let mut config = data.config.borrow_mut();
            config.disabled = !loading;
            config.loading = loading;
            config.pressed = true;
        }
        refresh(&mut dom);
        for id in IDS.into_iter().take(5) {
            dispatch(&dom, "click", targets[id]);
        }
        if !loading {
            assert!(!dispatch(&dom, "click", targets[IDS[5]]));
        }
        assert_eq!(data.calls.each_ref().map(Cell::get), [1; 6]);
        let html = dioxus_ssr::render(&dom);
        for id in IDS.into_iter().take(5) {
            let control = tag(&html, id);
            assert_eq!(attribute(control, "aria-disabled"), Some("true"));
            assert_eq!(
                attribute(control, "aria-busy"),
                Some(if loading { "true" } else { "false" })
            );
            if id == IDS[1] {
                assert_eq!(attribute(control, "href"), None);
                assert_eq!(attribute(control, "tabindex"), Some("-1"));
            } else {
                assert!(
                    control.contains(" disabled"),
                    "raw attributes cannot remove native button blocking"
                );
            }
        }
        assert_eq!(attribute(tag(&html, IDS[3]), "aria-pressed"), Some("true"));
        assert_eq!(attribute(tag(&html, IDS[4]), "aria-pressed"), Some("true"));
    }
    {
        let mut config = data.config.borrow_mut();
        config.disabled = false;
        config.loading = false;
        config.inert = true;
    }
    refresh(&mut dom);
    for id in IDS.into_iter().take(3) {
        assert!(!dispatch(&dom, "click", targets[id]));
    }
    assert_eq!(data.calls.each_ref().map(Cell::get), [1; 6]);
    assert_eq!(data.raw_calls.get(), 0);
}
