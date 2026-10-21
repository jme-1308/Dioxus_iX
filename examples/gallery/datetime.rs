use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn DateTimeExamples() -> Element {
    // A fixed review fixture; application hosts supply the user's local date.
    let today = IxDate::new(2026, 10, 16).unwrap();
    let mut single = use_signal(|| Some(today));
    let mut range = use_signal(|| DateSelection {
        from: today.add_days(-6),
        to: Some(today),
    });
    let mut clock = use_signal(|| IxTime::new(13, 7, 4, 0).unwrap());
    let mut date_text = use_signal(|| "2026/10/16".to_string());
    let mut time_text = use_signal(|| "13:07:04".to_string());
    let mut datetime_text = use_signal(|| "2026/10/16 13:07:04".to_string());
    let mut invalid_text = use_signal(|| "2026/02/30".to_string());
    let mut preset_id = use_signal(|| "last-7".to_string());
    let mut last_selection =
        use_signal(|| "Choose a date or time to review the event payload.".to_string());
    let mut form_valid = use_signal(|| true);
    let options = vec![
        DateDropdownOption {
            id: "today".into(),
            label: "Today".into(),
            from: today,
            to: today,
        },
        DateDropdownOption {
            id: "last-7".into(),
            label: "Last 7 days".into(),
            from: today.add_days(-6).unwrap(),
            to: today,
        },
        DateDropdownOption {
            id: "last-30".into(),
            label: "Last 30 days".into(),
            from: today.add_days(-29).unwrap(),
            to: today,
        },
    ];
    rsx! {
        section { class: "demo-section", id: "date-picker",
            h2 { "Date picker" }
            p { "Single date, date range, bounds, and week numbers. The review clock is fixed to 16 October 2026." }
            div { class: "demo-date-row",
                div { h3 { "Single selection" }
                    DatePicker { today, from: single(), single_selection: true,
                        ondatechange: move |value: DateSelection| single.set(value.from),
                        onselect: move |value: DateSelection| last_selection.set(format!("Selected date: {}", value.from.unwrap())),
                    }
                }
                div { h3 { "Range selection with week numbers" }
                    DatePicker { today, from: range().from, to: range().to, show_week_numbers: true,
                        min_date: today.add_days(-45), max_date: today.add_days(45),
                        ondatechange: move |value| range.set(value),
                        onselect: move |value: DateSelection| last_selection.set(format!("Range: {:?} – {:?}", value.from, value.to)),
                    }
                }
                div { h3 { "Sunday first" }
                    DatePicker { today, from: single(), single_selection: true, week_start_index: 6,
                        ondatechange: move |value: DateSelection| single.set(value.from),
                    }
                }
            }
        }
        section { class: "demo-section", id: "date-input",
            h2 { "Date input" }
            div { class: "demo-date-input-grid",
                DateInput { today, label: "Date", value: date_text(), onvaluechange: move |value| date_text.set(value), helper_text: "Format: YYYY/MM/DD" }
                DateInput { today, label: "Required date", value: invalid_text(), required: true, onvaluechange: move |value| invalid_text.set(value),
                    onvaliditychange: move |validity: DateInputValidity| form_valid.set(validity.valid()),
                    helper_text: "Type an impossible date to review validation." }
                DateInput { today, label: "Minimum and maximum date", value: date_text(), min_date: today.add_days(-5), max_date: today.add_days(5),
                    onvaluechange: move |value| date_text.set(value), helper_text: "Choose 11–21 October 2026." }
                DateInput { today, label: "Read-only", value: "2026/10/16", readonly: true, onvaluechange: |_| {} }
                DateInput { today, label: "Disabled", value: "2026/10/16", disabled: true, onvaluechange: |_| {} }
            }
            p { role: "status", "Date value: {date_text}. Last validation event valid: {form_valid}." }
        }
        BoundaryDateTimeExample { today }
        section { class: "demo-section", id: "time-picker",
            h2 { "Time picker" }
            div { class: "demo-date-row",
                div { h3 { "Hours, minutes, seconds" }
                    TimePicker { time: clock(), ontimechange: move |value| clock.set(value),
                        onselect: move |value: IxTime| last_selection.set(format!("Confirmed time: {value}")), }
                }
                ClockCompositionExample { time: clock(), ontimechange: move |value| clock.set(value) }
                div { h3 { "15-minute intervals, 08:30–17:00" }
                    TimePicker { time: clock(), format: TimeFormat::Hm, minute_interval: 15,
                        min_time: IxTime::new(8,30,0,0), max_time: IxTime::new(17,0,0,0), ontimechange: move |value| clock.set(value) }
                }
            }
        }
        section { class: "demo-section", id: "time-input",
            h2 { "Time and date/time inputs" }
            div { class: "demo-date-input-grid",
                TimeInput { label: "Time", value: time_text(), onvaluechange: move |value| time_text.set(value), helper_text: "Format: HH:MM:SS" }
                TimeInput { label: "Read-only time", value: "13:07:04", readonly: true, onvaluechange: |_| {} }
                TimeInput { label: "Disabled time", value: "13:07:04", disabled: true, onvaluechange: |_| {} }
                DateTimeInput { today, label: "Date and time", value: datetime_text(), onvaluechange: move |value| datetime_text.set(value),
                    helper_text: "Calendar and clock share a confirmation action." }
                DateTimeInput { today, label: "Read-only date/time", value: "2026/10/16 13:07:04", readonly: true, onvaluechange: |_| {} }
            }
            p { role: "status", "Time: {time_text}. Date/time: {datetime_text}." }
        }
        section { class: "demo-section", id: "date-dropdown",
            h2 { "Date dropdown presets" }
            div { class: "demo-row",
                DateDropdown { today, options, date_range_id: preset_id(), from: range().from, to: range().to,
                    onrangechange: move |value: DateRangeChange| { preset_id.set(value.id); range.set(value.dates); }, }
                DateDropdown { today, disabled: true, onrangechange: |_| {} }
                DateDropdown { today, loading: true, onrangechange: |_| {} }
            }
            p { role: "status", "{last_selection}" }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClockCompositionState {
    hide_heading: bool,
    embedded: bool,
    show_confirmation: bool,
    confirmed: Option<IxTime>,
}

impl Default for ClockCompositionState {
    fn default() -> Self {
        Self {
            hide_heading: true,
            embedded: true,
            show_confirmation: true,
            confirmed: None,
        }
    }
}

/// An inspection-time draft shares its controlled clock with the adjacent examples.
#[component]
pub fn ClockCompositionExample(time: IxTime, ontimechange: EventHandler<IxTime>) -> Element {
    let mut preparation = use_signal(ClockCompositionState::default);
    let current = preparation();
    let draft_text = TimeFormat::Hms.format(time);
    let confirmed_value = current
        .confirmed
        .map(|value| TimeFormat::Hms.format(value))
        .unwrap_or_default();
    let confirmation_text = current.confirmed.map_or_else(
        || "No inspection time confirmed.".to_string(),
        |value| {
            format!(
                "Confirmed inspection time: {}.",
                TimeFormat::Hms12.format(value)
            )
        },
    );
    rsx! {
        div { id: "clock-composition", class: "clock-composition",
            h3 { "Inspection time · 12-hour clock" }
            div { class: "clock-composition-options", role: "group", "aria-label": "Clock presentation",
                label { r#for: "clock-hide-heading",
                    input { id: "clock-hide-heading", r#type: "checkbox", checked: current.hide_heading,
                        onchange: move |event: FormEvent| preparation.write().hide_heading = event.checked(),
                    }
                    "Hide heading"
                }
                label { r#for: "clock-embedded",
                    input { id: "clock-embedded", r#type: "checkbox", checked: current.embedded,
                        onchange: move |event: FormEvent| preparation.write().embedded = event.checked(),
                    }
                    "Embedded appearance"
                }
                label { r#for: "clock-show-confirmation",
                    input { id: "clock-show-confirmation", r#type: "checkbox", checked: current.show_confirmation,
                        onchange: move |event: FormEvent| preparation.write().show_confirmation = event.checked(),
                    }
                    "Show confirmation"
                }
            }
            TimePicker { time, format: TimeFormat::Hms12, hide_header: current.hide_heading,
                embedded: current.embedded, has_footer: current.show_confirmation,
                ontimechange,
                onselect: move |value| preparation.write().confirmed = Some(value),
            }
            p { id: "clock-composition-status", class: "clock-composition-status", role: "status", "aria-live": "polite",
                "data-draft": draft_text.clone(), "data-confirmed": confirmed_value,
                "data-hide-heading": current.hide_heading, "data-embedded": current.embedded,
                "data-show-confirmation": current.show_confirmation,
                "Draft {draft_text}. {confirmation_text} Confirm stores this time locally; changing the draft keeps the last confirmation."
            }
            Button { id: "clock-restore", variant: ButtonVariant::Secondary,
                onclick: move |_| { preparation.set(ClockCompositionState::default()); ontimechange.call(IxTime::new(13, 7, 4, 0).unwrap()); },
                "Restore clock"
            }
        }
    }
}

/// Additional behavior review; the upstream HTML fixture adaptations stay isolated.
fn date_time_summary(selection: DateSelection, time: IxTime) -> String {
    let from = selection
        .from
        .map_or_else(|| "No date".to_string(), |date| date.to_string());
    let end = selection
        .to
        .map_or_else(String::new, |date| format!(" – {date}"));
    format!("{from}{end} at {}", TimeFormat::Hm.format(time))
}

#[component]
fn BoundaryDateTimeExample(today: IxDate) -> Element {
    let first = today.add_days(-5).unwrap();
    let last = today.add_days(5).unwrap();
    let mut dates = use_signal(|| DateSelection {
        from: Some(today),
        to: None,
    });
    let mut time = use_signal(|| IxTime::new(12, 0, 0, 0).unwrap());
    let mut range = use_signal(|| false);
    let mut typed = use_signal(|| "2026/10/16 12:00:00".to_string());
    let mut confirmed = use_signal(|| "No selection confirmed.".to_string());
    let selected = date_time_summary(dates(), time());
    rsx! {
        section { class: "demo-section", id: "datetime-boundaries",
            h2 { "Date-aware time constraints" }
            p { "Choose 11–21 October 2026. The first day starts at 13:10; the last day ends at 17:30. Middle days allow every time. Range selection ignores clock bounds." }
            div { class: "demo-row",
                Button { variant: ButtonVariant::Secondary,
                    onclick: move |_| dates.set(DateSelection { from: Some(first), to: if range() { Some(last) } else { None } }),
                    "First day" }
                Button { variant: ButtonVariant::Secondary,
                    onclick: move |_| dates.set(DateSelection { from: Some(today), to: if range() { Some(last) } else { None } }),
                    "Middle day" }
                Button { variant: ButtonVariant::Secondary,
                    onclick: move |_| dates.set(DateSelection { from: Some(last), to: if range() { Some(last) } else { None } }),
                    "Last day" }
                ToggleButton { pressed: range(), onpressedchange: move |enabled| {
                    range.set(enabled);
                    dates.set(DateSelection { from: Some(if enabled { first } else { today }), to: enabled.then_some(last) });
                }, "Range selection" }
            }
            div { id: "boundary-datetime-picker",
                DateTimePicker { today, from: dates().from, to: dates().to,
                    min_date: Some(first), max_date: Some(last),
                    min_time: IxTime::new(13,10,0,0), max_time: IxTime::new(17,30,0,0),
                    time: time(), time_format: TimeFormat::Hm, single_selection: !range(),
                    ondatechange: move |value| dates.set(value),
                    ontimechange: move |value| time.set(value),
                    onselect: move |value: DateTimeSelection| confirmed.set(format!("Confirmed {}.", date_time_summary(value.dates, value.time))),
                }
            }
            p { role: "status", id: "boundary-datetime-status", "Selected {selected}. {confirmed}" }
            div { class: "demo-date-input-grid",
                DateTimeInput { today, id: "boundary-datetime-input", label: "Validate a typed date and time",
                    value: typed(), min_date: Some(first), max_date: Some(last),
                    min_time: IxTime::new(13,10,0,0), max_time: IxTime::new(17,30,0,0),
                    helper_text: "YYYY/MM/DD HH:MM:SS. Try 2026/10/11 12:00:00, then change the day to 16.",
                    onvaluechange: move |value| typed.set(value),
                }
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/datetime.css");
