//! Isolated Rust counterparts of the upstream date and time preview sources.
use chrono::{Datelike, Timelike};
use dioxus::prelude::*;
use dioxus_ix::*;

pub const SCENARIOS: &[&str] = &[
    "aria-label-properties",
    "date-dropdown",
    "date-dropdown-presets",
    "date-dropdown-user-range",
    "date-input",
    "date-input-disabled",
    "date-input-label",
    "date-input-min-max-date",
    "date-input-readonly",
    "date-input-validation",
    "date-input-with-slots",
    "date-range",
    "datepicker",
    "datepicker-locale",
    "datepicker-range",
    "datetime-input",
    "datetime-input-disabled",
    "datetime-input-label",
    "datetime-input-min-max-date",
    "datetime-input-readonly",
    "datetime-input-validation",
    "datetime-input-with-slots",
    "datetime-range",
    "datetimepicker",
    "datetimepicker-locale",
    "range-field",
    "time-input",
    "time-input-disabled",
    "time-input-label",
    "time-input-readonly",
    "time-input-validation",
    "time-input-with-slots",
    "time-range",
    "timepicker",
    "timepicker-format-adjusted",
    "timepicker-intervals",
    "timepicker-min-max-time",
];

#[derive(Clone, Copy)]
struct LocalFixture {
    today: IxDate,
    time: IxTime,
}
fn local_fixture() -> LocalFixture {
    let now = chrono::Local::now();
    LocalFixture {
        today: IxDate::new(now.year() as u16, now.month() as u8, now.day() as u8).unwrap(),
        time: IxTime::new(
            now.hour() as u8,
            now.minute() as u8,
            now.second() as u8,
            now.timestamp_subsec_millis() as u16,
        )
        .unwrap(),
    }
}

/// A newly selected scenario receives fresh state and the user's current local clock.
#[component]
pub fn DateTimeScenario(name: String) -> Element {
    rsx! { style { {STYLES} } for selected in [name] { ScenarioBody { key: "{selected}", name: selected } } }
}

#[component]
fn ScenarioBody(name: String) -> Element {
    let fixture = use_hook(local_fixture);
    let body = match name.as_str() {
        name if name.starts_with("date-input")
            || name.starts_with("time-input")
            || name.starts_with("datetime-input") =>
        {
            rsx! { InputSource { name: name.to_string(), today: fixture.today } }
        }
        "date-range" | "time-range" | "datetime-range" | "range-field" => {
            rsx! { RangeSource { name: name.clone(), today: fixture.today } }
        }
        "date-dropdown" | "date-dropdown-presets" | "date-dropdown-user-range" => {
            rsx! { DropdownSource { name: name.clone(), today: fixture.today } }
        }
        "datepicker-locale" | "datetimepicker-locale" => {
            rsx! { LocaleSource { combined: name == "datetimepicker-locale", today: fixture.today, initial_time: fixture.time } }
        }
        "datepicker" | "datepicker-range" | "aria-label-properties" => {
            rsx! { CalendarSource { name: name.clone(), today: fixture.today } }
        }
        "datetimepicker" => {
            rsx! { CombinedSource { today: fixture.today, initial_time: fixture.time } }
        }
        "timepicker"
        | "timepicker-format-adjusted"
        | "timepicker-intervals"
        | "timepicker-min-max-time" => {
            rsx! { ClockSource { name: name.clone(), initial_time: fixture.time } }
        }
        _ => rsx! { p { "No date/time source scenario named {name}." } },
    };
    rsx! { div { class: "datetime-source-case", {body} } }
}

