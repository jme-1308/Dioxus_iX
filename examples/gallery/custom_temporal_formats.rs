//! Host-owned Norwegian text codecs shared by temporal fields and form validation.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{collections::BTreeMap, rc::Rc};

pub const STYLES: &str = include_str!("styles/custom_temporal_formats.css");
const MONTHS: [&str; 12] = [
    "januar",
    "februar",
    "mars",
    "april",
    "mai",
    "juni",
    "juli",
    "august",
    "september",
    "oktober",
    "november",
    "desember",
];

fn parse_date_text(value: &str) -> Option<IxDate> {
    let parts: Vec<_> = value.split_whitespace().collect();
    if let [day, month, year] = parts.as_slice() {
        let month = MONTHS
            .iter()
            .position(|candidate| candidate.eq_ignore_ascii_case(month))?
            + 1;
        return IxDate::new(
            year.parse().ok()?,
            month as u8,
            day.strip_suffix('.')?.parse().ok()?,
        );
    }
    let parts: Vec<_> = value.trim().split('.').collect();
    let [day, month, year] = parts.as_slice() else {
        return None;
    };
    IxDate::new(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)
}

fn parse_clock(value: &str, separator: char) -> Option<IxTime> {
    let (hour, minute) = value.split_once(separator)?;
    if hour.len() != 2 || minute.len() != 2 {
        return None;
    }
    IxTime::new(hour.parse().ok()?, minute.parse().ok()?, 0, 0)
}

fn parse_time_text(value: &str) -> Option<IxTime> {
    let value = value.trim();
    if let Some(clock) = value.strip_prefix("kl.") {
        parse_clock(clock.trim(), '.')
    } else {
        parse_clock(value, ':')
    }
}

fn parse_datetime_text(value: &str) -> Option<IxDateTime> {
    if let Some((date, time)) = value.trim().rsplit_once(" kl. ") {
        Some(IxDateTime {
            date: parse_date_text(date)?,
            time: parse_clock(time, '.')?,
        })
    } else {
        let (date, time) = value.trim().rsplit_once(' ')?;
        Some(IxDateTime {
            date: parse_date_text(date)?,
            time: parse_time_text(time)?,
        })
    }
}

fn format_date_text(date: IxDate, numeric: bool) -> String {
    if numeric {
        format!("{:02}.{:02}.{:04}", date.day(), date.month(), date.year())
    } else {
        format!(
            "{}. {} {}",
            date.day(),
            MONTHS[usize::from(date.month() - 1)],
            date.year()
        )
    }
}

fn format_time_text(time: IxTime, numeric: bool) -> String {
    if numeric {
        format!("{:02}:{:02}", time.hour(), time.minute())
    } else {
        format!("kl. {:02}.{:02}", time.hour(), time.minute())
    }
}

fn format_datetime_text(value: IxDateTime, numeric: bool) -> String {
    format!(
        "{} {}",
        format_date_text(value.date, numeric),
        format_time_text(value.time, numeric)
    )
}

fn date_in_bounds(date: IxDate) -> bool {
    date >= IxDate::new(2026, 11, 1).unwrap() && date <= IxDate::new(2026, 11, 30).unwrap()
}

