use super::*;

#[derive(Clone)]
struct TemporalCustomSettings {
    date_value: String,
    date_state: crate::FieldState,
    time_state: crate::FieldState,
    show: bool,
}
fn temporal_custom_fields(config: Rc<RefCell<TemporalCustomSettings>>) -> Element {
    let settings = config.borrow().clone();
    rsx! {
        section {"data-temporal-case":"date",
            crate::CustomField {label:"Service date",helper_text:"Outer date helper",invalid_text:"Outer date invalid",warning_text:"Outer date warning",
                if settings.show {DateInput {today:date(2026,10,16),value:settings.date_value,state:settings.date_state,onvaluechange:move |_|{}}}
            }
        }
        section {"data-temporal-case":"time",
            crate::CustomField {label:"Service time",helper_text:"Outer time helper",warning_text:"Outer time warning",invalid_text:"Outer time invalid",
                if settings.show {TimeInput {value:"12:00:00",state:settings.time_state,onvaluechange:move |_|{}}}
            }
        }
        section {"data-temporal-case":"independent",
            crate::CustomField {label:"Independent time",helper_text:"Independent helper",invalid_text:"Independent invalid",warning_text:"Independent warning",
                TimeInput {value:"12:00:00",onvaluechange:move |_|{}}
            }
        }
    }
}
fn temporal_custom_settings() -> Rc<RefCell<TemporalCustomSettings>> {
    Rc::new(RefCell::new(TemporalCustomSettings {
        date_value: "2026/02/30".into(),
        date_state: crate::FieldState::Warning,
        time_state: crate::FieldState::Warning,
        show: true,
    }))
}
fn settle_temporal_custom_fields(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
}
fn temporal_custom_case<'a>(html: &'a str, name: &str) -> &'a str {
    html.split_once(&format!("data-temporal-case=\"{name}\""))
        .expect("temporal field section")
        .1
        .split_once("</section>")
        .unwrap()
        .0
}
#[test]
fn temporal_errors_and_explicit_states_update_scoped_custom_field_feedback() {
    let config = temporal_custom_settings();
    let mut dom = VirtualDom::new_with_props(temporal_custom_fields, config.clone());
    dom.rebuild_in_place();
    settle_temporal_custom_fields(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(
        temporal_custom_case(&html, "date").contains("Outer date invalid"),
        "a date parsing error takes precedence over explicit warning"
    );
    assert!(temporal_custom_case(&html, "time").contains("Outer time warning"));
    let independent = temporal_custom_case(&html, "independent");
    assert!(independent.contains("Independent helper"));
    assert!(
        !independent.contains("Independent invalid")
            && !independent.contains("Independent warning")
    );
    config.borrow_mut().date_value = "2026/10/16".into();
    dom.mark_dirty(ScopeId::APP);
    settle_temporal_custom_fields(&mut dom);
    assert!(
        temporal_custom_case(&dioxus_ssr::render(&dom), "date").contains("Outer date warning"),
        "clearing the error reveals the explicit warning"
    );
    {
        let mut settings = config.borrow_mut();
        settings.date_state = crate::FieldState::Default;
        settings.time_state = crate::FieldState::Default;
    }
    dom.mark_dirty(ScopeId::APP);
    settle_temporal_custom_fields(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(temporal_custom_case(&html, "date").contains("Outer date helper"));
    assert!(temporal_custom_case(&html, "time").contains("Outer time helper"));
    assert!(!html.contains("Outer date invalid") && !html.contains("Outer time warning"));
}
#[test]
fn removing_and_remounting_temporal_controls_clears_custom_field_contributions() {
    let config = temporal_custom_settings();
    let mut dom = VirtualDom::new_with_props(temporal_custom_fields, config.clone());
    dom.rebuild_in_place();
    settle_temporal_custom_fields(&mut dom);
    assert!(dioxus_ssr::render(&dom).contains("Outer date invalid"));
    config.borrow_mut().show = false;
    dom.mark_dirty(ScopeId::APP);
    settle_temporal_custom_fields(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(temporal_custom_case(&html, "date").contains("Outer date helper"));
    assert!(temporal_custom_case(&html, "time").contains("Outer time helper"));
    {
        let mut settings = config.borrow_mut();
        settings.show = true;
        settings.date_value = "2026/10/16".into();
        settings.date_state = crate::FieldState::Default;
        settings.time_state = crate::FieldState::Default;
    }
    dom.mark_dirty(ScopeId::APP);
    settle_temporal_custom_fields(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(temporal_custom_case(&html, "date").contains("Outer date helper"));
    assert!(temporal_custom_case(&html, "time").contains("Outer time helper"));
    assert!(!html.contains("Outer date invalid") && !html.contains("Outer time warning"));
}

// Exercise the actual preset and trigger listeners with a listener-only
// parent, so notification timing cannot depend on a controlled prop update.
struct DateDropdownMouse;
impl dioxus::html::InteractionLocation for DateDropdownMouse {
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
impl dioxus::html::InteractionElementOffset for DateDropdownMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for DateDropdownMouse {
    fn modifiers(&self) -> dioxus::html::Modifiers {
        dioxus::html::Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for DateDropdownMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for DateDropdownMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct DateDropdownTestConverter;
macro_rules! date_dropdown_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for DateDropdownTestConverter {
                $(fn $convert(&self, _: &dioxus::html::PlatformEventData) -> dioxus::html::$data { date_dropdown_converter!(@data $group) })*
            }
        };
        (@data Mouse) => { MouseData::new(DateDropdownMouse) };
        (@data Mounted) => { MountedData::new(()) };
        (@data $other:ident) => { panic!("Unexpected event in date dropdown regression") };
    }
dioxus::html::with_html_event_groups!(date_dropdown_converter);

fn dropdown_event_fixture() -> Element {
    let events = consume_context::<Rc<RefCell<Vec<DateRangeChange>>>>();
    DateDropdown(DateDropdownProps {
        today: date(2026, 10, 16),
        from: None,
        to: None,
        min_date: None,
        max_date: None,
        options: vec![
            DateDropdownOption {
                id: "today".into(),
                label: "Today".into(),
                from: date(2026, 10, 16),
                to: date(2026, 10, 16),
            },
            DateDropdownOption {
                id: "week".into(),
                label: "Last week".into(),
                from: date(2026, 10, 10),
                to: date(2026, 10, 16),
            },
        ],
        date_range_id: "today".into(),
        format: DateFormat::default(),
        single_selection: false,
        disabled: false,
        loading: false,
        controller: None,
        enable_top_layer: false,
        id: None,
        class: String::new(),
        week_start_index: 0,
        show_week_numbers: false,
        variant: crate::ButtonVariant::default(),
        labels: CalendarLabels::default(),
        empty_label: "No range set".into(),
        custom_label: "Custom range".into(),
        onrangechange: EventHandler::new(move |event| events.borrow_mut().push(event)),
    })
}
fn dropdown_listener(
    dom: &VirtualDom,
    node: dioxus::core::MountedVNode<'_>,
    marker: &str,
    event: &str,
) -> Option<dioxus::dioxus_core::ListenerCallback> {
    for anchor in node.template().anchors() {
        // Dioxus can split one element's attributes between dynamic slots.
        // Join only slots belonging to this DOM anchor before identifying
        // the control and its listener.
        let attributes: Vec<_> = node.dynamic_attr_values()[anchor.attributes()]
            .iter()
            .flat_map(|slot| slot.iter())
            .collect();
        if attributes.iter().any(|attribute| attribute.name == marker || matches!(&attribute.value, dioxus::dioxus_core::AttributeValue::Text(text) if text.starts_with(marker))) {
                for attribute in attributes {
                    if attribute.name == event {
                        if let dioxus::dioxus_core::AttributeValue::Listener(listener) = &attribute.value { return Some(listener.clone()); }
                    }
                }
            }
    }
    node.dynamic_anchors()
        .flat_map(|anchor| anchor.nodes())
        .find_map(|slot| match &*slot {
            dioxus::core::DynamicNode::Component(component) => component
                .mounted_scope(slot, node, dom)
                .and_then(|scope| scope.try_mounted_root_node())
                .and_then(|child| dropdown_listener(dom, child, marker, event)),
            dioxus::dioxus_core::DynamicNode::Fragment(_) => node
                .mounted_fragment_children(slot, dom)
                .into_iter()
                .find_map(|child| dropdown_listener(dom, child, marker, event)),
            _ => None,
        })
}
fn click_dropdown(dom: &mut VirtualDom, marker: &str) {
    call_dropdown_listener(dom, marker, "onclick");
}
fn call_dropdown_listener(dom: &mut VirtualDom, marker: &str, event: &str) {
    let root = dom
        .get_scope(ScopeId::APP)
        .unwrap()
        .try_mounted_root_node()
        .unwrap();
    let listener = dropdown_listener(dom, root, marker, event).unwrap_or_else(|| {
        panic!(
            "dropdown control listener {marker}: {:?}",
            root.dynamic_attr_values()
        )
    });
    dom.in_runtime(|| {
        listener.call(
            Event::new(
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))),
                true,
            )
            .into_any(),
        )
    });
    settle_dropdown(dom);
}
fn settle_dropdown(dom: &mut VirtualDom) {
    for _ in 0..8 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
#[test]
fn date_dropdown_presets_notify_while_open_and_repeated_dismissals_retain_the_range() {
    // HtmlEventConverter is process-global. Exercise the actual listeners
    // in an isolated test process so other concurrently running components
    // (including chart mount regressions) keep their own event converters.
    const ISOLATED: &str = "DIOXUS_IX_DATE_DROPDOWN_EVENT_TEST";
    if std::env::var_os(ISOLATED).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "datetime::tests::date_dropdown_presets_notify_while_open_and_repeated_dismissals_retain_the_range", "--nocapture", "--test-threads=1"])
                .env(ISOLATED, "1")
                .output().expect("run isolated date dropdown regression");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let events = Rc::new(RefCell::new(Vec::<DateRangeChange>::new()));
    let mut dom = VirtualDom::new(dropdown_event_fixture).with_root_context(events.clone());
    dioxus::html::set_event_converter(Box::new(DateDropdownTestConverter));
    dom.rebuild_in_place();
    settle_dropdown(&mut dom);
    call_dropdown_listener(&mut dom, "aria-expanded", "onmounted");
    click_dropdown(&mut dom, "aria-expanded");
    assert!(events.borrow().is_empty());
    click_dropdown(&mut dom, "Last week:");
    assert_eq!(events.borrow().len(), 1);
    assert_eq!(events.borrow()[0].id, "week");
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("aria-expanded=true"));
    assert!(html.contains("2026/10/10 - 2026/10/16"));
    click_dropdown(&mut dom, "Last week:");
    assert_eq!(
        events.borrow().len(),
        2,
        "re-selecting a preset is another source notification"
    );
    click_dropdown(&mut dom, "aria-expanded");
    assert_eq!(
        events.borrow().len(),
        3,
        "dismissal also publishes the selected range"
    );
    click_dropdown(&mut dom, "aria-expanded");
    assert_eq!(events.borrow().len(), 3, "opening does not publish");
    assert!(dioxus_ssr::render(&dom).contains("2026/10/10 - 2026/10/16"));
    click_dropdown(&mut dom, "aria-expanded");
    assert_eq!(events.borrow().len(), 4);
    assert!(
        events
            .borrow()
            .iter()
            .all(|event| event == &events.borrow()[0])
    );
    click_dropdown(&mut dom, "aria-expanded");
    click_dropdown(&mut dom, "Done");
    assert_eq!(
        events.borrow().len(),
        6,
        "Done publishes explicit selection and shared dismissal"
    );
    assert!(
        events
            .borrow()
            .iter()
            .all(|event| event == &events.borrow()[0])
    );
}
fn dropdown_scheduler_fixture() -> Element {
    let tick = use_signal(|| 0_u32);
    let capture = consume_context::<Rc<RefCell<Option<Signal<u32>>>>>();
    *capture.borrow_mut() = Some(tick);
    rsx! {
        DateDropdown { today: date(2026,10,16), onrangechange: |_| {} }
        output { "External update {tick}" }
    }
}
#[test]
fn date_dropdown_focus_effect_settles_and_does_not_starve_external_updates() {
    // A self-subscribing effect can synchronously loop inside wait_for_work.
    // A worker deadline bounds this regression rather than hanging the suite.
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::{
            future::Future,
            pin::pin,
            task::{Context, Poll, Waker},
        };
        let capture = Rc::new(RefCell::new(None::<Signal<u32>>));
        let mut dom =
            VirtualDom::new(dropdown_scheduler_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        settle_dropdown(&mut dom);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            pin!(dom.wait_for_work()).poll(&mut context),
            Poll::Pending
        ));
        dom.in_scope(ScopeId::ROOT, || capture.borrow().unwrap().set(1));
        assert!(matches!(
            pin!(dom.wait_for_work()).poll(&mut context),
            Poll::Ready(())
        ));
        dom.render_immediate_to_vec();
        settle_dropdown(&mut dom);
        assert!(dioxus_ssr::render(&dom).contains("External update 1"));
        assert!(matches!(
            pin!(dom.wait_for_work()).poll(&mut context),
            Poll::Pending
        ));
        send.send(()).unwrap();
    });
    receive
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("DateDropdown effects must settle without starving the renderer");
}
#[test]
fn range_handoff_only_advances_first_pointer_commit_to_enabled_second_field() {
    let entries = [(3, true), (8, true), (11, true)];
    assert_eq!(range_handoff_target(3, false, entries.into_iter()), Some(8));
    assert_eq!(range_handoff_target(3, true, entries.into_iter()), None);
    assert_eq!(range_handoff_target(8, false, entries.into_iter()), None);
    assert_eq!(
        range_handoff_target(3, false, [(3, true), (8, false), (11, true)].into_iter()),
        None
    );
    assert_eq!(
        range_handoff_target(3, false, [(3, true)].into_iter()),
        None
    );
}
#[test]
fn localized_event_formatting_preserves_iso_date_math() {
    let day = date(2026, 2, 8);
    assert_eq!(
        CalendarLocale::De
            .format_date(day, "EEEE, dd MMMM yyyy")
            .as_deref(),
        Some("Sonntag, 08 Februar 2026")
    );
    assert_eq!(
        CalendarLocale::Fr
            .format_date(day, "dd MMM yyyy")
            .as_deref(),
        Some("08 févr. 2026")
    );
    assert_eq!(
        CalendarLocale::Ja
            .format_date(day, "dd MMMM yyyy")
            .as_deref(),
        Some("08 2月 2026")
    );
    assert_eq!(
        CalendarLocale::En.format_date(day, "LL/dd/yyyy").as_deref(),
        Some("02/08/2026")
    );
    assert!(
        CalendarLocale::En
            .format_date(day, "unsupported tokens")
            .is_none()
    );
    assert_eq!(day.to_string(), "2026-02-08");
    assert_eq!(
        CalendarLocale::Ja.format_time(IxTime::new(13, 5, 0, 0).unwrap(), TimeFormat::Hm12),
        "01:05 午後"
    );
}
#[test]
fn twelve_hour_intervals_follow_upstream_one_based_clock_options() {
    assert_eq!(
        clock_unit_values(ClockUnit::Hour, 2, TimeFormat::Hms12, IxTime::default()),
        [1, 3, 5, 7, 9, 11]
    );
    assert_eq!(
        clock_unit_values(
            ClockUnit::Hour,
            2,
            TimeFormat::Hms12,
            IxTime::new(13, 0, 0, 0).unwrap()
        ),
        [13, 15, 17, 19, 21, 23]
    );
    assert_eq!(
        clock_unit_values(ClockUnit::Hour, 1, TimeFormat::Hms12, IxTime::default()).last(),
        Some(&0)
    );
    let time = IxTime::new(1, 1, 1, 100).unwrap();
    assert_eq!(TimeFormat::HmsMillis12.format(time), "01:01:01.100");
    assert_eq!(TimeFormat::HmsMillis12.parse("01:01:01.100"), Some(time));
}
fn date(year: u16, month: u8, day: u8) -> IxDate {
    IxDate::new(year, month, day).unwrap()
}
fn time(hour: u8, minute: u8, second: u8) -> IxTime {
    IxTime::new(hour, minute, second, 0).unwrap()
}
#[test]
fn gregorian_leap_years_and_month_ends() {
    assert!(IxDate::new(2000, 2, 29).is_some());
    assert!(IxDate::new(1900, 2, 29).is_none());
    assert!(IxDate::new(2024, 2, 29).is_some());
    assert!(IxDate::new(2025, 2, 29).is_none());
    assert_eq!(date(2024, 1, 31).add_months(1), Some(date(2024, 2, 29)));
    assert_eq!(date(2024, 3, 31).add_months(-1), Some(date(2024, 2, 29)));
    assert_eq!(date(2023, 12, 31).add_days(1), Some(date(2024, 1, 1)));
    assert!(date(1, 1, 1).add_days(-1).is_none());
    assert!(date(9999, 12, 31).add_days(1).is_none());
}
#[test]
fn epoch_arithmetic_roundtrips_every_supported_day() {
    for days in -719162..=2932896 {
        let date = IxDate::from_epoch_days(days).unwrap();
        assert_eq!(date.days_since_epoch(), days);
    }
}
#[test]
fn monday_first_calendar_and_iso_week_year_boundary() {
    assert_eq!(date(1970, 1, 1).weekday(), 3);
    assert_eq!(date(2021, 1, 1).iso_week(), (2020, 53));
    assert_eq!(date(2024, 12, 30).iso_week(), (2025, 1));
    let rows = calendar_rows(date(2026, 2, 1), 0);
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0][6], Some(date(2026, 2, 1)));
    let sunday_first = calendar_rows(date(2026, 2, 1), 6);
    assert_eq!(sunday_first.len(), 4);
    assert_eq!(sunday_first[0][0], Some(date(2026, 2, 1)));
}
#[test]
fn header_navigation_keeps_a_valid_date_tab_stop() {
    // Moving from a 31-day month to February retains the nearest valid day.
    assert_eq!(
        focused_date_in_month(date(2024, 2, 1), date(2024, 1, 31), None, None),
        Some(date(2024, 2, 29))
    );
    // A leap-year selection also survives changing the year to a non-leap year.
    assert_eq!(
        focused_date_in_month(date(2025, 2, 1), date(2024, 2, 29), None, None),
        Some(date(2025, 2, 28))
    );
    // Partially selectable months put the tab stop inside the available dates.
    assert_eq!(
        focused_date_in_month(
            date(2026, 10, 1),
            date(2026, 9, 3),
            Some(date(2026, 10, 12)),
            None
        ),
        Some(date(2026, 10, 12))
    );
    assert_eq!(
        focused_date_in_month(
            date(2026, 10, 1),
            date(2026, 9, 30),
            None,
            Some(date(2026, 10, 20))
        ),
        Some(date(2026, 10, 20))
    );
    assert_eq!(
        focused_date_in_month(
            date(2026, 9, 1),
            date(2026, 10, 16),
            Some(date(2026, 10, 12)),
            None
        ),
        None
    );
}
#[test]
fn range_selection_sorts_then_restarts() {
    let first = date(2026, 10, 16);
    let earlier = date(2026, 10, 3);
    let pending = DateSelection::default().select(first, false);
    assert_eq!(
        pending,
        DateSelection {
            from: Some(first),
            to: None
        }
    );
    let completed = pending.select(earlier, false);
    assert_eq!(
        completed,
        DateSelection {
            from: Some(earlier),
            to: Some(first)
        }
    );
    assert_eq!(completed.select(first, false), pending);
    let single = date(2026, 10, 24);
    for selection in [DateSelection::default(), pending, completed] {
        assert_eq!(
            selection.select(single, true),
            DateSelection {
                from: Some(single),
                to: None
            }
        );
    }
}
#[test]
fn date_formats_reject_impossible_dates() {
    for format in [
        DateFormat::YmdSlash,
        DateFormat::Iso,
        DateFormat::DmyDot,
        DateFormat::DmySlash,
        DateFormat::MdySlash,
    ] {
        assert_eq!(
            format.parse(&format.format(date(2024, 2, 29))),
            Some(date(2024, 2, 29))
        );
    }
    assert!(DateFormat::Iso.parse("2025-02-29").is_none());
    assert!(DateFormat::Iso.parse("2026-13-01").is_none());
    assert!(DateFormat::Iso.parse("-001-01-01").is_none());
}
#[test]
fn named_date_formats_and_spaced_datetime_inputs_roundtrip() {
    let expected = IxDateTime {
        date: date(2026, 10, 20),
        time: time(13, 30, 0),
    };
    for locale in [
        CalendarLocale::En,
        CalendarLocale::De,
        CalendarLocale::Fr,
        CalendarLocale::Ja,
    ] {
        for format in [
            DateFormat::LongMonth(locale),
            DateFormat::ShortMonth(locale),
            DateFormat::WeekdayLongMonth(locale),
        ] {
            assert_eq!(
                format.parse(&format.format(expected.date)),
                Some(expected.date)
            );
            for clock in [TimeFormat::Hm12, TimeFormat::Hms] {
                let text = expected.format(format, clock);
                assert_eq!(
                    IxDateTime::parse(&text, format, clock),
                    Some(expected),
                    "{text}"
                );
            }
        }
    }
    assert_eq!(
        IxDateTime::parse(
            "2026/10/20 01:30 PM",
            DateFormat::YmdSlash,
            TimeFormat::Hm12
        ),
        Some(expected)
    );
    for text in [
        "Monday, 20 October 2026 01:30 PM",
        "Tuesday, 31 October 2026 01:30 PM",
        "Tuesday, 20 October 2026 13:30 PM",
    ] {
        assert!(
            IxDateTime::parse(
                text,
                DateFormat::WeekdayLongMonth(CalendarLocale::En),
                TimeFormat::Hm12
            )
            .is_none(),
            "{text}"
        );
    }
}
#[test]
fn noon_midnight_milliseconds_and_range_validation() {
    assert_eq!(TimeFormat::Hm12.parse("12:00 AM"), Some(time(0, 0, 0)));
    assert_eq!(TimeFormat::Hm12.parse("12:00 PM"), Some(time(12, 0, 0)));
    assert!(TimeFormat::Hm12.parse("00:00 AM").is_none());
    assert!(TimeFormat::Hms.parse("24:00:00").is_none());
    assert!(TimeFormat::Hms.parse("23:60:00").is_none());
    let millis = IxTime::new(23, 59, 59, 999).unwrap();
    assert_eq!(
        TimeFormat::HmsMillis.parse(&TimeFormat::HmsMillis.format(millis)),
        Some(millis)
    );
    assert!(
        !validate_date_input(
            "2026/02/01",
            DateFormat::YmdSlash,
            false,
            Some(date(2026, 2, 2)),
            None
        )
        .valid()
    );
    assert!(
        validate_time_input(
            "12:00:00",
            TimeFormat::Hms,
            false,
            None,
            Some(time(11, 0, 0))
        )
        .range_overflow
    );
}
#[test]
fn constrained_clock_selection_reaches_partial_hours() {
    let min = Some(time(10, 30, 0));
    let max = Some(time(11, 15, 0));
    assert_eq!(
        select_clock_unit(time(0, 0, 0), ClockUnit::Hour, 10, min, max),
        Some(time(10, 30, 0))
    );
    assert_eq!(
        select_clock_unit(time(10, 45, 0), ClockUnit::Hour, 11, min, max),
        Some(time(11, 15, 0))
    );
    assert!(select_clock_unit(time(10, 45, 0), ClockUnit::Hour, 12, min, max).is_none());
    assert!(select_clock_unit(time(10, 45, 0), ClockUnit::Minute, 29, min, max).is_none());
}
#[test]
fn datetime_time_bounds_follow_boundary_dates_and_one_sided_windows() {
    // Upstream datetime-picker.ct.ts's min/max combination matrix.
    let first = date(2026, 2, 1);
    let middle = date(2026, 2, 10);
    let last = date(2026, 2, 28);
    let min = Some(time(13, 0, 0));
    let max = Some(time(17, 30, 0));
    let bounds = (Some(first), Some(last));
    assert_eq!(
        date_scoped_time_bounds(Some(first), bounds, (min, max)),
        (min, None)
    );
    assert_eq!(
        date_scoped_time_bounds(Some(middle), bounds, (min, max)),
        (None, None)
    );
    assert_eq!(
        date_scoped_time_bounds(Some(last), bounds, (min, max)),
        (None, max)
    );
    assert_eq!(
        date_scoped_time_bounds(None, bounds, (min, max)),
        (None, None)
    );
    assert_eq!(
        date_scoped_time_bounds(Some(middle), (None, None), (min, max)),
        (min, max)
    );
    assert_eq!(
        date_scoped_time_bounds(Some(first), (Some(first), None), (min, max)),
        (min, None)
    );
    assert_eq!(
        date_scoped_time_bounds(Some(middle), (Some(first), None), (min, max)),
        (None, None)
    );
    assert_eq!(
        date_scoped_time_bounds(Some(last), (None, Some(last)), (min, max)),
        (None, max)
    );
    assert_eq!(
        date_scoped_time_bounds(Some(middle), (None, Some(last)), (min, max)),
        (None, None)
    );
    assert_eq!(
        date_scoped_time_bounds(Some(first), (Some(first), Some(first)), (min, max)),
        (min, max)
    );
}
#[test]
fn typed_datetime_checks_first_last_and_middle_days_consistently() {
    // Upstream datetime-input.ct.ts's Apr 27–29 / 13:10–17:30 cases.
    let check = |value: &str| {
        datetime_input_validity(
            value,
            IxDateTime::parse(value, DateFormat::YmdSlash, TimeFormat::Hms),
            true,
            (Some(date(2026, 4, 27)), Some(date(2026, 4, 29))),
            (Some(time(13, 10, 0)), Some(time(17, 30, 0))),
        )
    };
    assert!(check("2026/04/27 13:09:59").range_underflow);
    assert!(check("2026/04/27 13:10:00").valid());
    assert!(check("2026/04/27 23:59:59").valid());
    assert!(check("2026/04/28 00:00:00").valid());
    assert!(check("2026/04/28 23:59:59").valid());
    assert!(check("2026/04/29 00:00:00").valid());
    assert!(check("2026/04/29 17:30:00").valid());
    assert!(check("2026/04/29 17:30:01").range_overflow);
    assert!(check("2026/04/26 14:00:00").range_underflow);
    assert!(check("2026/04/30 14:00:00").range_overflow);
    assert!(check("2026/04/31 14:00:00").bad_input);
    assert!(check("").value_missing);
}
#[test]
fn datetime_input_renders_invalid_patterns_only_for_violated_boundary_times() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            for value in ["2026/04/27 12:00:00", "2026/04/28 12:00:00", "2026/04/28 20:00:00", "2026/04/29 20:00:00"] {
                DateTimeInput {
                    today: date(2026,4,28), value,
                    min_date: Some(date(2026,4,27)), max_date: Some(date(2026,4,29)),
                    min_time: Some(time(13,10,0)), max_time: Some(time(17,30,0)),
                    onvaluechange: |_| {},
                }
            }
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("pattern=\"(?!)\"").count(), 2);
    // Dioxus SSR serializes boolean-valued ARIA attributes without quotes.
    assert_eq!(
        html.matches("aria-invalid=true").count(),
        2,
        "rendered inputs: {:?}",
        html.split("<input")
            .skip(1)
            .map(|input| input.split('>').next().unwrap())
            .collect::<Vec<_>>()
    );
}
#[test]
fn reversed_time_bounds_are_ignored_instead_of_creating_an_empty_clock() {
    let min = Some(time(17, 30, 0));
    let max = Some(time(13, 10, 0));
    assert_eq!(normalized_time_bounds(min, max), (None, None));
    assert_eq!(
        select_clock_unit(time(12, 0, 0), ClockUnit::Hour, 23, min, max),
        Some(time(23, 0, 0)),
    );
    assert!(validate_time_input("12:00:00", TimeFormat::Hms, false, min, max).valid());
    let value = "2026/04/27 12:00:00";
    assert!(
        datetime_input_validity(
            value,
            IxDateTime::parse(value, DateFormat::YmdSlash, TimeFormat::Hms),
            false,
            (Some(date(2026, 4, 27)), Some(date(2026, 4, 29))),
            (min, max),
        )
        .valid()
    );
    let equal = Some(time(13, 10, 0));
    assert_eq!(normalized_time_bounds(equal, equal), (equal, equal));
}
#[test]
fn datetime_picker_renders_effective_clock_bounds_including_range_mode() {
    fn fixture(from: IxDate, single: bool) -> String {
        let mut dom = VirtualDom::new_with_props(
            |(from, single): (IxDate, bool)| {
                rsx! {
                    DateTimePicker {
                        today: date(2026,2,10), from: Some(from), to: Some(date(2026,2,28)),
                        min_date: Some(date(2026,2,1)), max_date: Some(date(2026,2,28)),
                        min_time: Some(time(13,0,0)), max_time: Some(time(17,30,0)),
                        time: time(12,0,0), single_selection: single,
                        ondatechange: |_| {}, ontimechange: |_| {}, onselect: |_| {},
                    }
                }
            },
            (from, single),
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
    fn hour_disabled(html: &str, hour: usize) -> bool {
        html.split("<button")
            .filter(|part| part.split('>').next().unwrap().contains("role=\"option\""))
            .nth(hour)
            .expect("clock hour option")
            .split('>')
            .next()
            .unwrap()
            .contains("disabled")
    }
    let first = fixture(date(2026, 2, 1), true);
    assert!(hour_disabled(&first, 12));
    assert!(!hour_disabled(&first, 13));
    assert!(!hour_disabled(&first, 18));
    let middle = fixture(date(2026, 2, 10), true);
    assert!(!hour_disabled(&middle, 12));
    assert!(!hour_disabled(&middle, 18));
    let last = fixture(date(2026, 2, 28), true);
    assert!(!hour_disabled(&last, 12));
    assert!(!hour_disabled(&last, 17));
    assert!(hour_disabled(&last, 18));
    let range = fixture(date(2026, 2, 1), false);
    assert!(!hour_disabled(&range, 12));
    assert!(!hour_disabled(&range, 18));
}
#[test]
fn clock_keyboard_navigation_is_provisional_until_activation() {
    let selectable = [8, 10, 12];
    assert_eq!(
        clock_key_action(&selectable, 8, &Key::ArrowDown),
        ClockKeyAction::Focus(10)
    );
    assert_eq!(
        clock_key_action(&selectable, 8, &Key::ArrowUp),
        ClockKeyAction::Focus(12)
    );
    assert_eq!(
        clock_key_action(&selectable, 10, &Key::Home),
        ClockKeyAction::Focus(8)
    );
    assert_eq!(
        clock_key_action(&selectable, 10, &Key::End),
        ClockKeyAction::Focus(12)
    );
    assert_eq!(
        clock_key_action(&selectable, 10, &Key::Enter),
        ClockKeyAction::Commit
    );
    assert_eq!(
        clock_key_action(&selectable, 10, &Key::Character(" ".into())),
        ClockKeyAction::Commit
    );
    assert_eq!(
        clock_key_action(&selectable, 10, &Key::Tab),
        ClockKeyAction::Unhandled
    );
    assert_eq!(
        clock_key_action(&[], 10, &Key::ArrowDown),
        ClockKeyAction::Unhandled
    );
}
#[test]
fn shifted_calendar_page_navigation_moves_years_and_clamps_leap_days() {
    let leap_day = date(2024, 2, 29);
    assert_eq!(
        calendar_page_target(leap_day, true, false),
        Some(date(2024, 3, 29))
    );
    assert_eq!(
        calendar_page_target(leap_day, true, true),
        Some(date(2025, 2, 28))
    );
    assert_eq!(
        calendar_page_target(leap_day, false, true),
        Some(date(2023, 2, 28))
    );
    assert_eq!(
        calendar_page_target(date(2026, 12, 31), true, true),
        Some(date(2027, 12, 31))
    );
    assert_eq!(calendar_page_target(date(1, 1, 1), false, true), None);
}
#[test]
fn clock_option_visibility_scrolls_only_its_list_coordinates() {
    assert_eq!(
        clock_list_scroll_offset(100.0, 200.0, 260.0, 32.0, 0.0),
        0.0
    );
    assert_eq!(
        clock_list_scroll_offset(100.0, 200.0, 516.0, 32.0, 0.0),
        248.0
    );
    assert_eq!(
        clock_list_scroll_offset(100.0, 200.0, 68.0, 32.0, 200.0),
        168.0
    );
    // A picker below the viewport needs exactly the same internal scrolling.
    assert_eq!(
        clock_list_scroll_offset(2706.0, 200.0, 3122.0, 32.0, 0.0),
        248.0
    );
    // Once scrolled, an already visible selection preserves the current offset.
    assert_eq!(
        clock_list_scroll_offset(100.0, 200.0, 268.0, 32.0, 248.0),
        248.0
    );
}
#[test]
fn formatted_inputs_use_native_names_and_invalid_constraint_patterns() {
    fn render(build: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(build);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
    let invalid = render(|| {
        rsx! {
            DateInput { today: IxDate::new(2026,10,16).unwrap(), name: "service-date", value: "2026/02/30", onvaluechange: |_| {} }
            TimeInput { name: "service-time", value: "07:00:00", min_time: IxTime::new(8,0,0,0), onvaluechange: |_| {} }
            DateTimeInput { today: IxDate::new(2026,10,16).unwrap(), name: "service-datetime", value: "2026/02/30 12:00:00", onvaluechange: |_| {} }
        }
    });
    assert_eq!(invalid.matches("pattern=\"(?!)\"").count(), 3);
    for name in ["service-date", "service-time", "service-datetime"] {
        assert_eq!(invalid.matches(&format!("name=\"{name}\"")).count(), 1);
    }
    assert!(!invalid.contains("type=\"hidden\""));
    let valid = render(|| {
        rsx! {
            DateInput { today: IxDate::new(2026,10,16).unwrap(), name: "service-date", value: "2026/10/16", onvaluechange: |_| {} }
        }
    });
    assert!(valid.contains("name=\"service-date\""));
    assert!(!valid.contains("pattern="));
}
mod scheduling_tests {
    use super::*;
    include!("../datetime_scheduling_tests.rs");
    include!("../temporal_form_tests.rs");
    include!("../temporal_mount_tests.rs");
    mod date_dropdown_entry_tests {
        use super::*;
        include!("date_dropdown_entry_tests.rs");
    }
    mod picker_roots_tests {
        use super::*;
        include!("picker_roots_tests.rs");
    }
}