#[component]
fn InputSource(name: String, today: IxDate) -> Element {
    let date = name.starts_with("date-input");
    let datetime = name.starts_with("datetime-input");
    let validation = name.ends_with("-validation");
    let customized = datetime && name.ends_with("-label");
    let mut values = use_signal(|| {
        vec![
            if date {
                "2026/02/08".to_string()
            } else if datetime {
                if customized {
                    "08.02.2026 00:00".to_string()
                } else {
                    "2026/02/08 00:00:00".to_string()
                }
            } else {
                String::new()
            };
            4
        ]
    });
    let bounds = name.ends_with("-min-max-date");
    let disabled = name.ends_with("-disabled");
    let readonly = name.ends_with("-readonly");
    let slots = name.ends_with("-with-slots");
    let labels = ["Info", "Warning", "Valid", "Invalid"];
    let states = [
        FieldState::Info,
        FieldState::Warning,
        FieldState::Valid,
        FieldState::Invalid,
    ];
    rsx! { div { class: "datetime-source-inputs",
        for index in 0..if validation { 4 } else { 1 } {
            {
                let state = if validation { states[index] } else { FieldState::Default };
                let start = slots.then(|| rsx! { Icon { name: "bulb", size: 16, label: if datetime { Some("Info".to_string()) } else { None } } });
                let end = slots.then(|| rsx! { Typography { "Slot" } });
                rsx! {
                    if date {
                        DateInput { today, value: values()[index].clone(), disabled, readonly, state, start, end,
                            label: if name.ends_with("-label") { "Begin" } else { "" },
                            name: name.ends_with("-label").then(|| "begin".to_string()),
                            helper_text: name.ends_with("-label").then(|| "Some helper text".to_string()),
                            min_date: if bounds { IxDate::new(2026,2,1) } else { None }, max_date: if bounds { IxDate::new(2026,2,28) } else { None },
                            info_text: "Info text", warning_text: "Warning text", valid_text: "Valid text", invalid_text: "Invalid text",
                            onvaluechange: move |next| values.write()[index] = next,
                        }
                    } else if datetime {
                        DateTimeInput { today, value: values()[index].clone(), disabled, readonly, state, start, end,
                            label: if validation { labels[index] } else if customized { "Meeting schedule" } else { "Date and time" },
                            date_format: if customized { DateFormat::DmyDot } else { DateFormat::YmdSlash },
                            time_format: if customized { TimeFormat::Hm } else { TimeFormat::Hms },
                            min_date: if bounds { IxDate::new(2026,2,1) } else { None }, max_date: if bounds { IxDate::new(2026,2,28) } else { None },
                            info_text: "Info text", warning_text: "Warning text", valid_text: "Valid text", invalid_text: "Invalid text",
                            onvaluechange: move |next| values.write()[index] = next,
                        }
                    } else {
                        TimeInput { value: values()[index].clone(), disabled, readonly, state, start, end,
                            label: if name.ends_with("-label") { "Time" } else { "" },
                            name: name.ends_with("-label").then(|| "time".to_string()),
                            helper_text: name.ends_with("-label").then(|| "Some helper text".to_string()),
                            info_text: "Info text", warning_text: "Warning text", valid_text: "Valid text", invalid_text: "Invalid text",
                            onvaluechange: move |next| values.write()[index] = next,
                        }
                    }
                }
            }
        }
    } }
}

#[component]
fn RangeSource(name: String, today: IxDate) -> Element {
    let mut values = use_signal(|| vec![String::new(); 6]);
    rsx! { div { class: "datetime-source-ranges",
        if name == "range-field" || name == "date-range" {
            div { role: "group", "aria-label": if name == "date-range" { "date range" } else { "Date range" }, RangeField {
                DateInput { today, label: "Start date", value: values()[0].clone(), onvaluechange: move |next| values.write()[0] = next }
                DateInput { today, label: "End date", value: values()[1].clone(), onvaluechange: move |next| values.write()[1] = next }
            } }
        }
        if name == "range-field" || name == "time-range" {
            div { role: "group", "aria-label": "Time range", RangeField {
                TimeInput { label: "Start time", value: values()[2].clone(), onvaluechange: move |next| values.write()[2] = next }
                TimeInput { label: "End time", value: values()[3].clone(), onvaluechange: move |next| values.write()[3] = next }
            } }
        }
        if name == "range-field" || name == "datetime-range" {
            div { role: "group", "aria-label": "Datetime range", RangeField {
                DateTimeInput { today, label: "Start date and time", value: values()[4].clone(), onvaluechange: move |next| values.write()[4] = next }
                DateTimeInput { today, label: "End date and time", value: values()[5].clone(), onvaluechange: move |next| values.write()[5] = next }
            } }
        }
    } }
}

#[component]
fn DropdownSource(name: String, today: IxDate) -> Element {
    let mut selection = use_signal(DateSelection::default);
    let mut id = use_signal(|| {
        if name == "date-dropdown" {
            "custom".to_string()
        } else {
            "last-7".to_string()
        }
    });
    let options = if name == "date-dropdown-presets" {
        [
            ("today", "Today", 0),
            ("last-7", "Last 7 days", 6),
            ("last-30", "Last 30 days", 29),
            ("last-90", "Last 90 days", 89),
            ("last-365", "Last 365 days", 364),
        ]
        .into_iter()
        .map(|(id, label, days)| DateDropdownOption {
            id: id.into(),
            label: label.into(),
            from: today.add_days(-days).unwrap_or(today),
            to: today,
        })
        .collect()
    } else if name == "date-dropdown-user-range" {
        vec![
            DateDropdownOption {
                id: "last-7".into(),
                label: "Last 7 days".into(),
                from: today.add_days(-7).unwrap_or(today),
                to: today,
            },
            DateDropdownOption {
                id: "today".into(),
                label: "Today".into(),
                from: today,
                to: today,
            },
        ]
    } else {
        vec![]
    };
    rsx! { DateDropdown { today, options, date_range_id: id(), from: selection().from, to: selection().to,
        format: if name == "date-dropdown-user-range" { DateFormat::MdySlash } else { DateFormat::YmdSlash },
        onrangechange: move |next: DateRangeChange| { id.set(next.id); selection.set(next.dates); }
    } }
}

