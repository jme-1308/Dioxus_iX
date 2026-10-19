//! Exercise the real composer listener; browser IME integration is separate.
use super::*;
use std::{cell::Cell, cell::RefCell, rc::Rc};

#[derive(Default)]
struct ComposerCapture {
    value: Cell<Option<Signal<String>>>,
    events: RefCell<Vec<(&'static str, String)>>,
}

fn composer_fixture() -> Element {
    let capture = consume_context::<Rc<ComposerCapture>>();
    let mut value = use_signal(String::new);
    capture.value.set(Some(value));
    let changes = capture.clone();
    rsx! {
        ChatInput {
            id: "composition-prompt",
            value: value(),
            onvaluechange: move |next| value.set(next),
            onchange: move |next| changes.events.borrow_mut().push(("change", next)),
            onpromptsubmit: move |next| capture.events.borrow_mut().push(("submit", next)),
        }
    }
}

#[derive(Clone)]
struct ComposerKey {
    composing: bool,
    modifiers: Modifiers,
}
impl dioxus::html::ModifiersInteraction for ComposerKey {
    fn modifiers(&self) -> Modifiers {
        self.modifiers
    }
}
impl dioxus::html::HasKeyboardData for ComposerKey {
    fn key(&self) -> Key {
        Key::Enter
    }
    fn code(&self) -> dioxus::html::Code {
        dioxus::html::Code::Enter
    }
    fn location(&self) -> dioxus::html::Location {
        dioxus::html::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        self.composing
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct ComposerConverter;
macro_rules! composer_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for ComposerConverter {
            $(fn $convert(&self, event: &dioxus::html::PlatformEventData) -> dioxus::html::$data {
                let _ = event;
                composer_converter!(@data $group, event)
            })*
        }
    };
    (@data Keyboard, $event:ident) => {
        dioxus::html::KeyboardData::new($event.downcast::<ComposerKey>().unwrap().clone())
    };
    (@data $other:ident, $event:ident) => { panic!("Unexpected composer keyboard test event") };
}
dioxus::html::with_html_event_groups!(composer_converter);

#[test]
fn composing_enter_preserves_input_then_normal_enter_commits_and_submits_once() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ComposerConverter));
    let capture = Rc::new(ComposerCapture::default());
    let mut dom = VirtualDom::new(composer_fixture).with_root_context(capture.clone());
    let mut current = dioxus::core::ElementId::ROOT;
    let mut textarea = None;
    for edit in dom.rebuild_to_vec().edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id;
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" && value == "composition-prompt" => {
                textarea = Some(current);
            }
            _ => {}
        }
    }
    let textarea = textarea.expect("actual ChatInput textarea with its keyboard listener");
    // A controlled host update leaves an uncommitted value. An IME confirmation
    // must not commit or submit that value; the next ordinary Enter may do both.
    let prompt = "  次の点検を確認してください  ";
    dom.in_scope(ScopeId::ROOT, || {
        capture.value.get().unwrap().set(prompt.into());
    });
    for _ in 0..3 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
    let enter = |composing, modifiers| {
        let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new(
            ComposerKey {
                composing,
                modifiers,
            },
        )));
        let event = Event::new(data, true);
        dom.runtime()
            .handle_event("keydown", event.clone(), textarea);
        event.default_action_enabled()
    };
    assert!(
        enter(true, Modifiers::empty()),
        "IME keeps its native default"
    );
    assert!(
        capture.events.borrow().is_empty(),
        "IME does not commit or submit"
    );
    assert!(
        enter(false, Modifiers::SHIFT),
        "modified Enter keeps newline handling"
    );
    assert!(capture.events.borrow().is_empty());
    assert!(
        !enter(false, Modifiers::empty()),
        "send cancels native Enter"
    );
    assert_eq!(
        capture.events.borrow().as_slice(),
        [
            ("change", prompt.to_string()),
            ("submit", prompt.to_string())
        ],
        "ordinary Enter emits one unchanged payload, committing before submission"
    );
    assert_eq!(
        (capture.value.get().unwrap())(),
        prompt,
        "host value is preserved"
    );
}
