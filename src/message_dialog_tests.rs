// Focused composition/event tests; actual native activation and return-focus stay in the shared modal/native suite.
use crate::{MessageDialog, MessageDialogVariant};
use std::cell::Cell;
use std::collections::BTreeMap;

#[derive(Clone)]
struct MessageConfig {
    title: String,
    message: String,
    variant: MessageDialogVariant,
    icon_label: Option<String>,
    close_label: String,
    hide_close: bool,
    veto_close: bool,
    single: bool,
    action_value: String,
    size: ModalSize,
    centered: bool,
    hide_backdrop: bool,
    close_backdrop: bool,
    disable_animation: bool,
    non_blocking: bool,
}
impl Default for MessageConfig {
    fn default() -> Self {
        Self {
            title: "Service inspection".into(),
            message: "Review Motor 001 before continuing.".into(),
            variant: MessageDialogVariant::Info,
            icon_label: None,
            close_label: "Close service notice".into(),
            hide_close: false,
            veto_close: false,
            single: false,
            action_value: "inspection-approved".into(),
            size: ModalSize::Xs,
            centered: false,
            hide_backdrop: false,
            close_backdrop: false,
            disable_animation: false,
            non_blocking: false,
        }
    }
}
#[derive(Default)]
struct MessageObservation {
    config: Cell<Option<Signal<MessageConfig>>>,
    modal: Cell<Option<ModalHandle>>,
    requests: RefCell<Vec<ModalDismissRequest>>,
    results: RefCell<Vec<ModalResult>>,
    open: RefCell<Vec<bool>>,
}
#[derive(Clone, Default)]
struct MessageFixture(Rc<MessageObservation>);
impl PartialEq for MessageFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[component]
fn LocalMessageBody() -> Element {
    let mut confirmations = use_signal(|| 0usize);
    rsx! {button {id:String::from("message-local-action"),type:"button",onclick:move |_|confirmations+=1,"Local confirmations: {confirmations}"}}
}
fn message_fixture(fixture: MessageFixture) -> Element {
    let modal = use_modal();
    let config = use_signal(MessageConfig::default);
    fixture.0.config.set(Some(config));
    fixture.0.modal.set(Some(modal));
    let config = config();
    let requests = fixture.clone();
    let results = fixture.clone();
    let open = fixture.clone();
    rsx! {MessageDialog {
        modal,title:config.title,message:config.message,variant:config.variant,icon_label:config.icon_label,
        close_label:config.close_label,hide_close:config.hide_close,
        size:config.size,centered:config.centered,hide_backdrop:config.hide_backdrop,close_on_backdrop_click:config.close_backdrop,
        disable_animation:config.disable_animation,is_non_blocking:config.non_blocking,
        oncloseclick:move |event:MouseEvent|if config.veto_close {event.prevent_default();},
        ondismissrequest:move |request|requests.0.requests.borrow_mut().push(request),
        onresult:move |result|results.0.results.borrow_mut().push(result),
        onopenchange:move |value|open.0.open.borrow_mut().push(value),
        actions:rsx! {
            if !config.single {ModalClose {dismiss:true,"Keep request"}}
            button {id:String::from("message-confirm"),type:"button",onclick:move |_|modal.close(config.action_value.clone()),"Confirm inspection"}
        },
        LocalMessageBody {}
    }}
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
    (@data $other:ident,$event:ident) => { panic!("Unexpected message composition event") };
}
dioxus::html::with_html_event_groups!(converter);

#[derive(Default)]
struct MessageTargets {
    ids: BTreeMap<String, dioxus::core::ElementId>,
    labels: BTreeMap<String, dioxus::core::ElementId>,
}
impl MessageTargets {
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
}
fn message_setup(fixture: MessageFixture) -> (VirtualDom, MessageTargets) {
    let mut dom = VirtualDom::new_with_props(message_fixture, fixture);
    let mut targets = MessageTargets::default();
    targets.collect(dom.rebuild_to_vec());
    message_settle(&mut dom, &mut targets);
    (dom, targets)
}
fn message_settle(dom: &mut VirtualDom, targets: &mut MessageTargets) {
    for _ in 0..4 {
        targets.collect(dom.render_immediate_to_vec());
        dom.process_events();
    }
}
fn message_click(dom: &VirtualDom, id: dioxus::core::ElementId) -> bool {
    let event = Event::new(
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))) as Rc<dyn std::any::Any>,
        true,
    );
    dom.runtime().handle_event("click", event.clone(), id);
    event.default_action_enabled()
}
fn message_dialog_tag(html: &str) -> &str {
    html.split('<')
        .find(|tag| tag.starts_with("dialog "))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}