#[component]
fn CalendarSource(name: String, today: IxDate) -> Element {
    let mut selection = use_signal(|| {
        if name == "datepicker-range" {
            DateSelection {
                from: IxDate::new(2022, 12, 15),
                to: IxDate::new(2022, 12, 24),
            }
        } else {
            DateSelection::default()
        }
    });
    let labels = CalendarLabels {
        previous_month: "Previous month".into(),
        next_month: "Next month".into(),
        ..Default::default()
    };
    rsx! { DatePicker { today, from: selection().from, to: selection().to, single_selection: name == "datepicker", labels,
        ondatechange: move |next| selection.set(next)
    } }
}

#[component]
fn CombinedSource(today: IxDate, initial_time: IxTime) -> Element {
    let mut selection = use_signal(DateSelection::default);
    let mut time = use_signal(|| initial_time);
    let mut result = use_signal(String::new);
    rsx! { DateTimePicker { today, from: selection().from, to: selection().to, time: time(),
        ondatechange: move |next| selection.set(next), ontimechange: move |next| time.set(next),
        onselect: move |next: DateTimeSelection| result.set(format!("{} – {} {}", next.dates.from.map(|date|date.to_string()).unwrap_or_default(), next.dates.to.map(|date|date.to_string()).unwrap_or_default(), TimeFormat::Hms.format(next.time)))
    } if !result().is_empty() { output { class: "datetime-source-result", role: "status", "{result}" } } }
}

#[component]
fn ClockSource(name: String, initial_time: IxTime) -> Element {
    let initial = match name.as_str() {
        "timepicker-format-adjusted" => IxTime::new(1, 1, 1, 100).unwrap(),
        "timepicker-intervals" => IxTime::new(1, 15, 10, 0).unwrap(),
        "timepicker-min-max-time" => IxTime::new(12, 0, 0, 0).unwrap(),
        _ => initial_time,
    };
    let mut time = use_signal(|| initial);
    let mut result = use_signal(String::new);
    let format = match name.as_str() {
        "timepicker-format-adjusted" => TimeFormat::HmsMillis12,
        "timepicker-intervals" => TimeFormat::Hms12,
        _ => TimeFormat::Hms,
    };
    let intervals = name == "timepicker-intervals";
    let bounds = name == "timepicker-min-max-time";
    rsx! { TimePicker { time: time(), format, hour_interval: if intervals { 2 } else { 1 }, minute_interval: if intervals { 15 } else { 1 }, second_interval: if intervals { 10 } else { 1 },
        min_time: if bounds { IxTime::new(9,0,0,0) } else { None }, max_time: if bounds { IxTime::new(17,30,0,0) } else { None },
        ontimechange: move |next| time.set(next), onselect: move |next| result.set(format.format(next)),
    } if !result().is_empty() { output { class: "datetime-source-result", role: "status", "{result}" } } }
}

fn calendar_locale(code: &str) -> CalendarLocale {
    match code {
        "de" => CalendarLocale::De,
        "fr" => CalendarLocale::Fr,
        "ja" => CalendarLocale::Ja,
        _ => CalendarLocale::En,
    }
}
fn date_formats() -> Vec<SelectOption> {
    [
        ("yyyy/LL/dd", "yyyy/LL/dd (default)"),
        (
            "EEEE, dd MMMM yyyy",
            "EEEE, dd MMMM yyyy (all locale names)",
        ),
        ("dd MMMM yyyy", "dd MMMM yyyy (month name)"),
        ("dd MMM yyyy", "dd MMM yyyy (short month)"),
        ("LL/dd/yyyy", "LL/dd/yyyy"),
    ]
    .into_iter()
    .map(|(value, label)| SelectOption::new(value, label))
    .collect()
}

