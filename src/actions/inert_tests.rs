//! Actual native control rendering and listener guards for inert actions.
use super::*;
use std::cell::Cell;
use std::collections::BTreeMap;

#[derive(Clone, Default)]
struct Config {
    inert: bool,
    disabled: bool,
    loading: bool,
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    calls: [Cell<usize>; 3],
    parent_calls: Cell<usize>,
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let parent = data.clone();
    let button = data.clone();
    let link = data.clone();
    let icon = data.clone();
    rsx! { div {
        onclick: move |_| parent.parent_calls.set(parent.parent_calls.get() + 1),
        Button {id:"inert-button",button_type:ButtonType::Submit,inert:config.inert,disabled:config.disabled,loading:config.loading,
            onclick:move |_|button.calls[0].set(button.calls[0].get()+1),"Save work"
        }
        Button {id:"inert-anchor",href:"#manual",inert:config.inert,disabled:config.disabled,loading:config.loading,
            onclick:move |_|link.calls[1].set(link.calls[1].get()+1),"Read manual"
        }
        IconButton {id:"inert-icon",icon:"check",label:"Confirm work",button_type:ButtonType::Submit,inert:config.inert,disabled:config.disabled,loading:config.loading,
            onclick:move |_|icon.calls[2].set(icon.calls[2].get()+1)
        }
    }}
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
struct Converter;
macro_rules! inert_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for Converter {
            $(fn $convert(&self, _: &dioxus::html::PlatformEventData) -> dioxus::html::$data { inert_converter!(@data $group) })*
        }
    };
    (@data Mouse) => { MouseData::new(Mouse) };
    (@data $other:ident) => { panic!("Unexpected inert action test event") };
}
dioxus::html::with_html_event_groups!(inert_converter);
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
    assert_eq!(targets.len(), 3);
    (dom, targets)
}
fn refresh(dom: &mut VirtualDom) {
    dom.mark_dirty(ScopeId::APP);
    let edits = dom.render_immediate_to_vec();
    assert!(
        !edits.edits.iter().any(|edit| matches!(edit,
            dioxus::core::Mutation::NewEventListener {name} if name == "click"
        )),
        "state changes retain the existing native control listeners"
    );
}
fn dispatch(dom: &VirtualDom, target: dioxus::core::ElementId) -> (bool, bool) {
    let data = Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))) as Rc<dyn std::any::Any>;
    let event = Event::new(data, true);
    dom.runtime().handle_event("click", event.clone(), target);
    (event.default_action_enabled(), event.propagates())
}
fn tag<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .filter(|part| part.starts_with("button ") || part.starts_with("a "))
        .find(|part| part.contains(&format!("id=\"{id}\"")))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn attribute_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let rest = tag.split_once(&format!(" {name}="))?.1;
    if let Some(quoted) = rest.strip_prefix('"') {
        quoted.split_once('"').map(|(value, _)| value)
    } else {
        rest.split_whitespace().next()
    }
}
fn has_attribute(tag: &str, name: &str) -> bool {
    tag.split_ascii_whitespace()
        .any(|part| part == name || part.starts_with(&format!("{name}=")))
}
fn counts(data: &Fixture) -> [usize; 3] {
    data.calls.each_ref().map(Cell::get)
}

#[test]
fn inert_actions_preserve_paint_and_native_semantics_while_attribute_tracks_live_state() {
    let mut defaults = VirtualDom::new(|| {
        rsx! {
            Button {id:"default-button","Default button"}
            Button {id:"default-anchor",href:"#manual","Default anchor"}
            IconButton {id:"default-icon",icon:"check",label:"Default icon"}
        }
    });
    defaults.rebuild_in_place();
    let default_html = dioxus_ssr::render(&defaults);
    for id in ["default-button", "default-anchor", "default-icon"] {
        assert!(!has_attribute(tag(&default_html, id), "inert"));
    }
    let data = Rc::new(Fixture::default());
    let (mut dom, _) = setup(data.clone());
    let initial = dioxus_ssr::render(&dom);
    let ids = ["inert-button", "inert-anchor", "inert-icon"];
    let classes = ids.map(|id| {
        attribute_value(tag(&initial, id), "class")
            .unwrap()
            .to_string()
    });
    for id in ids {
        assert!(!has_attribute(tag(&initial, id), "inert"));
    }
    data.config.borrow_mut().inert = true;
    refresh(&mut dom);
    let inert = dioxus_ssr::render(&dom);
    for (id, class) in ids.into_iter().zip(&classes) {
        let control = tag(&inert, id);
        assert!(has_attribute(control, "inert"));
        assert!(!has_attribute(control, "disabled"));
        assert_eq!(attribute_value(control, "aria-disabled"), Some("false"));
        assert_eq!(attribute_value(control, "aria-busy"), Some("false"));
        assert_eq!(attribute_value(control, "class"), Some(class.as_str()));
    }
    assert_eq!(
        attribute_value(tag(&inert, "inert-anchor"), "href"),
        Some("#manual")
    );
    assert_eq!(
        attribute_value(tag(&inert, "inert-icon"), "aria-label"),
        Some("Confirm work")
    );
    assert_eq!(
        attribute_value(tag(&inert, "inert-button"), "type"),
        Some("submit")
    );
    data.config.borrow_mut().inert = false;
    refresh(&mut dom);
    let enabled = dioxus_ssr::render(&dom);
    for (id, class) in ids.into_iter().zip(&classes) {
        assert!(!has_attribute(tag(&enabled, id), "inert"));
        assert_eq!(
            attribute_value(tag(&enabled, id), "class"),
            Some(class.as_str())
        );
    }
}

#[test]
fn inert_actions_guard_real_listeners_without_changing_disabled_or_loading_guards() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, targets) = setup(data.clone());
    let ids = ["inert-button", "inert-anchor", "inert-icon"];
    for id in ids {
        assert_eq!(dispatch(&dom, targets[id]), (true, true));
    }
    assert_eq!(counts(&data), [1, 1, 1]);
    assert_eq!(data.parent_calls.get(), 3);
    data.config.borrow_mut().inert = true;
    refresh(&mut dom);
    for id in ids {
        assert_eq!(dispatch(&dom, targets[id]), (false, false));
    }
    assert_eq!(counts(&data), [1, 1, 1]);
    assert_eq!(
        data.parent_calls.get(),
        3,
        "inert callbacks do not bubble into consumer actions"
    );
    data.config.borrow_mut().inert = false;
    refresh(&mut dom);
    for id in ids {
        assert_eq!(dispatch(&dom, targets[id]), (true, true));
    }
    assert_eq!(counts(&data), [2, 2, 2]);
    assert_eq!(data.parent_calls.get(), 6);
    for loading in [false, true] {
        {
            let mut config = data.config.borrow_mut();
            config.disabled = !loading;
            config.loading = loading;
        }
        refresh(&mut dom);
        for id in ids {
            let _ = dispatch(&dom, targets[id]);
        }
        assert_eq!(
            counts(&data),
            [2, 2, 2],
            "disabled/loading callbacks remain blocked"
        );
    }
    {
        let mut config = data.config.borrow_mut();
        config.disabled = false;
        config.loading = false;
    }
    refresh(&mut dom);
    for id in ids {
        assert_eq!(dispatch(&dom, targets[id]), (true, true));
    }
    assert_eq!(counts(&data), [3, 3, 3]);
}
