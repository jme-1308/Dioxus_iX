//! Focused MessageBar lifecycle and event regressions.
use super::*;
use std::{cell::RefCell, collections::BTreeMap};

#[derive(Clone)]
struct Config {
    persistent: bool,
    veto: bool,
    remove_onclose: bool,
    label: String,
    message: String,
    kind: MessageType,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            persistent: false,
            veto: false,
            remove_onclose: false,
            label: "Dismiss maintenance notice".into(),
            message: "Inspection due".into(),
            kind: MessageType::Info,
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    events: RefCell<Vec<&'static str>>,
    actions: Cell<usize>,
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let mut present = use_signal(|| true);
    let request = data.clone();
    let accepted = data.clone();
    let complete = data.clone();
    rsx! {
        if present() {
            crate::MessageBar {
                message_type:config.kind,persistent:config.persistent,close_label:config.label,
                oncloserequest: move |event:crate::MessageBarCloseRequest| {
                    request.events.borrow_mut().push("request");
                    if config.veto {
                        event.clone().prevent_default();
                        assert!(event.default_prevented(),"request clones share cancellation");
                    }
                },
                onclose:move |_| {
                    accepted.events.borrow_mut().push("accepted");
                    if config.remove_onclose {present.set(false);}
                },
                onclosecomplete:move |_|complete.events.borrow_mut().push("complete"),
                span {"{config.message}"}
                button {type:"button",aria_label:String::from("Acknowledge maintenance"),onclick:move |_|data.actions.set(data.actions.get()+1),"Acknowledge"}
            }
        }
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
    (@data $other:ident,$event:ident) => { panic!("Unexpected message-bar test conversion") };
}
dioxus::html::with_html_event_groups!(converter);

#[derive(Default)]
struct Targets {
    labels: BTreeMap<String, dioxus::core::ElementId>,
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
                } if name == "aria-label" => {
                    self.labels.insert(value, current);
                }
                _ => {}
            }
        }
    }
    fn label(&self, label: &str) -> dioxus::core::ElementId {
        *self
            .labels
            .get(label)
            .unwrap_or_else(|| panic!("Missing {label}"))
    }
}
fn setup(data: Rc<Fixture>) -> (VirtualDom, Targets) {
    let mut dom = VirtualDom::new_with_props(fixture, data);
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    settle(&mut dom, &mut targets);
    (dom, targets)
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
fn click(dom: &VirtualDom, id: dioxus::core::ElementId) {
    let event = Event::new(
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))) as Rc<dyn std::any::Any>,
        true,
    );
    dom.runtime().handle_event("click", event, id);
}

#[test]
fn message_bar_veto_then_accept_completes_once_after_closed_render() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().veto = true;
    let (mut dom, mut targets) = setup(data.clone());
    assert!(
        data.events.borrow().is_empty(),
        "initial render must not notify completion"
    );
    let close = targets.label("Dismiss maintenance notice");
    click(&dom, close);
    settle(&mut dom, &mut targets);
    assert_eq!(*data.events.borrow(), ["request"]);
    assert!(dioxus_ssr::render(&dom).contains("Inspection due"));

    data.config.borrow_mut().veto = false;
    refresh(&mut dom, &mut targets);
    click(&dom, close);
    assert_eq!(*data.events.borrow(), ["request", "request", "accepted"]);
    // Repeated activation before the pending render cannot accept twice.
    click(&dom, close);
    assert_eq!(*data.events.borrow(), ["request", "request", "accepted"]);
    targets.collect(dom.render_immediate_to_vec());
    assert!(!dioxus_ssr::render(&dom).contains("ix-message-bar"));
    settle(&mut dom, &mut targets);
    assert_eq!(
        *data.events.borrow(),
        ["request", "request", "accepted", "complete"]
    );
    refresh(&mut dom, &mut targets);
    assert_eq!(
        *data.events.borrow(),
        ["request", "request", "accepted", "complete"]
    );
}

#[test]
fn message_bar_persistent_and_live_props_preserve_content_actions_and_labels() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().persistent = true;
    let (mut dom, mut targets) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("ix-message-close"));
    assert!(html.contains("role=\"alert\""));
    click(&dom, targets.label("Acknowledge maintenance"));
    assert_eq!(data.actions.get(), 1);
    assert!(data.events.borrow().is_empty());
    {
        let mut config = data.config.borrow_mut();
        config.persistent = false;
        config.label = "Wartungshinweis schließen".into();
        config.message = "Inspection ready".into();
        config.kind = MessageType::Warning;
    }
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Inspection ready"));
    assert!(html.contains("data-type=\"warning\""));
    assert!(html.contains("aria-label=\"Wartungshinweis schließen\""));
    click(&dom, targets.label("Acknowledge maintenance"));
    assert_eq!(data.actions.get(), 2);
    assert!(data.events.borrow().is_empty());
    click(&dom, targets.label("Wartungshinweis schließen"));
    settle(&mut dom, &mut targets);
    assert_eq!(*data.events.borrow(), ["request", "accepted", "complete"]);
}

#[test]
fn message_bar_parent_unmount_skips_owned_completion_and_legacy_variants_remain() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().remove_onclose = true;
    let (mut dom, mut targets) = setup(data.clone());
    click(&dom, targets.label("Dismiss maintenance notice"));
    settle(&mut dom, &mut targets);
    assert!(!dioxus_ssr::render(&dom).contains("ix-message-bar"));
    assert_eq!(*data.events.borrow(), ["request", "accepted"]);

    fn legacy() -> Element {
        rsx! {
            MessageBar {"Default information"}
            for kind in [MessageType::Alarm,MessageType::Critical,MessageType::Warning,MessageType::Success,MessageType::Neutral,MessageType::Primary] {
                crate::overlays::MessageBar {message_type:kind,persistent:true,"Contextual notice"}
            }
        }
    }
    let mut legacy_dom = VirtualDom::new(legacy);
    legacy_dom.rebuild_in_place();
    let html = dioxus_ssr::render(&legacy_dom);
    assert_eq!(html.matches("role=\"alert\"").count(), 7);
    assert_eq!(html.matches("ix-message-close").count(), 1);
    assert!(html.contains("aria-label=\"Close message\""));
    for name in [
        "info", "alarm", "critical", "warning", "success", "neutral", "primary",
    ] {
        assert!(
            html.contains(&format!("data-type=\"{name}\"")),
            "missing {name}"
        );
    }
}
