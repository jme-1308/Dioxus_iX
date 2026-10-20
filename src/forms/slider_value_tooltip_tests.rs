// These component tests exercise the actual native-range handlers and shared
// Tooltip lifecycle. Browser/native workflows verify placement and clipping.
use std::cell::{Cell, RefCell};

struct SliderValueFocus;
impl dioxus::html::HasFocusData for SliderValueFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl dioxus::html::HasPointerData for GroupFeedbackMouse {
    fn pointer_id(&self) -> i32 {
        1
    }
    fn width(&self) -> f64 {
        1.
    }
    fn height(&self) -> f64 {
        1.
    }
    fn pressure(&self) -> f32 {
        0.
    }
    fn tangential_pressure(&self) -> f32 {
        0.
    }
    fn tilt_x(&self) -> i32 {
        0
    }
    fn tilt_y(&self) -> i32 {
        0
    }
    fn twist(&self) -> i32 {
        0
    }
    fn pointer_type(&self) -> String {
        "mouse".into()
    }
    fn is_primary(&self) -> bool {
        true
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct SliderValueMount;
impl dioxus::html::RenderedElementBacking for SliderValueMount {
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
                dioxus::html::geometry::euclid::Point2D::new(100., 100.),
                dioxus::html::geometry::euclid::Size2D::new(40., 20.),
            ))
        })
    }
}
struct SliderValueConverter;
macro_rules! slider_value_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for SliderValueConverter {
            $(fn $convert(&self, event:&dioxus::html::PlatformEventData) -> dioxus::html::$data {
                let _=event;
                slider_value_converter!(@data $group, $convert, event)
            })*
        }
    };
    (@data Focus, $convert:ident, $event:ident) => { dioxus::html::FocusData::new(SliderValueFocus) };
    (@data Mounted, $convert:ident, $event:ident) => { MountedData::new(SliderValueMount) };
    (@data Pointer, $convert:ident, $event:ident) => { dioxus::html::PointerData::new(GroupFeedbackMouse) };
    (@data $other:ident, $convert:ident, $event:ident) => {
        <PaneFormsConverter as dioxus::html::HtmlEventConverter>::$convert(&PaneFormsConverter, $event)
    };
}
dioxus::html::with_html_event_groups!(slider_value_converter);

struct SliderValueSettings {
    value: Cell<f64>,
    disabled: Cell<bool>,
    show_tooltip: Cell<bool>,
    feedback: Cell<bool>,
    visible: Cell<bool>,
    requests: RefCell<Vec<f64>>,
}
impl Default for SliderValueSettings {
    fn default() -> Self {
        Self {
            value: Cell::new(30.),
            disabled: Cell::new(false),
            show_tooltip: Cell::new(true),
            feedback: Cell::new(false),
            visible: Cell::new(true),
            requests: RefCell::new(Vec::new()),
        }
    }
}
fn slider_value_fixture(settings: Rc<SliderValueSettings>) -> Element {
    let value = settings.value.get();
    rsx! {
        if settings.visible.get() {
            Slider {label:"Flow rate",name:"rate",form:"external-order",value,
                disabled:settings.disabled.get(),show_tooltip:settings.show_tooltip.get(),
                helper_text:settings.feedback.get().then(||"Maintain safe flow".to_string()),
                show_text_as_tooltip:settings.feedback.get(),
                onvaluechange:move|value|{settings.value.set(value);settings.requests.borrow_mut().push(value);}
            }
        }
    }
}
struct SliderValueHarness {
    dom: VirtualDom,
    settings: Rc<SliderValueSettings>,
    range: dioxus::core::ElementId,
    id: String,
}
impl SliderValueHarness {
    fn new(feedback: bool) -> Self {
        let settings = Rc::new(SliderValueSettings::default());
        settings.feedback.set(feedback);
        let mut dom = VirtualDom::new_with_props(slider_value_fixture, settings.clone());
        let changes = dom.rebuild_to_vec();
        let html = dioxus_ssr::render(&dom);
        let id = feedback_attribute(slider_value_range(&html), "id")
            .unwrap()
            .to_string();
        let mut current = dioxus::core::ElementId::ROOT;
        let mut range = None;
        for edit in &changes.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = *id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if name == "id" && value == &id => range = Some(current),
                _ => {}
            }
        }
        mount_pane_forms(&mut dom, changes);
        feedback_refresh(&mut dom);
        Self {
            dom,
            settings,
            range: range.expect("native range event target"),
            id,
        }
    }
    fn event(&mut self, event: &str, payload: impl std::any::Any) -> bool {
        let allowed = feedback_event(&self.dom, event, self.range, payload);
        feedback_refresh(&mut self.dom);
        allowed
    }
    fn html(&self) -> String {
        dioxus_ssr::render(&self.dom)
    }
    fn panel_id(&self) -> String {
        format!("{}-value-tooltip", self.id)
    }
    fn panel_open(&self) -> bool {
        feedback_attribute(feedback_tag(&self.html(), &self.panel_id()), "data-open")
            == Some("true")
    }
}
fn slider_value_range(html: &str) -> &str {
    html.split('<')
        .filter_map(|part| part.split_once('>').map(|(tag, _)| tag))
        .find(|tag| tag.starts_with("input ") && feedback_attribute(tag, "type") == Some("range"))
        .expect("native range input")
}
fn slider_value_body<'a>(html: &'a str, id: &str) -> &'a str {
    html.split_once(&format!("id=\"{id}\""))
        .unwrap()
        .1
        .split("ix-tooltip-arrow")
        .next()
        .unwrap()
}

