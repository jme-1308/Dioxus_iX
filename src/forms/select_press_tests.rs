// Registered Dioxus handlers exercise clicks retargeted to the presentation
// wrapper. Trusted browser/native workflows cover blur-driven layout movement.
use std::cell::Cell;

struct SelectClickFocus;
impl dioxus::html::HasFocusData for SelectClickFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct SelectClickConverter;
macro_rules! select_click_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for SelectClickConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {
                let _ = event;
                select_click_converter!(@data $group,$convert,event)
            })*
        }
    };
    (@data Focus,$convert:ident,$event:ident) => { dioxus::html::FocusData::new(SelectClickFocus) };
    (@data $other:ident,$convert:ident,$event:ident) => {
        <PaneFormsConverter as dioxus::html::HtmlEventConverter>::$convert(&PaneFormsConverter,$event)
    };
}
dioxus::html::with_html_event_groups!(select_click_converter);

#[derive(Default)]
struct SelectClickData {
    disabled: Cell<bool>,
    readonly: Cell<bool>,
    value_calls: Cell<usize>,
    values_calls: Cell<usize>,
}
fn select_click_fixture(data: Rc<SelectClickData>) -> Element {
    let values_data = data.clone();
    rsx! {
        Select {
            id: "click-select", disabled: data.disabled.get(), readonly: data.readonly.get(),
            options: vec![SelectOption::new("a", "Alpha"), SelectOption::new("b", "Beta")],
            onvaluechange: move |_| data.value_calls.set(data.value_calls.get() + 1),
            onvalueschange: move |_| values_data.values_calls.set(values_data.values_calls.get() + 1),
        }
    }
}
fn select_click_dom() -> (VirtualDom, Rc<SelectClickData>, dioxus::core::ElementId, dioxus::core::ElementId) {
    let data = Rc::new(SelectClickData::default());
    let mut dom = VirtualDom::new_with_props(select_click_fixture, data.clone());
    let edits = dom.rebuild_to_vec().edits;
    let mut current = dioxus::core::ElementId::ROOT;
    let mut input = None;
    let mut wrapper = None;
    for edit in edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => current = id,
            dioxus::core::Mutation::SetAttribute {
                name, value: dioxus::core::AttributeValue::Text(value), ..
            } if name == "id" && value == "click-select" => input = Some(current),
            // The presentation wrapper is Select's first click handler; later
            // handlers belong to its independent chevron/clear actions.
            dioxus::core::Mutation::NewEventListener { name } if name == "click" && wrapper.is_none() => wrapper = Some(current),
            _ => {}
        }
    }
    let input = input.unwrap();
    let wrapper = wrapper.unwrap();
    assert_ne!(input, wrapper, "the actual input has no separate click activation");
    (dom, data, input, wrapper)
}
fn assert_select_click_open(dom: &VirtualDom, expanded: bool) {
    let html = dioxus_ssr::render(dom);
    assert_eq!(feedback_attribute(feedback_tag(&html, "click-select"), "aria-expanded"), Some(if expanded { "true" } else { "false" }));
}

#[test]
fn select_presentation_click_opens_focused_input_without_canceling_native_defaults() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(SelectClickConverter));
    let (mut dom, data, input, wrapper) = select_click_dom();
    assert!(feedback_event(&dom, "click", wrapper, ()));
    feedback_refresh(&mut dom);
    assert_select_click_open(&dom, false);
    feedback_event(&dom, "focus", input, ());
    feedback_refresh(&mut dom);
    assert_select_click_open(&dom, false);
    assert!(feedback_event(&dom, "click", wrapper, ()));
    feedback_refresh(&mut dom);
    assert_select_click_open(&dom, true);
    feedback_event(&dom, "keydown", input, Key::ArrowDown);
    feedback_refresh(&mut dom);
    let before = dioxus_ssr::render(&dom);
    let active = feedback_attribute(feedback_tag(&before, "click-select"), "aria-activedescendant").unwrap().to_string();
    assert!(feedback_event(&dom, "click", wrapper, ()));
    feedback_refresh(&mut dom);
    let after = dioxus_ssr::render(&dom);
    assert_eq!(feedback_attribute(feedback_tag(&after, "click-select"), "aria-activedescendant"), Some(active.as_str()));
    feedback_event(&dom, "input", input, String::from("Beta"));
    feedback_refresh(&mut dom);
    assert!(feedback_event(&dom, "click", wrapper, ()));
    feedback_refresh(&mut dom);
    let filtered = dioxus_ssr::render(&dom);
    assert_eq!(feedback_attribute(feedback_tag(&filtered, "click-select"), "value"), Some("Beta"));
    assert_select_click_open(&dom, true);
    assert_eq!(data.value_calls.get(), 0);
    assert_eq!(data.values_calls.get(), 0);
    feedback_event(&dom, "keydown", input, Key::Escape);
    feedback_event(&dom, "blur", input, ());
    feedback_refresh(&mut dom);
    assert!(feedback_event(&dom, "click", wrapper, ()));
    feedback_refresh(&mut dom);
    assert_select_click_open(&dom, false);
}

#[test]
fn select_presentation_click_follows_live_disabled_and_readonly_guards() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(SelectClickConverter));
    let (mut dom, data, input, wrapper) = select_click_dom();
    feedback_event(&dom, "focus", input, ());
    for (disabled, readonly) in [(true, false), (false, true), (true, true)] {
        data.disabled.set(disabled);
        data.readonly.set(readonly);
        feedback_refresh(&mut dom);
        assert!(feedback_event(&dom, "click", wrapper, ()));
        feedback_refresh(&mut dom);
        assert_select_click_open(&dom, false);
        let html = dioxus_ssr::render(&dom);
        let input = feedback_tag(&html, "click-select");
        assert_eq!(feedback_attribute(input, "disabled"), disabled.then_some("true"));
        assert_eq!(feedback_attribute(input, "readonly"), readonly.then_some("true"));
    }
    data.disabled.set(false);
    data.readonly.set(false);
    feedback_refresh(&mut dom);
    assert!(feedback_event(&dom, "click", wrapper, ()));
    feedback_refresh(&mut dom);
    assert_select_click_open(&dom, true);
    assert_eq!(data.value_calls.get(), 0);
    assert_eq!(data.values_calls.get(), 0);
}
