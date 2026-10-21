use super::*;

fn date(day: u8) -> IxDate {
    IxDate::new(2026, 10, day).unwrap()
}
fn time(hour: u8) -> IxTime {
    IxTime::new(hour, 30, 0, 0).unwrap()
}
fn parse_date(value: &str, prefix: &str) -> Option<IxDate> {
    DateFormat::Iso.parse(value.strip_prefix(prefix)?)
}
fn parse_time(value: &str, prefix: &str) -> Option<IxTime> {
    TimeFormat::Hms.parse(value.strip_prefix(prefix)?)
}
fn parse_datetime(value: &str, prefix: &str) -> Option<IxDateTime> {
    let (date, time) = value.strip_prefix(prefix)?.split_once('@')?;
    Some(IxDateTime {
        date: DateFormat::Iso.parse(date)?,
        time: TimeFormat::Hms.parse(time)?,
    })
}
#[test]
fn custom_parse_authority_keeps_required_date_time_and_boundary_date_constraints() {
    let results = Rc::new(RefCell::new(Vec::new()));
    let capture = results.clone();
    let mut dom = VirtualDom::new_with_props(
        move |capture: Rc<RefCell<Vec<DateInputValidity>>>| {
            let custom = DateTextFormat::new(
                use_callback(|value: String| parse_date(&value, "D")),
                use_callback(|value| format!("D{}", DateFormat::Iso.format(value))),
            );
            let rejected = parse_text("2026/10/20", Some(custom), |value| {
                DateFormat::default().parse(value)
            });
            let before = parse_text("D2026-10-19", Some(custom), |value| {
                DateFormat::default().parse(value)
            });
            *capture.borrow_mut() = vec![
                date_input_validity("2026/10/20", rejected, false, None, None),
                date_input_validity("D2026-10-19", before, false, Some(date(20)), None),
                date_input_validity("", None, true, None, None),
                time_input_validity(
                    "host-time",
                    Some(time(8)),
                    false,
                    Some(time(9)),
                    Some(time(17)),
                ),
                time_input_validity(
                    "host-time",
                    Some(time(8)),
                    false,
                    Some(time(17)),
                    Some(time(9)),
                ),
                datetime_input_validity(
                    "host-combined",
                    Some(IxDateTime {
                        date: date(20),
                        time: time(8),
                    }),
                    false,
                    (Some(date(20)), Some(date(22))),
                    (Some(time(9)), Some(time(17))),
                ),
                datetime_input_validity(
                    "host-combined",
                    Some(IxDateTime {
                        date: date(21),
                        time: time(8),
                    }),
                    false,
                    (Some(date(20)), Some(date(22))),
                    (Some(time(9)), Some(time(17))),
                ),
            ];
            let value = date(20);
            assert_eq!(
                parse_text(
                    &format_text(value, Some(custom), |value| value.to_string()),
                    Some(custom),
                    |_| None
                ),
                Some(value)
            );
            rsx! {span {"Pure host codecs"}}
        },
        capture,
    );
    dom.rebuild_in_place();
    let results = results.borrow();
    assert!(
        results[0].bad_input,
        "rejected custom parse never retries valid preset text"
    );
    assert!(results[1].range_underflow && results[2].value_missing && results[3].range_underflow);
    assert!(
        results[4].valid(),
        "reversed time bounds retain existing normalization"
    );
    assert!(
        results[5].range_underflow && results[6].valid(),
        "time bounds remain scoped to boundary dates"
    );
}
#[derive(Clone, Debug, PartialEq)]
enum Typed {
    Date(Option<IxDate>),
    Time(Option<IxTime>),
    Combined(Option<IxDateTime>),
}
#[derive(Default)]
struct Capture {
    kind: usize,
    mode: Cell<Option<Signal<bool>>>,
    raw: Cell<Option<Signal<String>>>,
    values: RefCell<Vec<String>>,
    typed: RefCell<Vec<Typed>>,
    validity: RefCell<Vec<DateInputValidity>>,
}
fn expected(kind: usize) -> Typed {
    match kind {
        0 => Typed::Date(Some(date(20))),
        1 => Typed::Time(Some(time(13))),
        _ => Typed::Combined(Some(IxDateTime {
            date: date(20),
            time: time(13),
        })),
    }
}
fn initial(kind: usize, changed: bool) -> String {
    match (kind, changed) {
        (0, false) => "D2026-10-20",
        (0, true) => "E2026-10-20",
        (1, false) => "H13:30:00",
        (1, true) => "J13:30:00",
        (_, false) => "P2026-10-20@13:30:00",
        (_, true) => "Q2026-10-20@13:30:00",
    }
    .into()
}
fn fixture(data: Rc<Capture>) -> Element {
    let mode = use_signal(|| false);
    let mut raw = use_signal(|| initial(data.kind, false));
    data.mode.set(Some(mode));
    data.raw.set(Some(raw));
    let changed = mode();
    let date_codec = DateTextFormat {
        revision: u64::from(changed),
        parse: use_callback(move |value: String| {
            parse_date(&value, if changed { "E" } else { "D" })
        }),
        format: use_callback(move |value| {
            format!(
                "{}{}",
                if changed { "E" } else { "D" },
                DateFormat::Iso.format(value)
            )
        }),
    };
    let time_codec = TimeTextFormat {
        revision: u64::from(changed),
        parse: use_callback(move |value: String| {
            parse_time(&value, if changed { "J" } else { "H" })
        }),
        format: use_callback(move |value| {
            format!(
                "{}{}",
                if changed { "J" } else { "H" },
                TimeFormat::Hms.format(value)
            )
        }),
    };
    let combined_codec = DateTimeTextFormat {
        revision: u64::from(changed),
        parse: use_callback(move |value: String| {
            parse_datetime(&value, if changed { "Q" } else { "P" })
        }),
        format: use_callback(move |value: IxDateTime| {
            format!(
                "{}{}@{}",
                if changed { "Q" } else { "P" },
                DateFormat::Iso.format(value.date),
                TimeFormat::Hms.format(value.time)
            )
        }),
    };
    let values = data.clone();
    let typed = data.clone();
    let validity = data.clone();
    let onvalue = use_callback(move |value: String| {
        values.values.borrow_mut().push(value.clone());
        raw.set(value);
    });
    let onvalidity = use_callback(move |value| validity.validity.borrow_mut().push(value));
    match data.kind {
        0 => {
            rsx! {DateInput {id:"custom",today:date(19),value:raw(),text_format:date_codec,min_date:date(19),max_date:date(22),onvaluechange:onvalue,onvaliditychange:onvalidity,ondatechange:move|value|typed.typed.borrow_mut().push(Typed::Date(value))}}
        }
        1 => {
            rsx! {TimeInput {id:"custom",value:raw(),text_format:time_codec,min_time:time(9),max_time:time(17),onvaluechange:onvalue,onvaliditychange:onvalidity,ontimechange:move|value|typed.typed.borrow_mut().push(Typed::Time(value))}}
        }
        _ => {
            rsx! {DateTimeInput {id:"custom",today:date(19),value:raw(),text_format:combined_codec,min_date:date(19),max_date:date(22),min_time:time(9),max_time:time(17),onvaluechange:onvalue,onvaliditychange:onvalidity,ondatetimechange:move|value|typed.typed.borrow_mut().push(Typed::Combined(value))}}
        }
    }
}
fn setup(kind: usize) -> (VirtualDom, Rc<Capture>) {
    let data = Rc::new(Capture {
        kind,
        ..Default::default()
    });
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    (dom, data)
}
fn settle(dom: &mut VirtualDom) {
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
// Inspect actual mounted VDOM listeners. No synthetic DOM/attribute inventory.
fn listener(
    dom: &VirtualDom,
    node: dioxus::core::MountedVNode<'_>,
    name: &str,
    value: &str,
    event: &str,
) -> Option<dioxus::core::ListenerCallback> {
    for anchor in node.template().anchors() {
        let attrs: Vec<_> = node.dynamic_attr_values()[anchor.attributes()]
            .iter()
            .flat_map(|slot| slot.iter())
            .collect();
        if attrs.iter().any(|attr|attr.name==name&&matches!(&attr.value,dioxus::core::AttributeValue::Text(text) if text.starts_with(value))){
            if let Some(callback)=attrs.iter().find_map(|attr|match &attr.value{dioxus::core::AttributeValue::Listener(callback) if attr.name==event=>Some(callback.clone()),_=>None}){return Some(callback);}
        }
    }
    node.dynamic_anchors()
        .flat_map(|anchor| anchor.nodes())
        .find_map(|slot| match &*slot {
            dioxus::core::DynamicNode::Component(component) => component
                .mounted_scope(slot, node, dom)
                .and_then(|scope| scope.try_mounted_root_node())
                .and_then(|child| listener(dom, child, name, value, event)),
            dioxus::core::DynamicNode::Fragment(_) => node
                .mounted_fragment_children(slot, dom)
                .into_iter()
                .find_map(|child| listener(dom, child, name, value, event)),
            _ => None,
        })
}
fn call(dom: &mut VirtualDom, name: &str, value: &str, event: &str, payload: impl std::any::Any) {
    let root = dom
        .get_scope(ScopeId::APP)
        .unwrap()
        .try_mounted_root_node()
        .unwrap();
    let callback = listener(dom, root, name, value, event)
        .unwrap_or_else(|| panic!("missing actual listener {name}={value} {event}"));
    dom.in_runtime(|| {
        callback.call(
            Event::new(
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))),
                true,
            )
            .into_any(),
        )
    });
    settle(dom);
}
fn input_tag(html: &str) -> &str {
    html.split('<')
        .find(|tag| tag.starts_with("input ") && tag.contains("id=\"custom\""))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
struct Form(String);
impl dioxus::html::HasFileData for Form {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for Form {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        Vec::new()
    }
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
struct CellMount;
impl dioxus::html::RenderedElementBacking for CellMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        _: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        Box::pin(async { Ok(()) })
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
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(Form($event.downcast::<String>().unwrap().clone()))};
    (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(Focus)};
    (@data Keyboard,$event:ident)=>{dioxus::html::KeyboardData::new(Keyboard($event.downcast::<Key>().unwrap().clone()))};
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(Mouse)};
    (@data Mounted,$event:ident)=>{MountedData::new(CellMount)};
    (@data $other:ident,$event:ident)=>{panic!("unexpected temporal text-format event")};
}
dioxus::html::with_html_event_groups!(converter);

