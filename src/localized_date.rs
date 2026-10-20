//! Strict Rust parsing for the named formats already emitted by CalendarLocale.
use crate::{CalendarLocale, IxDate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NamedDateStyle {
    Long,
    Short,
    WithWeekday,
}

fn decimal(value: &str, min_digits: usize, max_digits: usize) -> Option<u16> {
    (value.len() >= min_digits
        && value.len() <= max_digits
        && value.bytes().all(|byte| byte.is_ascii_digit()))
    .then(|| value.parse().ok())
    .flatten()
}

fn same_month(value: &str, canonical: &str, short: bool) -> bool {
    let without_optional_dot = |text: &str| {
        if short {
            text.strip_suffix('.').unwrap_or(text).to_lowercase()
        } else {
            text.to_lowercase()
        }
    };
    without_optional_dot(value) == without_optional_dot(canonical)
}

/// Parse only known locale labels and validate the resulting Gregorian date.
/// Short names come from the existing formatter, including its locale-specific
/// abbreviations; a single abbreviation period may be supplied or omitted.
/// This parser does not guess another locale, date order, or unknown names.
pub(crate) fn parse_named_date(
    value: &str,
    locale: CalendarLocale,
    style: NamedDateStyle,
) -> Option<IxDate> {
    let labels = locale.labels();
    let (weekday, date_text) = if style == NamedDateStyle::WithWeekday {
        let (weekday, date) = value.split_once(',')?;
        let weekday = weekday.trim().to_lowercase();
        let index = labels
            .weekdays
            .iter()
            .position(|label| label.to_lowercase() == weekday)?;
        (Some(index), date)
    } else {
        (None, value)
    };
    let mut parts = date_text.split_whitespace();
    let day = decimal(parts.next()?, 1, 2)?;
    let month_name = parts.next()?;
    let year = decimal(parts.next()?, 4, 4)?;
    if parts.next().is_some() {
        return None;
    }
    let month = if style == NamedDateStyle::Short {
        (1u8..=12).find(|month| {
            let reference = IxDate::new(2000, *month, 1).expect("valid reference month");
            locale
                .format_date(reference, "dd MMM yyyy")
                .and_then(|formatted| {
                    formatted
                        .split_whitespace()
                        .nth(1)
                        .map(|canonical| same_month(month_name, canonical, true))
                })
                .unwrap_or(false)
        })?
    } else {
        let index = labels
            .months
            .iter()
            .position(|label| same_month(month_name, label, false))?;
        (index + 1) as u8
    };
    let date = IxDate::new(year, month, day.try_into().ok()?)?;
    if weekday.is_some_and(|expected| expected != usize::from(date.weekday())) {
        return None;
    }
    Some(date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatter_named_dates_roundtrip_in_all_supported_locales_and_styles() {
        let styles = [
            (NamedDateStyle::Long, "dd MMMM yyyy"),
            (NamedDateStyle::Short, "dd MMM yyyy"),
            (NamedDateStyle::WithWeekday, "EEEE, dd MMMM yyyy"),
        ];
        for locale in [
            CalendarLocale::En,
            CalendarLocale::De,
            CalendarLocale::Fr,
            CalendarLocale::Ja,
        ] {
            for date in [
                IxDate::new(2026, 10, 20).unwrap(),
                IxDate::new(2024, 2, 29).unwrap(),
            ] {
                for (style, pattern) in styles {
                    let text = locale.format_date(date, pattern).unwrap();
                    assert_eq!(
                        parse_named_date(&text, locale, style),
                        Some(date),
                        "{locale:?} {style:?}: {text}"
                    );
                }
            }
        }
    }

    #[test]
    fn localized_names_accept_unicode_case_whitespace_and_short_month_periods() {
        for (text, locale, style, expected) in [
            (
                " 1 MÄRZ 2026 ",
                CalendarLocale::De,
                NamedDateStyle::Long,
                IxDate::new(2026, 3, 1).unwrap(),
            ),
            (
                "29\tFÉVR.\n2024",
                CalendarLocale::Fr,
                NamedDateStyle::Short,
                IxDate::new(2024, 2, 29).unwrap(),
            ),
            (
                "29 févr 2024",
                CalendarLocale::Fr,
                NamedDateStyle::Short,
                IxDate::new(2024, 2, 29).unwrap(),
            ),
            (
                "20 OCT. 2026",
                CalendarLocale::En,
                NamedDateStyle::Short,
                IxDate::new(2026, 10, 20).unwrap(),
            ),
            (
                "Dienstag , 20 OKTOBER 2026",
                CalendarLocale::De,
                NamedDateStyle::WithWeekday,
                IxDate::new(2026, 10, 20).unwrap(),
            ),
            (
                "\tMARDI\u{a0},\n20 OCTOBRE 2026 ",
                CalendarLocale::Fr,
                NamedDateStyle::WithWeekday,
                IxDate::new(2026, 10, 20).unwrap(),
            ),
            (
                "火曜日, 20 10月 2026",
                CalendarLocale::Ja,
                NamedDateStyle::WithWeekday,
                IxDate::new(2026, 10, 20).unwrap(),
            ),
        ] {
            assert_eq!(
                parse_named_date(text, locale, style),
                Some(expected),
                "{text}"
            );
        }
    }

    #[test]
    fn impossible_dates_unknown_names_extra_tokens_and_mismatched_weekdays_are_rejected() {
        for text in [
            "29 February 1900",
            "29 February 2023",
            "30 February 2024",
            "31 April 2026",
            "0 October 2026",
            "32 October 2026",
            "001 October 2026",
            "+1 October 2026",
            "20 October 0000",
            "20 October 10000",
            "20 October 26",
            "20 October +2026",
            "20 Smarch 2026",
            "20 Oktober 2026",
            "20 October. 2026",
            "20 October 2026 extra",
            "Tuesday, 20 October 2026",
            "20 2026 October",
            "",
            "20 October",
        ] {
            assert_eq!(
                parse_named_date(text, CalendarLocale::En, NamedDateStyle::Long),
                None,
                "{text}"
            );
        }
        for text in ["20 Oct.. 2026", "20 October 2026", "20 Octx 2026"] {
            assert_eq!(
                parse_named_date(text, CalendarLocale::En, NamedDateStyle::Short),
                None,
                "{text}"
            );
        }
        for text in [
            "Monday, 20 October 2026",
            "Tue, 20 October 2026",
            "Tuesday 20 October 2026",
            "Tuesday,, 20 October 2026",
            ", 20 October 2026",
            "Tuesday, 20 October 2026,",
        ] {
            assert_eq!(
                parse_named_date(text, CalendarLocale::En, NamedDateStyle::WithWeekday),
                None,
                "{text}"
            );
        }
        assert_eq!(
            parse_named_date("1 January 0001", CalendarLocale::En, NamedDateStyle::Long),
            IxDate::new(1, 1, 1)
        );
        assert_eq!(
            parse_named_date("31 December 9999", CalendarLocale::En, NamedDateStyle::Long),
            IxDate::new(9999, 12, 31)
        );
    }
}