#[test]
fn slider_shared_value_panel_tracks_controlled_native_input_and_dismisses_without_replacing_it() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(SliderValueConverter));
    let mut h = SliderValueHarness::new(false);
    let initial = h.html();
    let panel_id = h.panel_id();
    assert!(!h.panel_open());
    assert_eq!(
        feedback_attribute(feedback_tag(&initial, &panel_id), "role"),
        Some("tooltip")
    );
    assert_eq!(
        feedback_attribute(feedback_tag(&initial, &panel_id), "popover"),
        Some("manual")
    );
    assert_eq!(
        feedback_attribute(slider_value_range(&initial), "name"),
        Some("rate")
    );
    assert_eq!(
        feedback_attribute(slider_value_range(&initial), "form"),
        Some("external-order")
    );
    assert!(h.event("focus", ()));
    assert!(
        h.panel_open(),
        "native keyboard focus opens the registered value panel"
    );
    assert!(
        h.event("keydown", Key::ArrowRight),
        "native range defaults remain available"
    );
    assert!(
        h.settings.requests.borrow().is_empty(),
        "keys do not synthesize native range edits"
    );
    h.event("input", String::from("42"));
    assert_eq!(*h.settings.requests.borrow(), [42.]);
    let updated = h.html();
    assert_eq!(
        feedback_attribute(slider_value_range(&updated), "value"),
        Some("42")
    );
    assert!(
        slider_value_body(&updated, &panel_id).contains(">42<"),
        "panel shows accepted controlled value"
    );
    assert_eq!(
        feedback_attribute(slider_value_range(&updated), "id"),
        Some(h.id.as_str())
    );
    h.event("keydown", Key::Escape);
    assert!(!h.panel_open());
    h.event("blur", ());
    h.event("focus", ());
    assert!(h.panel_open(), "later focus can open again after Escape");
    h.event("blur", ());
    assert!(!h.panel_open());
}

#[test]
fn slider_value_panel_guards_prop_changes_and_owner_removal() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(SliderValueConverter));
    let mut h = SliderValueHarness::new(false);
    h.event("pointerenter", ());
    assert!(
        h.panel_open(),
        "native pointer entry activates the thumb's value panel"
    );
    let panel_id = h.panel_id();
    h.settings.show_tooltip.set(false);
    feedback_refresh(&mut h.dom);
    let hidden = h.html();
    assert!(!hidden.contains(&format!("id=\"{panel_id}\"")));
    assert!(
        !feedback_attribute(slider_value_range(&hidden), "aria-describedby")
            .is_some_and(|value| value.split_whitespace().any(|token| token == panel_id))
    );
    h.event("focus", ());
    assert!(!h.html().contains(&format!("id=\"{panel_id}\"")));
    h.settings.show_tooltip.set(true);
    feedback_refresh(&mut h.dom);
    h.event("pointerenter", ());
    assert!(h.panel_open());
    h.settings.disabled.set(true);
    feedback_refresh(&mut h.dom);
    assert!(!h.html().contains(&format!("id=\"{panel_id}\"")));
    h.event("input", String::from("80"));
    h.event("focus", ());
    h.event("pointerenter", ());
    assert_eq!(h.settings.value.get(), 30.);
    assert!(h.settings.requests.borrow().is_empty());
    assert!(!h.html().contains(&format!("id=\"{panel_id}\"")));
    h.settings.disabled.set(false);
    feedback_refresh(&mut h.dom);
    h.event("blur", ());
    h.event("focus", ());
    assert!(h.panel_open());
    h.settings.visible.set(false);
    feedback_refresh(&mut h.dom);
    let removed = h.html();
    assert!(!removed.contains("role=\"tooltip\""));
    assert!(!removed.contains("type=\"range\""));
}

#[test]
fn slider_value_tooltip_and_validation_feedback_have_independent_lifetimes_and_descriptions() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(SliderValueConverter));
    let mut h = SliderValueHarness::new(true);
    let panel_id = h.panel_id();
    let feedback_id = format!("{}-feedback", h.id);
    let html = h.html();
    assert_eq!(html.matches("role=\"tooltip\"").count(), 2);
    let description = feedback_attribute(slider_value_range(&html), "aria-describedby").unwrap();
    for token in [&panel_id, &feedback_id] {
        assert_eq!(
            description
                .split_whitespace()
                .filter(|value| *value == token.as_str())
                .count(),
            1
        );
        assert_eq!(html.matches(&format!("id=\"{token}\"")).count(), 1);
    }
    assert!(html.contains("Maintain safe flow"));
    h.event("focus", ());
    assert!(h.panel_open());
    h.event("keydown", Key::Escape);
    assert!(!h.panel_open());
    assert!(h.html().contains(&format!("id=\"{feedback_id}\"")));
    h.settings.show_tooltip.set(false);
    feedback_refresh(&mut h.dom);
    let feedback_only = h.html();
    assert_eq!(feedback_only.matches("role=\"tooltip\"").count(), 1);
    assert_eq!(
        feedback_attribute(slider_value_range(&feedback_only), "aria-describedby"),
        Some(feedback_id.as_str())
    );
    assert!(feedback_only.contains("Maintain safe flow"));
}