#[component]
fn LocaleSource(combined: bool, today: IxDate, initial_time: IxTime) -> Element {
    let mut locale = use_signal(|| "de".to_string());
    let mut format = use_signal(|| "yyyy/LL/dd".to_string());
    let mut time_format = use_signal(|| "HH:mm:ss".to_string());
    let mut selection = use_signal(DateSelection::default);
    let mut time = use_signal(|| initial_time);
    // Event logs preserve emitted values when locale/format controls subsequently change.
    let mut event_values = use_signal(|| None::<Vec<(String, String, String)>>);
    let language = calendar_locale(&locale());
    let clock_format = if time_format() == "hh:mm a" {
        TimeFormat::Hm12
    } else {
        TimeFormat::Hms
    };
    let emit = use_callback(move |event: DateTimeSelection| {
        let language = calendar_locale(&locale());
        let date_format = format();
        let clock_format = if time_format() == "hh:mm a" {
            TimeFormat::Hm12
        } else {
            TimeFormat::Hms
        };
        let formatted = |date: Option<IxDate>| {
            date.and_then(|date| language.format_date(date, &date_format))
                .unwrap_or_else(|| "—".into())
        };
        let iso = |date: Option<IxDate>| {
            date.map(|date| date.to_string())
                .unwrap_or_else(|| "—".into())
        };
        let mut rows = vec![
            (
                "from".into(),
                formatted(event.dates.from),
                "(locale-aware)".into(),
            ),
            (
                "to".into(),
                formatted(event.dates.to),
                "(locale-aware)".into(),
            ),
        ];
        if combined {
            rows.push((
                "time".into(),
                language.format_time(event.time, clock_format),
                "(locale-aware in 12h mode)".into(),
            ));
        }
        rows.push((
            "isoFrom".into(),
            iso(event.dates.from),
            "(ISO 8601, stable)".into(),
        ));
        rows.push((
            "isoTo".into(),
            iso(event.dates.to),
            "(ISO 8601, stable)".into(),
        ));
        if combined {
            rows.push((
                "isoTime".into(),
                TimeFormat::HmsMillis.format(event.time),
                "(ISO 8601, stable)".into(),
            ));
        }
        event_values.set(Some(rows));
    });
    let language_options = ["de", "en", "fr", "ja"]
        .map(|code| SelectOption::new(code, code))
        .to_vec();
    rsx! { div { class: "datetime-source-locale",
        div { class: if combined { "datetime-source-locale-controls datetime-source-locale-combined" } else { "datetime-source-locale-controls" },
            div { class: "datetime-source-language", span { "Language:" } Select { id: "locale-select", value: locale(), options: language_options, onvaluechange: move |next| locale.set(next) } }
            div { class: "datetime-source-format", span { if combined { "Date format:" } else { "Format:" } } Select { id: if combined { "date-format-select" } else { "format-select" }, value: format(), options: date_formats(), onvaluechange: move |next| format.set(next) } }
            if combined { div { class: "datetime-source-time-format", span { "Time format:" } Select { id: "time-format-select", value: time_format(), options: vec![SelectOption::new("HH:mm:ss", "HH:mm:ss (24h, default)"), SelectOption::new("hh:mm a", "hh:mm a (12h, locale-aware)")], onvaluechange: move |next| time_format.set(next) } } }
        }
        if combined {
            DateTimePicker { today, from: selection().from, to: selection().to, time: time(), time_format: clock_format, calendar_labels: language.labels(), time_labels: language.time_labels(),
                ondatechange: move |next| selection.set(next), ontimechange: move |next| time.set(next), onselect: move |next| emit.call(next) }
        } else {
            DatePicker { today, from: selection().from, to: selection().to, labels: language.labels(),
                ondatechange: move |next| { selection.set(next); emit.call(DateTimeSelection { dates: next, time: time() }); } }
        }
        div { class: if combined { "datetime-source-event datetime-source-event-wide" } else { "datetime-source-event" },
            strong { if combined { "Last dateSelect event" } else { "Last date-picker event" } }
            if let Some(rows) = event_values() {
                div { class: "datetime-source-event-values", for (key,value,note) in rows { div { class: "datetime-source-event-row", span { "{key}" } code { "{value}" } span { "{note}" } } } }
            } else { span { class: "datetime-source-event-placeholder", if combined { "Confirm a selection to see event values" } else { "Select a date to see event values" } } }
        }
    } }
}

pub const STYLES: &str = include_str!("styles/datetime_scenarios.css");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_registered_source_renders_and_only_its_own_scenario_is_selected() {
        for name in SCENARIOS {
            let mut dom = VirtualDom::new_with_props(
                DateTimeScenario,
                DateTimeScenarioProps {
                    name: (*name).to_string(),
                },
            );
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            assert!(
                !html.contains("No date/time source scenario"),
                "Unimplemented registered source: {name}"
            );
            assert_eq!(
                html.matches("class=\"datetime-source-case\"").count(),
                1,
                "{name}"
            );
            assert!(
                html.contains("ix-temporal-field")
                    || html.contains("ix-date-time-card")
                    || html.contains("ix-date-dropdown"),
                "No control rendered for {name}"
            );
        }
    }
}
