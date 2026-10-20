//! Localized date and date/time input formatting and parsing across supported languages.
use dioxus::prelude::*;
use dioxus_ix::*;

const SAMPLE_DATE: IxDate = match IxDate::new(2026, 10, 20) {
    Some(date) => date,
    None => unreachable!(),
};
const SAMPLE_TIME: IxTime = match IxTime::new(13, 30, 0, 0) {
    Some(time) => time,
    None => unreachable!(),
};
const MIN_DATE: IxDate = match IxDate::new(2026, 10, 1) {
    Some(date) => date,
    None => unreachable!(),
};
const MAX_DATE: IxDate = match IxDate::new(2026, 11, 30) {
    Some(date) => date,
    None => unreachable!(),
};

const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("de", "Deutsch"),
    ("fr", "Français"),
    ("ja", "日本語"),
];

fn calendar_locale(code: &str) -> CalendarLocale {
    match code {
        "de" => CalendarLocale::De,
        "fr" => CalendarLocale::Fr,
        "ja" => CalendarLocale::Ja,
        _ => CalendarLocale::En,
    }
}

fn sample_drafts(locale: CalendarLocale) -> [String; 4] {
    let long = DateFormat::LongMonth(locale);
    let short = DateFormat::ShortMonth(locale);
    let weekday = DateFormat::WeekdayLongMonth(locale);
    let datetime = IxDateTime {
        date: SAMPLE_DATE,
        time: SAMPLE_TIME,
    };
    [
        long.format(SAMPLE_DATE),
        short.format(SAMPLE_DATE),
        weekday.format(SAMPLE_DATE),
        datetime.format(long, TimeFormat::Hm12),
    ]
}

fn parsed_iso_date(value: &str, format: DateFormat) -> String {
    format
        .parse(value)
        .filter(|date| *date >= MIN_DATE && *date <= MAX_DATE)
        .map_or_else(|| "invalid".to_string(), |date| date.to_string())
}

fn parsed_iso_datetime(value: &str, date_format: DateFormat, time_format: TimeFormat) -> String {
    IxDateTime::parse(value, date_format, time_format)
        .filter(|dt| dt.date >= MIN_DATE && dt.date <= MAX_DATE)
        .map_or_else(
            || "invalid".to_string(),
            |dt| format!("{}T{}", dt.date, dt.time),
        )
}