#[test]
fn actual_custom_text_keeps_invalid_raw_and_typed_events_and_reads_current_parsers() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for kind in 0..3 {
        let (mut dom, data) = setup(kind);
        let invalid = match kind {
            0 => "2026/10/20",
            1 => "13:30:00",
            _ => "2026/10/20 13:30:00",
        };
        call(&mut dom, "id", "custom", "oninput", invalid.to_string());
        assert_eq!(
            data.values.borrow().last().map(String::as_str),
            Some(invalid)
        );
        assert_eq!(
            data.typed.borrow().last(),
            Some(&match kind {
                0 => Typed::Date(None),
                1 => Typed::Time(None),
                _ => Typed::Combined(None),
            })
        );
        assert!(data.validity.borrow().last().unwrap().bad_input);
        let html = dioxus_ssr::render(&dom);
        assert!(input_tag(&html).contains(invalid));
        assert!(input_tag(&html).contains("pattern="));
        dom.in_runtime(|| data.mode.get().unwrap().set(true));
        settle(&mut dom);
        call(&mut dom, "id", "custom", "oninput", initial(kind, true));
        assert_eq!(data.typed.borrow().last(), Some(&expected(kind)));
        assert!(data.validity.borrow().last().unwrap().valid());
        assert!(!input_tag(&dioxus_ssr::render(&dom)).contains("pattern="));
    }
}
#[test]
fn actual_custom_picker_preparation_and_commit_use_current_formatters_for_all_inputs() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for kind in 0..3 {
        let (mut dom, data) = setup(kind);
        call(&mut dom, "id", "custom-toggle", "onmounted", ());
        call(&mut dom, "id", "custom", "onkeydown", Key::ArrowDown);
        assert!(input_tag(&dioxus_ssr::render(&dom)).contains("aria-expanded=true"));
        if kind == 0 {
            let html = dioxus_ssr::render(&dom);
            let selected = html
                .split('<')
                .find(|tag| {
                    tag.starts_with("button ") && tag.contains("aria-label=\"20 October 2026\"")
                })
                .unwrap();
            assert!(
                selected.contains("aria-selected=true"),
                "calendar selection comes from custom text, not today"
            );
        }
        dom.in_runtime(|| data.mode.get().unwrap().set(true));
        settle(&mut dom);
        if kind == 0 {
            call(&mut dom, "aria-label", "20 October 2026", "onclick", ());
        } else {
            call(
                &mut dom,
                "class",
                "ix-button ix-button-standard",
                "onclick",
                (),
            );
        }
        assert_eq!(data.values.borrow().last(), Some(&initial(kind, true)));
        assert_eq!(data.typed.borrow().last(), Some(&expected(kind)));
        assert!(data.validity.borrow().last().unwrap().valid());
        assert!(!input_tag(&dioxus_ssr::render(&dom)).contains("pattern="));
    }
}