#[test]
fn message_dialog_live_content_variants_actions_and_description_preserve_composed_state() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let fixture = MessageFixture::default();
    let (mut dom, mut targets) = message_setup(fixture.clone());
    let modal = fixture.0.modal.get().unwrap();
    let mut config = fixture.0.config.get().unwrap();
    let description = format!("{}-message", modal.id());
    let html = dioxus_ssr::render(&dom);
    assert!(message_dialog_tag(&html).contains("aria-label=\"Service inspection\""));
    assert!(message_dialog_tag(&html).contains(&format!("aria-describedby=\"{description}\"")));
    assert!(html.contains(&format!("id=\"{description}\"")));
    assert!(html.contains("Review Motor 001 before continuing."));
    assert!(html.contains("Keep request"));
    message_click(&dom, targets.ids["message-local-action"]);
    message_settle(&mut dom, &mut targets);
    assert!(dioxus_ssr::render(&dom).contains("Local confirmations: 1"));
    dom.in_runtime(|| {
        let mut config = config.write();
        config.title = "Wartung prüfen".into();
        config.message = "Anfrage bleibt lokal.".into();
        config.variant = MessageDialogVariant::Warning;
        config.icon_label = Some("Warnung".into());
        config.close_label = "Hinweis schließen".into();
        config.single = true;
        config.action_value = "updated-confirmation".into();
    });
    message_settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(message_dialog_tag(&html).contains("aria-label=\"Wartung prüfen\""));
    assert!(html.contains("Anfrage bleibt lokal."));
    assert!(!html.contains("Keep request"));
    assert!(html.contains("aria-label=\"Warnung\""));
    assert!(html.contains("color:var(--ix-color-warning-text)"));
    assert!(
        html.contains("Local confirmations: 1"),
        "live message props must not remount composed state"
    );
    assert!(html.contains("aria-label=\"Hinweis schließen\""));
    dom.in_runtime(|| modal.opened());
    message_click(&dom, targets.ids["message-confirm"]);
    assert_eq!(
        *fixture.0.results.borrow(),
        [ModalResult::Closed("updated-confirmation".into())]
    );
}

#[test]
fn message_dialog_forwards_close_cancellation_deferred_guard_and_unmapped_results() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let fixture = MessageFixture::default();
    let (mut dom, mut targets) = message_setup(fixture.clone());
    let modal = fixture.0.modal.get().unwrap();
    let mut config = fixture.0.config.get().unwrap();
    dom.in_runtime(|| {
        config.write().veto_close = true;
        modal.opened();
    });
    message_settle(&mut dom, &mut targets);
    assert!(!message_click(&dom, targets.labels["Close service notice"]));
    assert!(fixture.0.requests.borrow().is_empty());
    assert!(modal.is_open());
    dom.in_runtime(|| config.write().veto_close = false);
    message_settle(&mut dom, &mut targets);
    message_click(&dom, targets.labels["Close service notice"]);
    let request = fixture.0.requests.borrow_mut().pop().unwrap();
    assert_eq!(request.reason, ModalDismissReason::Header);
    dom.in_runtime(|| assert!(request.deny()));
    assert!(modal.is_open());
    assert!(fixture.0.results.borrow().is_empty());
    message_click(&dom, targets.labels["Close service notice"]);
    let request = fixture.0.requests.borrow_mut().pop().unwrap();
    dom.in_runtime(|| assert!(request.allow()));
    assert_eq!(
        *fixture.0.results.borrow(),
        [ModalResult::Dismissed(ModalDismissReason::Header)]
    );
    message_settle(&mut dom, &mut targets);
    dom.in_runtime(|| modal.opened());
    message_click(&dom, targets.ids["message-confirm"]);
    assert_eq!(
        *fixture.0.results.borrow(),
        [
            ModalResult::Dismissed(ModalDismissReason::Header),
            ModalResult::Closed("inspection-approved".into())
        ]
    );
    assert_eq!(*fixture.0.open.borrow(), [true, false, true, false]);
}

#[test]
fn message_dialog_presets_and_existing_modal_options_keep_native_structure() {
    let fixture = MessageFixture::default();
    let (mut dom, mut targets) = message_setup(fixture.clone());
    let mut config = fixture.0.config.get().unwrap();
    for (variant, token) in [
        (MessageDialogVariant::Info, "--ix-color-info-text"),
        (MessageDialogVariant::Warning, "--ix-color-warning-text"),
        (MessageDialogVariant::Error, "--ix-color-danger-text"),
        (MessageDialogVariant::Success, "--ix-color-success"),
        (MessageDialogVariant::Question, "--ix-color-text"),
    ] {
        dom.in_runtime(|| config.write().variant = variant);
        message_settle(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains(&format!("color:var({token})")));
        assert!(html.contains("ix-modal-header-icon"));
        assert!(html.contains("width=24") || html.contains("width=\"24\""));
    }
    dom.in_runtime(|| {
        let mut config = config.write();
        config.hide_close = true;
        config.size = ModalSize::Md;
        config.centered = true;
        config.hide_backdrop = true;
        config.close_backdrop = true;
        config.disable_animation = true;
        config.non_blocking = true;
    });
    message_settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    let dialog = message_dialog_tag(&html);
    assert!(!html.contains("ix-dialog-close"));
    assert!(dialog.contains("ix-modal-size-600"));
    for attribute in [
        "data-centered=\"true\"",
        "data-hide-backdrop=\"true\"",
        "data-no-animation=\"true\"",
        "aria-modal=\"false\"",
        "popover=\"manual\"",
    ] {
        assert!(dialog.contains(attribute), "missing {attribute}");
    }
    assert!(html.contains("ix-modal-footer"));
    assert!(html.contains("ix-modal-content"));
    assert!(
        !dialog.contains(" open"),
        "native activation stays with ModalTrigger"
    );
}
