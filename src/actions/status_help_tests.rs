use super::*;
use std::{cell::Cell, collections::BTreeMap};
fn settle(dom: &mut VirtualDom) {
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
fn opening<'a>(html: &'a str, tag: &str, marker: &str) -> &'a str {
    html.split('<')
        .filter(|part| part.starts_with(&format!("{tag} ")))
        .map(|part| part.split('>').next().unwrap())
        .find(|part| part.contains(marker))
        .unwrap()
}
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else {
        value.split_whitespace().next()
    }
}
#[test]
fn status_help_defaults_preserve_names_and_geometry_and_descriptions_compose() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Pill {id:"plain-pill",class:"plain-pill","Ready"}
            Pill {id:"blank-pill",tooltip:"  ",aria_describedby:"existing-pill","Blank help"}
            Pill {id:"named-pill",tooltip:"Inspection passed",aria_label:"Ready for production",aria_describedby:"existing-note",tabindex:-1,"Ready"}
            Chip {id:"plain-chip",inactive:true,closable:true,aria_label:"Selected motor",aria_describedby:"existing-chip","Motor"}
        }
    });
    dom.rebuild_in_place();
    settle(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        attr(opening(&html, "span", "id=\"plain-pill\""), "tabindex"),
        None
    );
    let blank = opening(&html, "span", "id=\"blank-pill\"");
    assert_eq!(attr(blank, "tabindex"), None);
    assert_eq!(attr(blank, "aria-describedby"), Some("existing-pill"));
    let named = opening(&html, "span", "id=\"named-pill\"");
    assert_eq!(attr(named, "tabindex"), Some("-1"));
    assert_eq!(attr(named, "role"), Some("img"));
    assert_eq!(attr(named, "aria-label"), Some("Ready for production"));
    let tokens: Vec<_> = attr(named, "aria-describedby")
        .unwrap()
        .split_whitespace()
        .collect();
    assert_eq!(tokens[0], "existing-note");
    assert_eq!(tokens.len(), 2);
    assert!(html.contains(&format!("id=\"{}\"", tokens[1])));
    let main = opening(&html, "button", "id=\"plain-chip\"");
    assert!(
        main.split_whitespace()
            .any(|value| value == "disabled" || value.starts_with("disabled="))
    );
    assert_eq!(attr(main, "aria-label"), Some("Selected motor"));
    assert_eq!(attr(main, "aria-describedby"), Some("existing-chip"));
    assert_eq!(
        attr(opening(&html, "span", "ix-chip ix-status"), "tabindex"),
        None
    );
    assert!(!opening(&html, "button", "ix-chip-close").contains(" disabled"));
    assert_eq!(html.matches("role=\"tooltip\"").count(), 1);
    assert!(!html.contains(" title="));
}
#[derive(Default)]
struct Capture {
    mode: Cell<Option<Signal<u8>>>,
    note: Cell<Option<Signal<String>>>,
    mounts: Cell<usize>,
    drops: Cell<usize>,
    main: RefCell<Vec<u8>>,
    close: RefCell<Vec<u8>>,
}
impl PartialEq for Capture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn PrivateLabel(data: Rc<Capture>) -> Element {
    let note = use_signal(|| "Private status".to_string());
    data.note.set(Some(note));
    use_hook({
        let data = data.clone();
        move || data.mounts.set(data.mounts.get() + 1)
    });
    use_drop(move || data.drops.set(data.drops.get() + 1));
    rsx! {span {"{note}"}}
}
fn fixture(data: Rc<Capture>) -> Element {
    let mode = use_signal(|| 0u8);
    data.mode.set(Some(mode));
    let mode = mode();
    let main = data.clone();
    let close = data.clone();
    let text = if mode == 2 {
        None
    } else {
        Some(format!("Current guidance {mode}"))
    };
    rsx! {if mode<4 {
        Pill {id:"help-pill",tooltip:text.clone(),aria_describedby:"caller-pill",PrivateLabel {data:data.clone()}}
        Chip {id:"help-chip",tooltip:text,aria_label:"Motor selection",aria_describedby:"caller-chip",inactive:mode==3,closable:true,
            onclick:move|_|main.main.borrow_mut().push(mode),onclose:move|_|close.close.borrow_mut().push(mode),"Motor"
        }
    }}
}
struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Keyboard(Key);
impl dioxus::html::ModifiersInteraction for Keyboard {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for Keyboard {
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
struct StatusMount;
impl dioxus::html::RenderedElementBacking for StatusMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_client_rect(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>,
                >,
        >,
    > {
        Box::pin(async {
            Ok(dioxus::html::geometry::PixelsRect::new(
                dioxus::html::geometry::euclid::Point2D::new(100.0, 100.0),
                dioxus::html::geometry::euclid::Size2D::new(80.0, 24.0),
            ))
        })
    }
}
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*}
    };
    (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(Focus)};
    (@data Keyboard,$event:ident)=>{dioxus::html::KeyboardData::new(Keyboard($event.downcast::<Key>().unwrap().clone()))};
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(Mouse)};
    (@data Mounted,$event:ident)=>{MountedData::new(StatusMount)};
    (@data $other:ident,$event:ident)=>{panic!("unexpected status help test event")};
}
dioxus::html::with_html_event_groups!(converter);
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    target: dioxus::core::ElementId,
    payload: impl std::any::Any,
) -> bool {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    let event = Event::new(data, name != "mounted");
    dom.runtime().handle_event(name, event.clone(), target);
    event.propagates()
}
fn targets(edits: dioxus::core::Mutations) -> BTreeMap<String, dioxus::core::ElementId> {
    let mut current = dioxus::core::ElementId::ROOT;
    let mut result = BTreeMap::new();
    for edit in edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" => {
                result.insert(value, current);
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "aria-label" && value == "Close chip" => {
                result.insert("close".into(), current);
            }
            _ => {}
        }
    }
    result
}
fn setup() -> (
    VirtualDom,
    Rc<Capture>,
    BTreeMap<String, dioxus::core::ElementId>,
) {
    let data = Rc::new(Capture::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let targets = targets(dom.rebuild_to_vec());
    settle(&mut dom);
    for id in ["help-pill", "help-chip"] {
        dispatch(&dom, "mounted", targets[id], ());
    }
    settle(&mut dom);
    (dom, data, targets)
}
fn description(html: &str, id: &str, tag: &str) -> String {
    attr(
        opening(html, tag, &format!("id=\"{id}\"")),
        "aria-describedby",
    )
    .unwrap()
    .split_whitespace()
    .last()
    .unwrap()
    .to_string()
}
fn panel_open(html: &str, id: &str) -> bool {
    attr(opening(html, "div", &format!("id=\"{id}\"")), "data-open") == Some("true")
}
#[test]
fn status_keyboard_help_uses_pill_and_chip_controls_and_escape_preserves_actions() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, targets) = setup();
    let html = dioxus_ssr::render(&dom);
    let pill = description(&html, "help-pill", "span");
    let chip = description(&html, "help-chip", "button");
    assert_eq!(
        attr(opening(&html, "span", "id=\"help-pill\""), "tabindex"),
        Some("0")
    );
    dispatch(&dom, "focusin", targets["help-pill"], ());
    settle(&mut dom);
    assert!(panel_open(&dioxus_ssr::render(&dom), &pill));
    dispatch(&dom, "keydown", targets["help-pill"], Key::Escape);
    settle(&mut dom);
    assert!(!panel_open(&dioxus_ssr::render(&dom), &pill));
    dispatch(&dom, "focusin", targets["close"], ());
    settle(&mut dom);
    assert!(
        panel_open(&dioxus_ssr::render(&dom), &chip),
        "close-button focus reaches its main-mounted shared help"
    );
    dispatch(&dom, "keydown", targets["close"], Key::Escape);
    settle(&mut dom);
    assert!(!panel_open(&dioxus_ssr::render(&dom), &chip));
    dispatch(&dom, "click", targets["help-chip"], ());
    assert!(
        !dispatch(&dom, "click", targets["close"], ()),
        "close still stops click propagation"
    );
    assert_eq!(*data.main.borrow(), [0]);
    assert_eq!(*data.close.borrow(), [0]);
    dom.in_runtime(|| data.mode.get().unwrap().set(3));
    settle(&mut dom);
    dispatch(&dom, "click", targets["help-chip"], ());
    dispatch(&dom, "click", targets["close"], ());
    assert_eq!(*data.main.borrow(), [0]);
    assert_eq!(*data.close.borrow(), [0, 3]);
    dispatch(&dom, "focusin", targets["close"], ());
    settle(&mut dom);
    assert!(panel_open(&dioxus_ssr::render(&dom), &chip));
}
#[test]
fn status_help_text_and_removal_keep_description_tokens_and_private_state_current() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, targets) = setup();
    let html = dioxus_ssr::render(&dom);
    let panel = description(&html, "help-pill", "span");
    dom.in_runtime(|| data.note.get().unwrap().set("Retained note".into()));
    dispatch(&dom, "focusin", targets["help-pill"], ());
    settle(&mut dom);
    dom.in_runtime(|| data.mode.get().unwrap().set(1));
    settle(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Current guidance 1"));
    assert!(!html.contains("Current guidance 0"));
    assert_eq!(description(&html, "help-pill", "span"), panel);
    assert!(html.contains("Retained note"));
    dom.in_runtime(|| data.mode.get().unwrap().set(2));
    settle(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("role=\"tooltip\""));
    assert_eq!(
        attr(
            opening(&html, "span", "id=\"help-pill\""),
            "aria-describedby"
        ),
        Some("caller-pill")
    );
    assert_eq!(
        attr(opening(&html, "span", "id=\"help-pill\""), "tabindex"),
        None
    );
    assert_eq!(
        attr(
            opening(&html, "button", "id=\"help-chip\""),
            "aria-describedby"
        ),
        Some("caller-chip")
    );
    assert!(html.contains("Retained note"));
    assert_eq!((data.mounts.get(), data.drops.get()), (1, 0));
    dom.in_runtime(|| data.mode.get().unwrap().set(4));
    settle(&mut dom);
    assert_eq!(data.drops.get(), 1);
    assert!(!dioxus_ssr::render(&dom).contains("Current guidance"));
}
