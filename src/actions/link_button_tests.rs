//! Native navigation/default behavior; trusted browser activation is verified by the batch probe.
use super::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone)]
struct Config {
    url: Option<String>,
    disabled: bool,
    cancel: bool,
    target: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            url: Some("#motor".into()),
            disabled: false,
            cancel: false,
            target: "_self".into(),
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    clicks: Cell<usize>,
    parent_clicks: Cell<usize>,
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let parent = data.clone();
    rsx! { div {
        onclick: move |_| parent.parent_clicks.set(parent.parent_clicks.get()+1),
        LinkButton {
            url: config.url, disabled: config.disabled, target: config.target,
            class: "equipment-reference",
            onclick: move |event: MouseEvent| {
                data.clicks.set(data.clicks.get()+1);
                if config.cancel { event.prevent_default(); }
            },
            span { "Motor manual" }
        }
    } }
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
    (@data $other:ident,$event:ident) => { panic!("Unexpected link test conversion") };
}
dioxus::html::with_html_event_groups!(converter);
fn setup(data: Rc<Fixture>) -> (VirtualDom, dioxus::core::ElementId) {
    let mut dom = VirtualDom::new_with_props(fixture, data);
    let mut current = dioxus::core::ElementId::ROOT;
    let mut link = None;
    for edit in dom.rebuild_to_vec().edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id
            }
            // Parent precedes its one child anchor; neither icon nor label has a listener.
            dioxus::core::Mutation::NewEventListener { name } if name == "click" => {
                link = Some(current)
            }
            _ => {}
        }
    }
    (dom, link.expect("LinkButton click listener"))
}
fn refresh(dom: &mut VirtualDom) {
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
}
fn dispatch(dom: &VirtualDom, name: &str, id: dioxus::core::ElementId) -> bool {
    let event = Event::new(
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))) as Rc<dyn std::any::Any>,
        true,
    );
    dom.runtime().handle_event(name, event.clone(), id);
    event.default_action_enabled()
}
fn anchor(html: &str) -> &str {
    html.split('<')
        .find(|tag| tag.starts_with("a "))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn has_attribute(tag: &str, name: &str, value: &str) -> bool {
    tag.trim_end_matches('/')
        .split_ascii_whitespace()
        .any(|attribute| {
            attribute == format!("{name}={value}") || attribute == format!("{name}=\"{value}\"")
        })
}

#[test]
fn link_button_disabled_transition_removes_native_navigation_and_guards_events() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, link) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    let tag = anchor(&html);
    assert!(has_attribute(tag, "href", "#motor"));
    assert!(!tag.contains("tabindex="));
    assert!(!tag.contains("role="));
    assert!(dispatch(&dom, "click", link));
    assert_eq!(data.clicks.get(), 1);
    assert_eq!(data.parent_clicks.get(), 1);

    data.config.borrow_mut().disabled = true;
    refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    let tag = anchor(&html);
    assert!(!tag.contains("href="));
    assert!(has_attribute(tag, "role", "link"));
    assert!(has_attribute(tag, "tabindex", "-1"));
    assert!(has_attribute(tag, "aria-disabled", "true"));
    assert!(!dispatch(&dom, "click", link));
    assert_eq!(data.clicks.get(), 1);
    assert_eq!(
        data.parent_clicks.get(),
        1,
        "disabled event must not bubble to consumer actions"
    );

    {
        let mut config = data.config.borrow_mut();
        config.disabled = false;
        config.url = None;
    }
    refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    let tag = anchor(&html);
    assert!(!tag.contains("href="));
    assert!(!tag.contains("tabindex="));
    assert!(!tag.contains("role="));
    assert!(dispatch(&dom, "click", link));
    assert_eq!(
        data.clicks.get(),
        2,
        "a destination-free anchor still forwards enabled clicks"
    );
}

#[test]
fn link_button_cancellation_preserves_native_key_defaults_and_safe_target() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().cancel = true;
    let (mut dom, link) = setup(data.clone());
    // The VDOM cannot execute HTML Enter activation. With no keyboard listener,
    // keydown/keyup preserve browser defaults and do not fabricate extra clicks.
    assert!(dispatch(&dom, "keydown", link));
    assert!(dispatch(&dom, "keyup", link));
    assert_eq!(data.clicks.get(), 0);
    assert!(!dispatch(&dom, "click", link));
    assert_eq!(data.clicks.get(), 1);
    assert_eq!(
        data.parent_clicks.get(),
        1,
        "prevent_default alone does not stop propagation"
    );
    {
        let mut config = data.config.borrow_mut();
        config.cancel = false;
        config.target = "_blank".into();
    }
    refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    let tag = anchor(&html);
    assert!(has_attribute(tag, "target", "_blank"));
    assert!(tag.contains("rel=\"noopener noreferrer\""));
    assert!(!tag.contains("title="));
    assert!(dispatch(&dom, "click", link));
    assert_eq!(data.clicks.get(), 2);
}

#[test]
fn link_button_optional_destination_keeps_composed_label_and_decorative_chevron() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            LinkButton { url:"#manual", span { "Manual" } }
            LinkButton { "No destination" }
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("ix-link-button-icon").count(), 2);
    assert_eq!(html.matches("ix-link-button-label").count(), 2);
    assert_eq!(html.matches("aria-hidden=\"true\"").count(), 2);
    assert!(html.contains("width=16") || html.contains("width=\"16\""));
    assert!(html.contains("<span>Manual</span>"));
    assert!(html.contains("No destination"));
    assert!(has_attribute(anchor(&html), "target", "_self"));
    assert_eq!(html.matches("href=").count(), 1);
    assert!(!html.contains("tabindex="));
}
