use super::*;

fn time(hour: u8, minute: u8, second: u8, millisecond: u16) -> IxTime {
    IxTime::new(hour, minute, second, millisecond).unwrap()
}
fn section<'a>(html: &'a str, id: &str) -> &'a str {
    html.split_once(&format!("data-clock-case=\"{id}\""))
        .unwrap()
        .1
        .split("</section>")
        .next()
        .unwrap()
}
fn localized() -> TimeLabels {
    TimeLabels {
        header: "Inspection time".into(),
        am: "Vorm.".into(),
        pm: "Nachm.".into(),
        confirm: "Accept inspection".into(),
        ..Default::default()
    }
}
#[test]
fn heading_period_and_confirmation_are_independent_with_legacy_defaults_and_one_combined_done() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            section {"data-clock-case":"legacy",TimePicker {format:TimeFormat::Hms12,ontimechange:move |_|{}}}
            section {"data-clock-case":"embedded",TimePicker {format:TimeFormat::Hms12,embedded:true,ontimechange:move |_|{}}}
            section {"data-clock-case":"hidden",TimePicker {format:TimeFormat::Hms12,hide_header:true,embedded:true,has_footer:true,labels:localized(),ontimechange:move |_|{}}}
            section {"data-clock-case":"suppressed",TimePicker {format:TimeFormat::Hms12,has_footer:false,ontimechange:move |_|{}}}
            section {"data-clock-case":"24h",TimePicker {hide_header:true,format:TimeFormat::Hms,ontimechange:move |_|{}}}
            section {"data-clock-case":"combined",DateTimePicker {today:IxDate::new(2026,10,21).unwrap(),from:IxDate::new(2026,10,21).unwrap(),single_selection:true,time_format:TimeFormat::Hms12,time_labels:localized(),ondatechange:move |_|{},ontimechange:move |_|{},onselect:move |_|{}}}
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let legacy = section(&html, "legacy");
    assert!(legacy.contains("ix-clock-header") && legacy.contains("ix-date-card-footer"));
    assert_eq!(legacy.matches("ix-clock-meridiem").count(), 1);
    assert!(!legacy.contains("ix-clock-period-body"));
    let embedded = section(&html, "embedded");
    assert!(embedded.contains("ix-clock-header") && !embedded.contains("ix-date-card-footer"));
    let hidden = section(&html, "hidden");
    assert!(!hidden.contains("ix-date-card-header") && !hidden.contains("Inspection time"));
    assert!(hidden.contains("ix-clock-period-body") && hidden.contains("Accept inspection"));
    assert_eq!(hidden.matches("ix-clock-meridiem").count(), 1);
    assert!(hidden.contains("aria-label=\"Vorm.\"") && hidden.contains("aria-label=\"Nachm.\""));
    assert!(!section(&html, "suppressed").contains("ix-date-card-footer"));
    let full_day = section(&html, "24h");
    assert!(!full_day.contains("ix-clock-meridiem") && !full_day.contains("ix-clock-period-body"));
    let combined = section(&html, "combined");
    assert_eq!(combined.matches("ix-date-card-footer").count(), 1);
    assert_eq!(combined.matches(">Done<").count(), 1);
    assert!(!combined.contains("Accept inspection"));
}