fn time_in_bounds(time: IxTime) -> bool {
    time >= IxTime::new(8, 0, 0, 0).unwrap() && time <= IxTime::new(17, 0, 0, 0).unwrap()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AppointmentDraft {
    date: String,
    time: String,
    reminder: String,
}
impl Default for AppointmentDraft {
    fn default() -> Self {
        Self {
            date: "3. november 2026".into(),
            time: "kl. 09.30".into(),
            reminder: "4. november 2026 kl. 14.30".into(),
        }
    }
}

fn change_presentation(draft: &AppointmentDraft, numeric: bool) -> AppointmentDraft {
    AppointmentDraft {
        date: parse_date_text(&draft.date)
            .filter(|date| date_in_bounds(*date))
            .map(|date| format_date_text(date, numeric))
            .unwrap_or_else(|| draft.date.clone()),
        time: parse_time_text(&draft.time)
            .filter(|time| time_in_bounds(*time))
            .map(|time| format_time_text(time, numeric))
            .unwrap_or_else(|| draft.time.clone()),
        reminder: parse_datetime_text(&draft.reminder)
            .filter(|value| date_in_bounds(value.date) && time_in_bounds(value.time))
            .map(|value| format_datetime_text(value, numeric))
            .unwrap_or_else(|| draft.reminder.clone()),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Appointment {
    date: IxDate,
    time: IxTime,
    reminder: IxDateTime,
}

fn submitted_appointment(values: &BTreeMap<String, String>) -> Option<Appointment> {
    let date = parse_date_text(values.get("inspection-date")?)?;
    let time = parse_time_text(values.get("start-time")?)?;
    let reminder = parse_datetime_text(values.get("reminder")?)?;
    (date_in_bounds(date)
        && time_in_bounds(time)
        && date_in_bounds(reminder.date)
        && time_in_bounds(reminder.time))
    .then_some(Appointment {
        date,
        time,
        reminder,
    })
}

fn iso_datetime(value: IxDateTime) -> String {
    format!("{}T{}", value.date, TimeFormat::Hm.format(value.time))
}

fn norwegian_calendar_labels() -> CalendarLabels {
    CalendarLabels {
        months: MONTHS.map(str::to_owned),
        weekdays: [
            "mandag", "tirsdag", "onsdag", "torsdag", "fredag", "lørdag", "søndag",
        ]
        .map(str::to_owned),
        previous_month: "Forrige måned".into(),
        next_month: "Neste måned".into(),
        select_month: "Velg måned".into(),
        select_year: "Velg år".into(),
        done: "Ferdig".into(),
    }
}

#[component]
fn AppointmentPreparation(generation: usize) -> Element {
    let mut numeric = use_signal(|| false);
    let mut draft = use_signal(AppointmentDraft::default);
    let mut typed_date = use_signal(|| parse_date_text(&AppointmentDraft::default().date));
    let mut typed_time = use_signal(|| parse_time_text(&AppointmentDraft::default().time));
    let mut typed_reminder =
        use_signal(|| parse_datetime_text(&AppointmentDraft::default().reminder));
    let mut snapshot = use_signal(|| None::<(Appointment, BTreeMap<String, String>)>);
    let mut error = use_signal(|| None::<String>);
    let mut date_node = use_signal(|| None::<Rc<MountedData>>);
    let codec_revision = u64::from(numeric());
    let date_codec = DateTextFormat::new(
        use_callback(move |value: String| parse_date_text(&value)),
        use_callback(move |value: IxDate| format_date_text(value, *numeric.peek())),
    )
    .with_revision(codec_revision);
    let time_codec = TimeTextFormat::new(
        use_callback(move |value: String| parse_time_text(&value)),
        use_callback(move |value: IxTime| format_time_text(value, *numeric.peek())),
    )
    .with_revision(codec_revision);
    let datetime_codec = DateTimeTextFormat::new(
        use_callback(move |value: String| parse_datetime_text(&value)),
        use_callback(move |value: IxDateTime| format_datetime_text(value, *numeric.peek())),
    )
    .with_revision(codec_revision);
    let today = IxDate::new(2026, 11, 3).unwrap();
    let minimum_date = IxDate::new(2026, 11, 1).unwrap();
    let maximum_date = IxDate::new(2026, 11, 30).unwrap();
    let minimum_time = IxTime::new(8, 0, 0, 0).unwrap();
    let maximum_time = IxTime::new(17, 0, 0, 0).unwrap();
    let current = draft();
    let saved = snapshot();
    let payload = saved
        .as_ref()
        .map(|(_, values)| serde_json::to_string(values).unwrap_or_default())
        .unwrap_or_else(|| "{}".into());
    let date_iso = typed_date()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".into());
    let time_iso = typed_time()
        .map(|value| TimeFormat::Hm.format(value))
        .unwrap_or_else(|| "none".into());
    let reminder_iso = typed_reminder()
        .map(iso_datetime)
        .unwrap_or_else(|| "none".into());
    rsx! {
        div { id: "custom-temporal-owner", "data-generation": generation,
            label { class: "custom-temporal-option",
                input { id: "custom-temporal-numeric", r#type: "checkbox", checked: numeric(),
                    onchange: move |event: FormEvent| {
                        let next_mode = event.checked();
                        let next = change_presentation(&draft.peek(), next_mode);
                        draft.set(next); numeric.set(next_mode);
                    },
                }
                "Numeric presentation"
            }
            p { class: "custom-temporal-help", "Changing presentation keeps unfinished or out-of-range edits. Dates stay in November 2026 and times between 08:00 and 17:00." }
            form { id: "custom-temporal-form",
                onsubmit: move |event: FormEvent| {
                    event.prevent_default();
                    let values = event.values().into_iter().filter_map(|(name, value)| match value {
                        dioxus::html::FormValue::Text(value) => Some((name, value)), _ => None,
                    }).collect::<BTreeMap<_, _>>();
                    if let Some(appointment) = submitted_appointment(&values) { snapshot.set(Some((appointment, values))); error.set(None); }
                    else { error.set(Some("Enter a valid November date and times between 08:00 and 17:00 before saving.".into())); }
                },
                div { class: "custom-temporal-fields",
                    DateInput { today, id: "custom-temporal-date", name: "inspection-date", label: "Inspection date", required: true, enable_top_layer: true,
                        value: current.date, format: DateFormat::Iso, text_format: date_codec, min_date: minimum_date, max_date: maximum_date,
                        labels: norwegian_calendar_labels(), calendar_button_label: "Open inspection calendar",
                        helper_text: if numeric() { "Use 03.11.2026, or choose a November date." } else { "Use 3. november 2026, or choose a November date." },
                        onmounted: move |event: MountedEvent| date_node.set(Some(event.data())),
                        onvaluechange: move |value| draft.write().date = value,
                        ondatechange: move |value| typed_date.set(value),
                    }
                    TimeInput { id: "custom-temporal-time", name: "start-time", label: "Start time", required: true, enable_top_layer: true,
                        value: current.time, text_format: time_codec, format: TimeFormat::Hm, min_time: minimum_time, max_time: maximum_time,
                        time_button_label: "Open start time picker",
                        helper_text: if numeric() { "Use 09:30, between 08:00 and 17:00." } else { "Use kl. 09.30, between kl. 08.00 and kl. 17.00." },
                        onvaluechange: move |value| draft.write().time = value,
                        ontimechange: move |value| typed_time.set(value),
                    }
                    DateTimeInput { today, id: "custom-temporal-reminder", name: "reminder", label: "Reminder", required: true, enable_top_layer: true,
                        value: current.reminder, text_format: datetime_codec, date_format: DateFormat::Iso, time_format: TimeFormat::Hm,
                        min_date: minimum_date, max_date: maximum_date, min_time: minimum_time, max_time: maximum_time,
                        calendar_labels: norwegian_calendar_labels(), calendar_button_label: "Open reminder calendar", done_label: "Ferdig",
                        helper_text: if numeric() { "Use 04.11.2026 14:30, or choose a date and time." } else { "Use 4. november 2026 kl. 14.30, or choose a date and time." },
                        onvaluechange: move |value| draft.write().reminder = value,
                        ondatetimechange: move |value| typed_reminder.set(value),
                    }
                }
                div { class: "custom-temporal-actions",
                    Button { id: "custom-temporal-save", button_type: ButtonType::Submit, "Save preparation" }
                    Button { id: "custom-temporal-edit", button_type: ButtonType::Button, variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            if let Some(node) = date_node.peek().clone() {
                                spawn(async move {
                                    if date_node.peek().as_ref().is_some_and(|current| Rc::ptr_eq(current, &node)) { let _ = node.set_focus(true).await; }
                                });
                            }
                        }, "Edit inspection date"
                    }
                }
            }
            if let Some(message) = error() { p { id: "custom-temporal-error", class: "custom-temporal-error", role: "alert", "{message}" } }
            p { id: "custom-temporal-status", class: "custom-temporal-status", role: "status", "aria-live": "polite",
                "data-date": date_iso.clone(), "data-time": time_iso.clone(), "data-reminder": reminder_iso.clone(),
                "data-mode": if numeric() { "numeric" } else { "norwegian" }, "data-saved": saved.is_some(), "data-payload": payload,
                "Parsed inspection: {date_iso} · Start: {time_iso} · Reminder: {reminder_iso}"
            }
            if let Some((appointment, _)) = saved {
                div { class: "custom-temporal-saved",
                    h3 { "Saved preparation" }
                    p { "This snapshot stays unchanged while you edit. Nothing was scheduled or sent." }
                    dl { id: "custom-temporal-snapshot", "data-date": appointment.date.to_string(),
                        "data-time": TimeFormat::Hm.format(appointment.time), "data-reminder": iso_datetime(appointment.reminder),
                        div { dt { "Inspection date" } dd { "{appointment.date}" } }
                        div { dt { "Start time" } dd { "{TimeFormat::Hm.format(appointment.time)}" } }
                        div { dt { "Reminder" } dd { "{iso_datetime(appointment.reminder)}" } }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CustomTemporalFormatsExample() -> Element {
    let mut generation = use_signal(|| 0_usize);
    rsx! {
        style { {STYLES} }
        section { id: "custom-temporal-formats", class: "demo-section custom-temporal-formats-example",
            h2 { "Prepare an inspection in Norwegian" }
            p { class: "custom-temporal-intro", "Enter a maintenance appointment using Norwegian month names and your team's clock notation. The same Rust text rules drive typing, pickers and this local form; no request is sent to equipment or a scheduling service." }
            for owner in [generation()] { AppointmentPreparation { key: "{owner}", generation: owner } }
            Button { id: "custom-temporal-reset", button_type: ButtonType::Button, variant: ButtonVariant::Tertiary,
                onclick: move |_| generation += 1, "Reset preparation"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norwegian_and_numeric_codecs_round_trip_and_reject_impossible_values() {
        let date = IxDate::new(2026, 11, 3).unwrap();
        let time = IxTime::new(9, 30, 0, 0).unwrap();
        let value = IxDateTime { date, time };
        for numeric in [false, true] {
            assert_eq!(
                parse_date_text(&format_date_text(date, numeric)),
                Some(date)
            );
            assert_eq!(
                parse_time_text(&format_time_text(time, numeric)),
                Some(time)
            );
            assert_eq!(
                parse_datetime_text(&format_datetime_text(value, numeric)),
                Some(value)
            );
        }
        assert!(parse_date_text("31. november 2026").is_none());
        assert!(parse_date_text("3. unknown 2026").is_none());
        assert!(parse_date_text("2026-11-03").is_none());
        assert!(parse_time_text("kl. 25.00").is_none());
        assert!(parse_time_text("kl. 09.30.00").is_none());
    }

    #[test]
    fn presentation_changes_preserve_invalid_drafts_and_save_uses_the_same_bounds() {
        let draft = AppointmentDraft {
            date: "unfinished month".into(),
            time: "kl. 07.00".into(),
            ..Default::default()
        };
        let changed = change_presentation(&draft, true);
        assert_eq!(changed.date, draft.date);
        assert_eq!(changed.time, draft.time);
        assert_eq!(changed.reminder, "04.11.2026 14:30");
        let mut values = BTreeMap::from([
            ("inspection-date".into(), "3. november 2026".into()),
            ("start-time".into(), "kl. 09.30".into()),
            ("reminder".into(), "4. november 2026 kl. 14.30".into()),
        ]);
        let saved = submitted_appointment(&values).unwrap();
        values.insert("reminder".into(), "4. desember 2026 kl. 14.30".into());
        assert!(submitted_appointment(&values).is_none());
        assert_eq!(iso_datetime(saved.reminder), "2026-11-04T14:30");
        values.insert("inspection-date".into(), String::new());
        assert!(submitted_appointment(&values).is_none());
    }
}