fn revision_fixture(data: Rc<Capture>) -> Element {
    let mode = use_signal(|| false);
    data.mode.set(Some(mode));
    let changed = mode();
    let codec = DateTextFormat::new(
        use_callback(move |value: String| {
            if value == "service" {
                Some(date(if changed { 21 } else { 20 }))
            } else {
                parse_date(&value, "D")
            }
        }),
        use_callback(|value| format!("D{}", DateFormat::Iso.format(value))),
    )
    .with_revision(u64::from(changed));
    rsx! {DateInput {id:"custom",today:date(19),value:"service",text_format:codec,min_date:date(21),max_date:date(22),
        onvaluechange:move|value|data.values.borrow_mut().push(value)
    }}
}
#[test]
fn codec_revision_refreshes_unchanged_raw_validity_and_picker_preparation() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Capture::default());
    let mut dom = VirtualDom::new_with_props(revision_fixture, data.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    assert!(
        input_tag(&dioxus_ssr::render(&dom)).contains("pattern="),
        "initial custom alias is before the minimum"
    );
    dom.in_runtime(|| data.mode.get().unwrap().set(true));
    settle(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(input_tag(&html).contains("value=\"service\""));
    assert!(
        !input_tag(&html).contains("pattern="),
        "revision reparses unchanged raw text using current configuration"
    );
    assert!(
        data.values.borrow().is_empty(),
        "configuration refresh does not emit an input edit"
    );
    call(&mut dom, "id", "custom-toggle", "onmounted", ());
    call(&mut dom, "id", "custom", "onkeydown", Key::ArrowDown);
    let html = dioxus_ssr::render(&dom);
    let selected = html
        .split('<')
        .find(|tag| tag.starts_with("button ") && tag.contains("aria-label=\"21 October 2026\""))
        .unwrap();
    assert!(
        selected.contains("aria-selected=true"),
        "picker opens on the reinterpreted current custom date"
    );
}