#[derive(Clone, PartialEq)]
struct ClockSettings {
    time: IxTime,
    hidden: bool,
    embedded: bool,
    footer: Option<bool>,
    disabled: bool,
    min: Option<IxTime>,
    max: Option<IxTime>,
    labels: TimeLabels,
}
impl Default for ClockSettings {
    fn default() -> Self {
        Self {
            time: time(9, 37, 42, 125),
            hidden: true,
            embedded: true,
            footer: Some(true),
            disabled: false,
            min: None,
            max: None,
            labels: localized(),
        }
    }
}
#[derive(Default)]
struct Observation {
    settings: Cell<Option<Signal<ClockSettings>>>,
    requests: RefCell<Vec<IxTime>>,
    confirmations: RefCell<Vec<IxTime>>,
}
fn fixture(data: Rc<Observation>) -> Element {
    let settings = use_signal(ClockSettings::default);
    data.settings.set(Some(settings));
    let current = settings();
    let changes = data.clone();
    let confirmations = data.clone();
    rsx! {TimePicker {time:current.time,format:TimeFormat::Hms12,hide_header:current.hidden,embedded:current.embedded,has_footer:current.footer,disabled:current.disabled,min_time:current.min,max_time:current.max,labels:current.labels,
        ontimechange:move|next|changes.requests.borrow_mut().push(next),onselect:move|next|confirmations.confirmations.borrow_mut().push(next)
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
    fn modifiers(&self) -> dioxus::html::Modifiers {
        dioxus::html::Modifiers::empty()
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
macro_rules! converter{
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* })=>{
        impl dioxus::html::HtmlEventConverter for Converter{$(fn $convert(&self,_:&dioxus::html::PlatformEventData)->dioxus::html::$data{converter!(@data $group)})*}
    };
    (@data Mouse)=>{MouseData::new(Mouse)};
    (@data Mounted)=>{MountedData::new(())};
    (@data $other:ident)=>{panic!("Unexpected clock composition event")};
}
dioxus::html::with_html_event_groups!(converter);
fn collect(edits: dioxus::core::Mutations, attrs: &mut BTreeMap<usize, BTreeMap<String, String>>) {
    // This is a known registration index, not a second DOM renderer. Template
    // stack movement invalidates the current registration before static attrs.
    let mut current = None;
    for edit in edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } => {
                attrs.insert(id.raw(), BTreeMap::new());
                current = Some(id);
            }
            dioxus::core::Mutation::PushId { id } => current = Some(id),
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } => {
                if let Some(id) = current {
                    attrs.entry(id.raw()).or_default().insert(name, value);
                }
            }
            dioxus::core::Mutation::CreateElement { .. }
            | dioxus::core::Mutation::CreateText { .. }
            | dioxus::core::Mutation::Child { .. }
            | dioxus::core::Mutation::Pop
            | dioxus::core::Mutation::Clone
            | dioxus::core::Mutation::AppendChildren { .. }
            | dioxus::core::Mutation::ReplaceWith { .. }
            | dioxus::core::Mutation::InsertBefore { .. }
            | dioxus::core::Mutation::InsertAfter { .. }
            | dioxus::core::Mutation::Remove => current = None,
            _ => {}
        }
    }
}
fn settle(dom: &mut VirtualDom, attrs: &mut BTreeMap<usize, BTreeMap<String, String>>) {
    for _ in 0..4 {
        collect(dom.render_immediate_to_vec(), attrs);
        dom.process_events();
    }
}
fn setup() -> (
    VirtualDom,
    Rc<Observation>,
    BTreeMap<usize, BTreeMap<String, String>>,
) {
    let data = Rc::new(Observation::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut attrs = BTreeMap::new();
    collect(dom.rebuild_to_vec(), &mut attrs);
    settle(&mut dom, &mut attrs);
    (dom, data, attrs)
}
fn target(
    attrs: &BTreeMap<usize, BTreeMap<String, String>>,
    name: &str,
    value: &str,
) -> dioxus::core::ElementId {
    dioxus::core::ElementId::from_raw(
        *attrs
            .iter()
            .find(|(_, map)| map.get(name).is_some_and(|actual| actual == value))
            .unwrap_or_else(|| panic!("missing {name}={value}"))
            .0,
    )
}
fn click(dom: &VirtualDom, id: dioxus::core::ElementId) {
    let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new(())));
    dom.runtime()
        .handle_event("click", Event::new(data, true), id);
}
fn confirm_target(
    dom: &VirtualDom,
    attrs: &BTreeMap<usize, BTreeMap<String, String>>,
) -> dioxus::core::ElementId {
    let html = dioxus_ssr::render(dom);
    let footer = html.split_once("class=\"ix-date-card-footer\"").unwrap().1;
    let button = footer
        .split('<')
        .find(|tag| tag.starts_with("button "))
        .unwrap();
    let id = button
        .split_once("id=\"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    target(attrs, "id", id)
}
fn native_period<'a>(html: &'a str, label: &str) -> &'a str {
    html.split('<')
        .find(|tag| tag.starts_with("button ") && tag.contains(&format!("aria-label=\"{label}\"")))
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
#[test]
fn hidden_heading_period_uses_current_controlled_time_and_native_button_semantics() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, mut attrs) = setup();
    let mut settings = data.settings.get().unwrap();
    let html = dioxus_ssr::render(&dom);
    let pm = native_period(&html, "Nachm.");
    assert_eq!(
        ["type", "tabindex", "aria-pressed"].map(|name| attribute_value(pm, name)),
        [Some("button"), Some("0"), Some("false")],
        "{pm}"
    );
    click(&dom, target(&attrs, "aria-label", "Nachm."));
    assert_eq!(&*data.requests.borrow(), &[time(21, 37, 42, 125)]);
    assert_eq!(
        settings.peek().time,
        time(9, 37, 42, 125),
        "listener requests do not replace the controlled draft"
    );
    dom.in_runtime(|| {
        settings.with_mut(|value| {
            value.time = time(23, 12, 4, 750);
            value.hidden = false;
            value.embedded = false;
        })
    });
    settle(&mut dom, &mut attrs);
    assert!(!dioxus_ssr::render(&dom).contains("ix-clock-period-body"));
    click(&dom, target(&attrs, "aria-label", "Vorm."));
    assert_eq!(data.requests.borrow().last(), Some(&time(11, 12, 4, 750)));
    click(&dom, confirm_target(&dom, &attrs));
    assert_eq!(&*data.confirmations.borrow(), &[time(23, 12, 4, 750)]);
    dom.in_runtime(|| {
        settings.with_mut(|value| {
            value.time = time(6, 1, 2, 3);
            value.hidden = true;
            value.labels.am = "Morning".into();
            value.labels.pm = "Evening".into();
        })
    });
    settle(&mut dom, &mut attrs);
    click(&dom, target(&attrs, "aria-label", "Evening"));
    assert_eq!(data.requests.borrow().last(), Some(&time(18, 1, 2, 3)));
    click(&dom, confirm_target(&dom, &attrs));
    assert_eq!(data.confirmations.borrow().last(), Some(&time(6, 1, 2, 3)));
}
#[test]
fn hidden_period_and_confirmation_preserve_disabled_bounds_and_explicit_footer_guards() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, mut attrs) = setup();
    let mut settings = data.settings.get().unwrap();
    dom.in_runtime(|| {
        settings.with_mut(|value| {
            value.time = time(13, 10, 20, 30);
            value.min = Some(time(10, 0, 0, 0));
            value.max = Some(time(14, 0, 0, 0));
        })
    });
    settle(&mut dom, &mut attrs);
    assert!(native_period(&dioxus_ssr::render(&dom), "Vorm.").contains("disabled"));
    click(&dom, target(&attrs, "aria-label", "Vorm."));
    assert!(data.requests.borrow().is_empty());
    click(&dom, target(&attrs, "aria-label", "Nachm."));
    assert_eq!(data.requests.borrow().last(), Some(&time(13, 10, 20, 30)));
    dom.in_runtime(|| settings.with_mut(|value| value.disabled = true));
    settle(&mut dom, &mut attrs);
    data.requests.borrow_mut().clear();
    click(&dom, target(&attrs, "aria-label", "Nachm."));
    click(&dom, confirm_target(&dom, &attrs));
    assert!(data.requests.borrow().is_empty() && data.confirmations.borrow().is_empty());
    dom.in_runtime(|| {
        settings.with_mut(|value| {
            value.disabled = false;
            value.time = time(20, 0, 0, 0);
        })
    });
    settle(&mut dom, &mut attrs);
    click(&dom, confirm_target(&dom, &attrs));
    assert!(
        data.confirmations.borrow().is_empty(),
        "out-of-bounds current time cannot confirm"
    );
    dom.in_runtime(|| settings.with_mut(|value| value.footer = Some(false)));
    settle(&mut dom, &mut attrs);
    assert!(!dioxus_ssr::render(&dom).contains("ix-date-card-footer"));
}