#[component]
pub fn LocalizedDatesExample() -> Element {
    let mut locale_code = use_signal(|| "en".to_string());
    let mut revision = use_signal(|| 0_usize);
    let mut drafts = use_signal(|| sample_drafts(CalendarLocale::En));

    let code = locale_code();
    let locale = calendar_locale(&code);
    let long_format = DateFormat::LongMonth(locale);
    let short_format = DateFormat::ShortMonth(locale);
    let weekday_format = DateFormat::WeekdayLongMonth(locale);
    let time_format = TimeFormat::Hm12;

    let [long_val, short_val, weekday_val, datetime_val] = drafts();
    let data_long = parsed_iso_date(&long_val, long_format);
    let data_short = parsed_iso_date(&short_val, short_format);
    let data_weekday = parsed_iso_date(&weekday_val, weekday_format);
    let data_datetime = parsed_iso_datetime(&datetime_val, long_format, time_format);
    let form_key = format!("{code}-{}", revision());

    rsx! {
        section { id: "localized-dates", class: "demo-section localized-dates",
            h2 { "Localized date and time inputs" }
            p {
                "Format and parse month and weekday names across the four supported languages, "
                "with a 12-hour AM/PM appointment clock between 1 October and 30 November 2026. "
                "The illustrative sample is 20 October 2026 at 13:30. "
                "Changing the language reloads the sample values."
            }
            div { class: "localized-dates-toolbar",
                label { class: "localized-dates-language", r#for: "localized-locale",
                    span { "Language" }
                    select {
                        id: "localized-locale",
                        value: "{code}",
                        onchange: move |event| {
                            let next = event.value();
                            let next_locale = calendar_locale(&next);
                            locale_code.set(next);
                            drafts.set(sample_drafts(next_locale));
                            revision += 1;
                        },
                        for (value, label) in LANGUAGES.iter().copied() {
                            option { value, selected: code == value, "{label}" }
                        }
                    }
                }
                Button {
                    id: "localized-reset",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        let current_locale = calendar_locale(&locale_code());
                        drafts.set(sample_drafts(current_locale));
                        revision += 1;
                    },
                    "Restore sample"
                }
            }
            for key in [form_key] {
                div { key: "{key}", class: "localized-dates-grid",
                    DateInput {
                        id: "localized-long",
                        today: SAMPLE_DATE,
                        label: "Long date",
                        value: long_val.clone(),
                        format: long_format,
                        min_date: Some(MIN_DATE),
                        max_date: Some(MAX_DATE),
                        required: true,
                        labels: locale.labels(),
                        helper_text: long_format.pattern().to_string(),
                        onvaluechange: move |next| drafts.write()[0] = next,
                    }
                    DateInput {
                        id: "localized-short",
                        today: SAMPLE_DATE,
                        label: "Short date",
                        value: short_val.clone(),
                        format: short_format,
                        min_date: Some(MIN_DATE),
                        max_date: Some(MAX_DATE),
                        required: true,
                        labels: locale.labels(),
                        helper_text: short_format.pattern().to_string(),
                        onvaluechange: move |next| drafts.write()[1] = next,
                    }
                    DateInput {
                        id: "localized-weekday",
                        today: SAMPLE_DATE,
                        label: "Weekday date",
                        value: weekday_val.clone(),
                        format: weekday_format,
                        min_date: Some(MIN_DATE),
                        max_date: Some(MAX_DATE),
                        required: true,
                        labels: locale.labels(),
                        helper_text: weekday_format.pattern().to_string(),
                        onvaluechange: move |next| drafts.write()[2] = next,
                    }
                    DateTimeInput {
                        id: "localized-datetime",
                        today: SAMPLE_DATE,
                        label: "Appointment",
                        value: datetime_val.clone(),
                        date_format: long_format,
                        time_format,
                        min_date: Some(MIN_DATE),
                        max_date: Some(MAX_DATE),
                        required: true,
                        calendar_labels: locale.labels(),
                        time_labels: locale.time_labels(),
                        helper_text: format!("{} {}", long_format.pattern(), time_format.pattern()),
                        onvaluechange: move |next| drafts.write()[3] = next,
                    }
                }
            }
            output {
                id: "localized-result",
                class: "localized-dates-result",
                role: "status",
                "data-long": "{data_long}",
                "data-short": "{data_short}",
                "data-weekday": "{data_weekday}",
                "data-datetime": "{data_datetime}",
                span { "Long: " code { "{data_long}" } }
                span { "Short: " code { "{data_short}" } }
                span { "Weekday: " code { "{data_weekday}" } }
                span { "Appointment: " code { "{data_datetime}" } }
            }
        }
    }
}

pub const STYLES: &str = r#"
.localized-dates{min-width:0;max-width:100%}.localized-dates-toolbar{display:flex;flex-wrap:wrap;align-items:flex-end;gap:16px;margin-block:16px;min-width:0}.localized-dates-language{display:flex;flex-direction:column;gap:6px;min-width:0;font:14px/20px var(--ix-font)}.localized-dates-language>select{min-height:var(--ix-control-height,40px);min-width:160px;max-width:100%;padding:8px 12px;background:var(--ix-color-surface-1);color:var(--ix-color-text);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);font:14px/20px var(--ix-font)}.localized-dates-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,240px),1fr));gap:20px;margin-block:20px;min-width:0}.localized-dates-grid>.ix-temporal-field{min-width:0;width:100%}.localized-dates-result{display:flex;flex-wrap:wrap;gap:12px 24px;padding:12px 16px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);font:14px/20px var(--ix-font);overflow-wrap:anywhere}.localized-dates-result code{color:var(--ix-color-primary);font-variant-numeric:tabular-nums}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localized_dates_example_renders_initial_english_sample_and_iso_output() {
        let mut dom = VirtualDom::new(LocalizedDatesExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("id=\"localized-dates\""));
        assert!(html.contains("id=\"localized-locale\""));
        assert!(html.contains("id=\"localized-reset\""));
        assert!(html.contains("id=\"localized-long\""));
        assert!(html.contains("id=\"localized-short\""));
        assert!(html.contains("id=\"localized-weekday\""));
        assert!(html.contains("id=\"localized-datetime\""));
        assert!(html.contains("id=\"localized-result\""));
        assert!(html.contains("data-long=\"2026-10-20\""));
        assert!(html.contains("data-short=\"2026-10-20\""));
        assert!(html.contains("data-weekday=\"2026-10-20\""));
        assert!(html.contains("data-datetime=\"2026-10-20T13:30:00\""));
    }
}
