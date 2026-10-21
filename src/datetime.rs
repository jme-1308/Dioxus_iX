//! Calendar and clock controls implemented entirely in Rust.
//!
//! Date values have no timezone. Supply the user's current local date through
//! `today`; the library deliberately avoids guessing a timezone on the server.
//! Temporal inputs expose their real native input through `onmounted`. Retain
//! `event.data()` and call `MountedData::set_focus(true).await` for host-owned
//! focus, the Dioxus adaptation of upstream `focusInput`.
pub use crate::date_dropdown_controller::{
    DateDropdownController, DateDropdownError, use_date_dropdown,
};
use crate::temporal_text_format::{format_text, parse_text};
use crate::{DateTextFormat, DateTimeTextFormat, TimeTextFormat};
use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    fmt,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_DATE_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
struct RangeTarget {
    input: Rc<RefCell<Option<Rc<MountedData>>>>,
    enabled: Rc<Cell<bool>>,
    activate: Callback<()>,
}

#[derive(Clone)]
struct TemporalRangeContext {
    targets: Rc<RefCell<BTreeMap<usize, RangeTarget>>>,
    next: Rc<Cell<usize>>,
    keyboard: Rc<Cell<bool>>,
}

/// Install the range's local focus scope without any browser-specific API.
pub(crate) fn provide_range_navigation() {
    use_context_provider(|| TemporalRangeContext {
        targets: Rc::new(RefCell::new(BTreeMap::new())),
        next: Rc::new(Cell::new(0)),
        keyboard: Rc::new(Cell::new(false)),
    });
}

#[derive(Clone)]
struct RangeBinding {
    context: Option<TemporalRangeContext>,
    id: usize,
    input: Rc<RefCell<Option<Rc<MountedData>>>>,
    enabled: Rc<Cell<bool>>,
}
impl PartialEq for RangeBinding {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Rc::ptr_eq(&self.input, &other.input)
    }
}
fn range_handoff_target(
    source: usize,
    keyboard: bool,
    mut targets: impl Iterator<Item = (usize, bool)>,
) -> Option<usize> {
    if keyboard || targets.next().map(|(id, _)| id) != Some(source) {
        return None;
    }
    targets
        .next()
        .and_then(|(id, enabled)| enabled.then_some(id))
}
impl RangeBinding {
    fn input_mode(&self, keyboard: bool) {
        if let Some(context) = &self.context {
            context.keyboard.set(keyboard);
        }
    }
    fn advance(&self) -> bool {
        let Some(context) = &self.context else {
            return false;
        };
        let targets = context.targets.borrow();
        let next = range_handoff_target(
            self.id,
            context.keyboard.get(),
            targets
                .iter()
                .map(|(id, target)| (*id, target.enabled.get())),
        );
        let Some(next) = next.and_then(|id| targets.get(&id)).cloned() else {
            return false;
        };
        spawn(async move {
            if !next.enabled.get() {
                return;
            }
            let input = next.input.borrow().clone();
            if let Some(input) = input {
                let _ = input.set_focus(true).await;
            }
            if next.enabled.get() {
                next.activate.call(());
            }
        });
        true
    }
}
fn use_range_binding(activate: Callback<()>, enabled: bool) -> RangeBinding {
    let binding = use_hook(|| {
        let context = try_consume_context::<TemporalRangeContext>();
        let id = context.as_ref().map_or(0, |context| {
            let id = context.next.get();
            context.next.set(id + 1);
            id
        });
        let input = Rc::new(RefCell::new(None));
        let enabled = Rc::new(Cell::new(enabled));
        if let Some(context) = &context {
            context.targets.borrow_mut().insert(
                id,
                RangeTarget {
                    input: input.clone(),
                    enabled: enabled.clone(),
                    activate,
                },
            );
        }
        RangeBinding {
            context,
            id,
            input,
            enabled,
        }
    });
    binding.enabled.set(enabled);
    let cleanup = binding.clone();
    use_drop(move || {
        cleanup.enabled.set(false);
        if let Some(context) = cleanup.context {
            context.targets.borrow_mut().remove(&cleanup.id);
        }
    });
    binding
}

/// A validated Gregorian date in the inclusive year range 1–9999.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IxDate {
    year: u16,
    month: u8,
    day: u8,
}
impl IxDate {
    /// Construct a valid calendar date.
    pub const fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        if year >= 1
            && year <= 9999
            && month >= 1
            && month <= 12
            && day >= 1
            && day <= Self::days_in_month(year, month)
        {
            Some(Self { year, month, day })
        } else {
            None
        }
    }
    /// Calendar year.
    pub const fn year(self) -> u16 {
        self.year
    }
    /// Calendar month, January = 1.
    pub const fn month(self) -> u8 {
        self.month
    }
    /// Day of month, starting at 1.
    pub const fn day(self) -> u8 {
        self.day
    }
    /// Gregorian leap-year rule, including century exceptions.
    pub const fn is_leap_year(year: u16) -> bool {
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
    }
    /// Number of days in a month; invalid months have zero days.
    pub const fn days_in_month(year: u16, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        }
    }
    /// Days since 1970-01-01, also allowing dates before the epoch.
    pub fn days_since_epoch(self) -> i64 {
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let yoe = year - era * 400;
        let month = i64::from(self.month);
        let doy =
            (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468
    }
    /// Construct a date from its epoch-day index, rejecting unsupported years.
    pub fn from_epoch_days(days: i64) -> Option<Self> {
        if !(-719162..=2932896).contains(&days) {
            return None;
        }
        let z = days + 719468;
        let era = z.div_euclid(146097);
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let mut year = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = mp + if mp < 10 { 3 } else { -9 };
        year += i64::from(month <= 2);
        Self::new(year as u16, month as u8, day as u8)
    }
    /// Add calendar days, returning `None` outside the supported date range.
    pub fn add_days(self, days: i64) -> Option<Self> {
        Self::from_epoch_days(self.days_since_epoch().checked_add(days)?)
    }
    /// Add months, clamping the day to the last day of the target month.
    pub fn add_months(self, months: i32) -> Option<Self> {
        let index = (i32::from(self.year) - 1) * 12 + i32::from(self.month) - 1;
        let next = index.checked_add(months)?;
        if !(0..9999 * 12).contains(&next) {
            return None;
        }
        let year = (next / 12 + 1) as u16;
        let month = (next % 12 + 1) as u8;
        Self::new(year, month, self.day.min(Self::days_in_month(year, month)))
    }
    /// Weekday index, Monday = 0 and Sunday = 6, matching iX weekStartIndex.
    pub fn weekday(self) -> u8 {
        (self.days_since_epoch() + 3).rem_euclid(7) as u8
    }
    /// ISO week number and corresponding ISO week year.
    pub fn iso_week(self) -> (u16, u8) {
        let thursday = self.add_days(3 - i64::from(self.weekday())).unwrap_or(self);
        let year = thursday.year;
        let jan4 = Self::new(year, 1, 4).unwrap();
        let first_monday = jan4.days_since_epoch() - i64::from(jan4.weekday());
        (
            year,
            ((self.days_since_epoch() - first_monday).div_euclid(7) + 1) as u8,
        )
    }
    /// Parse the canonical `YYYY-MM-DD` representation.
    pub fn parse_iso(value: &str) -> Option<Self> {
        DateFormat::Iso.parse(value)
    }
    fn month_start(self) -> Self {
        Self { day: 1, ..self }
    }
    fn within(self, min: Option<Self>, max: Option<Self>) -> bool {
        min.is_none_or(|min| self >= min) && max.is_none_or(|max| self <= max)
    }
}
impl fmt::Display for IxDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Supported unambiguous date input formats. Default matches iX `yyyy/LL/dd`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateFormat {
    #[default]
    YmdSlash,
    Iso,
    DmyDot,
    DmySlash,
    MdySlash,
    /// Day, localized full month name, and four-digit year.
    LongMonth(CalendarLocale),
    /// Day, localized abbreviated month name, and four-digit year.
    ShortMonth(CalendarLocale),
    /// Matching localized weekday, day, full month name, and year.
    WeekdayLongMonth(CalendarLocale),
}
impl DateFormat {
    /// A placeholder describing the expected input.
    pub const fn pattern(self) -> &'static str {
        match self {
            Self::YmdSlash => "YYYY/MM/DD",
            Self::Iso => "YYYY-MM-DD",
            Self::DmyDot => "DD.MM.YYYY",
            Self::DmySlash => "DD/MM/YYYY",
            Self::MdySlash => "MM/DD/YYYY",
            Self::LongMonth(_) => "DD Month YYYY",
            Self::ShortMonth(_) => "DD Mon YYYY",
            Self::WeekdayLongMonth(_) => "Weekday, DD Month YYYY",
        }
    }
    /// Format a validated date.
    pub fn format(self, date: IxDate) -> String {
        let IxDate { year, month, day } = date;
        match self {
            Self::YmdSlash => format!("{year:04}/{month:02}/{day:02}"),
            Self::Iso => date.to_string(),
            Self::DmyDot => format!("{day:02}.{month:02}.{year:04}"),
            Self::DmySlash => format!("{day:02}/{month:02}/{year:04}"),
            Self::MdySlash => format!("{month:02}/{day:02}/{year:04}"),
            Self::LongMonth(locale) => locale
                .format_date(date, "dd MMMM yyyy")
                .expect("supported full-month format"),
            Self::ShortMonth(locale) => locale
                .format_date(date, "dd MMM yyyy")
                .expect("supported abbreviated-month format"),
            Self::WeekdayLongMonth(locale) => locale
                .format_date(date, "EEEE, dd MMMM yyyy")
                .expect("supported weekday format"),
        }
    }
    /// Strictly parse this format; impossible dates are rejected.
    pub fn parse(self, value: &str) -> Option<IxDate> {
        use crate::localized_date::{NamedDateStyle, parse_named_date};
        let (separator, year_index, month_index, day_index) = match self {
            Self::YmdSlash => ('/', 0, 1, 2),
            Self::Iso => ('-', 0, 1, 2),
            Self::DmyDot => ('.', 2, 1, 0),
            Self::DmySlash => ('/', 2, 1, 0),
            Self::MdySlash => ('/', 2, 0, 1),
            Self::LongMonth(locale) => {
                return parse_named_date(value, locale, NamedDateStyle::Long);
            }
            Self::ShortMonth(locale) => {
                return parse_named_date(value, locale, NamedDateStyle::Short);
            }
            Self::WeekdayLongMonth(locale) => {
                return parse_named_date(value, locale, NamedDateStyle::WithWeekday);
            }
        };
        let parts: Vec<_> = value.trim().split(separator).collect();
        if parts.len() != 3
            || !parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return None;
        }
        let (year, month, day) = (parts[year_index], parts[month_index], parts[day_index]);
        if year.len() != 4 || month.len() > 2 || day.len() > 2 {
            return None;
        }
        IxDate::new(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)
    }
}

/// Date picker selection. A pending range has `from` set and `to` empty.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateSelection {
    pub from: Option<IxDate>,
    pub to: Option<IxDate>,
}
impl DateSelection {
    /// Apply the upstream first-date / second-date / restart-range selection rules.
    pub fn select(self, date: IxDate, single: bool) -> Self {
        match (single, self.from, self.to) {
            (false, Some(from), None) => Self {
                from: Some(from.min(date)),
                to: Some(from.max(date)),
            },
            _ => Self {
                from: Some(date),
                to: None,
            },
        }
    }
    /// Whether a day is an endpoint of the selection.
    pub fn is_endpoint(self, date: IxDate) -> bool {
        self.from == Some(date) || self.to == Some(date)
    }
    /// Whether a day lies strictly between two selected endpoints.
    pub fn contains_interior(self, date: IxDate) -> bool {
        matches!((self.from, self.to), (Some(from), Some(to)) if date > from && date < to)
    }
}

/// Translatable calendar names; default language is English.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarLabels {
    pub months: [String; 12],
    pub weekdays: [String; 7],
    pub previous_month: String,
    pub next_month: String,
    pub select_month: String,
    pub select_year: String,
    pub done: String,
}
impl Default for CalendarLabels {
    fn default() -> Self {
        Self {
            months: [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ]
            .map(str::to_string),
            weekdays: [
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
                "Sunday",
            ]
            .map(str::to_string),
            previous_month: "Previous month".into(),
            next_month: "Next month".into(),
            select_month: "Select month".into(),
            select_year: "Select year".into(),
            done: "Done".into(),
        }
    }
}

/// Locale names used by the upstream date-picker examples. Calendar math and ISO
/// event values stay locale independent. Applications may supply custom labels
/// for additional languages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CalendarLocale {
    #[default]
    En,
    De,
    Fr,
    Ja,
}
impl CalendarLocale {
    /// Calendar month and weekday names, retaining independently configurable
    /// navigation and action labels.
    pub fn labels(self) -> CalendarLabels {
        let (months, weekdays) = match self {
            Self::En => return CalendarLabels::default(),
            Self::De => (
                [
                    "Januar",
                    "Februar",
                    "März",
                    "April",
                    "Mai",
                    "Juni",
                    "Juli",
                    "August",
                    "September",
                    "Oktober",
                    "November",
                    "Dezember",
                ],
                [
                    "Montag",
                    "Dienstag",
                    "Mittwoch",
                    "Donnerstag",
                    "Freitag",
                    "Samstag",
                    "Sonntag",
                ],
            ),
            Self::Fr => (
                [
                    "janvier",
                    "février",
                    "mars",
                    "avril",
                    "mai",
                    "juin",
                    "juillet",
                    "août",
                    "septembre",
                    "octobre",
                    "novembre",
                    "décembre",
                ],
                [
                    "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche",
                ],
            ),
            Self::Ja => (
                [
                    "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月",
                    "12月",
                ],
                [
                    "月曜日",
                    "火曜日",
                    "水曜日",
                    "木曜日",
                    "金曜日",
                    "土曜日",
                    "日曜日",
                ],
            ),
        };
        CalendarLabels {
            months: months.map(str::to_string),
            weekdays: weekdays.map(str::to_string),
            ..Default::default()
        }
    }
    /// Format the five named/numeric formats used by the upstream locale
    /// examples. Unsupported tokens return `None`, never a guessed value.
    pub fn format_date(self, date: IxDate, pattern: &str) -> Option<String> {
        let labels = self.labels();
        let month = &labels.months[usize::from(date.month - 1)];
        let weekday = &labels.weekdays[usize::from(date.weekday())];
        let short_months = match self {
            Self::En => [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ],
            Self::De => [
                "Jan.", "Feb.", "März", "Apr.", "Mai", "Juni", "Juli", "Aug.", "Sept.", "Okt.",
                "Nov.", "Dez.",
            ],
            Self::Fr => [
                "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.",
                "nov.", "déc.",
            ],
            Self::Ja => [
                "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月",
                "12月",
            ],
        };
        Some(match pattern {
            "yyyy/LL/dd" => DateFormat::YmdSlash.format(date),
            "LL/dd/yyyy" => DateFormat::MdySlash.format(date),
            "EEEE, dd MMMM yyyy" => format!("{weekday}, {:02} {month} {:04}", date.day, date.year),
            "dd MMMM yyyy" => format!("{:02} {month} {:04}", date.day, date.year),
            "dd MMM yyyy" => format!(
                "{:02} {} {:04}",
                date.day,
                short_months[usize::from(date.month - 1)],
                date.year
            ),
            _ => return None,
        })
    }
    /// Locale-specific column labels and meridiem labels used by the upstream
    /// datetime-picker example; other action labels remain caller controlled.
    pub fn time_labels(self) -> TimeLabels {
        let (hour, minute, second, am, pm) = match self {
            Self::En => ("hr", "min", "sec", "AM", "PM"),
            Self::De => ("Std", "Min", "Sek", "AM", "PM"),
            Self::Fr => ("h", "min", "s", "AM", "PM"),
            Self::Ja => ("時", "分", "秒", "午前", "午後"),
        };
        TimeLabels {
            hour: hour.into(),
            minute: minute.into(),
            second: second.into(),
            am: am.into(),
            pm: pm.into(),
            ..Default::default()
        }
    }
    /// Locale-aware time output while keeping `IxTime` and its ISO output stable.
    pub fn format_time(self, time: IxTime, format: TimeFormat) -> String {
        let text = format.format(time);
        if self == Self::Ja && format.has_meridiem() {
            let labels = self.time_labels();
            format!(
                "{} {}",
                text.rsplit_once(' ').unwrap().0,
                if time.hour < 12 { labels.am } else { labels.pm }
            )
        } else {
            text
        }
    }
}

/// Calendar/time-card corner treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateTimeCorners {
    #[default]
    Rounded,
    Left,
    Right,
    Straight,
}
impl DateTimeCorners {
    fn class(self) -> &'static str {
        match self {
            Self::Rounded => "rounded",
            Self::Left => "left",
            Self::Right => "right",
            Self::Straight => "straight",
        }
    }
}

/// Structural card used by the calendar and time picker.
///
/// `has_footer` defaults to whether footer content was supplied. Set it
/// explicitly to reserve an empty footer or suppress supplied footer content.
/// `onmounted` exposes the actual card div for native Dioxus composition.
#[component]
pub fn DateTimeCard(
    children: Element,
    header: Option<Element>,
    footer: Option<Element>,
    #[props(default)] has_footer: Option<bool>,
    #[props(default)] embedded: bool,
    #[props(default)] hide_header: bool,
    #[props(default)] time_picker_appearance: bool,
    #[props(default)] no_padding: bool,
    #[props(default)] corners: DateTimeCorners,
    id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] onmounted: EventHandler<MountedEvent>,
) -> Element {
    let show_footer = has_footer.unwrap_or(footer.is_some());
    rsx! { div { id, onmounted, class: "ix-date-time-card ix-date-corners-{corners.class()} {class}",
        class: if embedded { "ix-date-embedded" }, class: if no_padding { "ix-date-no-padding" },
        class: if time_picker_appearance { "ix-date-time-picker-appearance" },
        if !hide_header { if let Some(header) = header { div { class: "ix-date-card-header", {header} } } }
        div { class: "ix-date-card-content", {children} }
        if show_footer { div { class: "ix-date-card-footer", {footer} } }
    } }
}

fn calendar_rows(month: IxDate, week_start: u8) -> Vec<Vec<Option<IxDate>>> {
    let offset = (month.month_start().weekday() + 7 - week_start % 7) % 7;
    let days = IxDate::days_in_month(month.year, month.month);
    let count = (usize::from(offset) + usize::from(days)).div_ceil(7) * 7;
    (0..count)
        .map(|index| {
            let day = index as i16 - i16::from(offset) + 1;
            if day >= 1 && day <= i16::from(days) {
                IxDate::new(month.year, month.month, day as u8)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .chunks(7)
        .map(<[_]>::to_vec)
        .collect()
}
fn month_selectable(month: IxDate, min: Option<IxDate>, max: Option<IxDate>) -> bool {
    let end = IxDate::new(
        month.year,
        month.month,
        IxDate::days_in_month(month.year, month.month),
    )
    .unwrap();
    max.is_none_or(|max| month.month_start() <= max) && min.is_none_or(|min| end >= min)
}

fn calendar_page_target(date: IxDate, forward: bool, by_year: bool) -> Option<IxDate> {
    let months = if by_year { 12 } else { 1 };
    date.add_months(if forward { months } else { -months })
}

fn focused_date_in_month(
    month: IxDate,
    previous: IxDate,
    min: Option<IxDate>,
    max: Option<IxDate>,
) -> Option<IxDate> {
    let first = month.month_start();
    let last = IxDate::new(
        month.year,
        month.month,
        IxDate::days_in_month(month.year, month.month),
    )?;
    let first = min.map_or(first, |min| first.max(min));
    let last = max.map_or(last, |max| last.min(max));
    if first > last {
        return None;
    }
    let candidate = IxDate::new(
        month.year,
        month.month,
        previous
            .day
            .min(IxDate::days_in_month(month.year, month.month)),
    )?;
    Some(candidate.clamp(first, last))
}

/// iX calendar with date-range selection, bounds, ISO week numbers, and roving
/// keyboard focus. Arrow keys move days; Page Up/Down move months (years with
/// Shift); Home/End move to week boundaries. `today` comes from the host's date source.
#[component]
pub fn DatePicker(
    today: IxDate,
    from: Option<IxDate>,
    to: Option<IxDate>,
    min_date: Option<IxDate>,
    max_date: Option<IxDate>,
    #[props(default)] single_selection: bool,
    #[props(default)] disabled: bool,
    #[props(default)] auto_focus: bool,
    #[props(default)] week_start_index: u8,
    #[props(default)] show_week_numbers: bool,
    #[props(default)] embedded: bool,
    #[props(default)] corners: DateTimeCorners,
    #[props(default)] labels: CalendarLabels,
    ondatechange: EventHandler<DateSelection>,
    onselect: Option<EventHandler<DateSelection>>,
) -> Element {
    let initial = from.unwrap_or(today);
    let initial = min_date.map_or(initial, |min| initial.max(min));
    let initial = max_date.map_or(initial, |max| initial.min(max));
    let mut month = use_signal(|| initial.month_start());
    let mut focused = use_signal(|| initial);
    let mut focus_requested = use_signal(|| auto_focus);
    let day_refs = use_hook(|| Rc::new(RefCell::new(BTreeMap::<IxDate, Rc<MountedData>>::new())));
    // Registrations belong to the currently rendered month. Every calendar
    // row contains only that month's days; retire historical native handles
    // while leaving normal mounted registration and focus unchanged.
    let mounted_month = *month.peek();
    day_refs
        .borrow_mut()
        .retain(|date, _| date.month_start() == mounted_month);
    let initial_focus_pending = use_hook(|| Rc::new(Cell::new(auto_focus)));
    let focus_initial_visible_day = {
        let day_refs = day_refs.clone();
        use_callback(move |event: VisibleEvent| {
            // Native popovers can still be hidden when mounted handlers run.
            // Retry the initial intent once after the calendar is visible.
            if !disabled
                && event.is_intersecting().is_ok_and(|visible| visible)
                && initial_focus_pending.get()
            {
                let element = day_refs.borrow().get(&*focused.peek()).cloned();
                if let Some(element) = element {
                    initial_focus_pending.set(false);
                    spawn(async move {
                        let _ = element.set_focus(true).await;
                    });
                }
            }
        })
    };
    let register = {
        let day_refs = day_refs.clone();
        use_callback(move |(date, element): (IxDate, Rc<MountedData>)| {
            day_refs.borrow_mut().insert(date, element.clone());
            if *focus_requested.peek() && *focused.peek() == date {
                focus_requested.set(false);
                spawn(async move {
                    let _ = element.set_focus(true).await;
                });
            }
        })
    };
    let focus_day = use_callback(move |date: IxDate| {
        let date = if let Some(min) = min_date {
            date.max(min)
        } else {
            date
        };
        let date = if let Some(max) = max_date {
            date.min(max)
        } else {
            date
        };
        let same_month = *month.peek() == date.month_start();
        focused.set(date);
        month.set(date.month_start());
        focus_requested.set(true);
        let element = same_month
            .then(|| day_refs.borrow().get(&date).cloned())
            .flatten();
        if let Some(element) = element {
            focus_requested.set(false);
            spawn(async move {
                let _ = element.set_focus(true).await;
            });
        }
    });
    let change_month = use_callback(move |target: IxDate| {
        let previous = *focused.peek();
        if let Some(next) = focused_date_in_month(target, previous, min_date, max_date) {
            // Header navigation keeps its own focus while preserving one date-grid
            // tab stop in the newly displayed month.
            focus_requested.set(false);
            focused.set(next);
            month.set(target.month_start());
        }
    });
    let previous_dates = use_hook(|| Rc::new(Cell::new((from, to, min_date, max_date))));
    use_effect(use_reactive!(|from, to, min_date, max_date| {
        let previous = previous_dates.replace((from, to, min_date, max_date));
        // Only newly supplied endpoints navigate the view. Clearing selection
        // preserves browsing; simultaneous endpoint updates retain from priority.
        let supplied = (previous.0 != from)
            .then_some(from)
            .flatten()
            .or_else(|| (previous.1 != to).then_some(to).flatten());
        let bounds_changed = previous.2 != min_date || previous.3 != max_date;
        let Some(anchor) = supplied.or_else(|| bounds_changed.then_some(from.unwrap_or(today)))
        else {
            return;
        };
        let anchor = min_date.map_or(anchor, |min| anchor.max(min));
        let anchor = max_date.map_or(anchor, |max| anchor.min(max));
        // Owner synchronization changes the roving date, never native focus.
        // Unchanged dates leave ordinary keyboard/header focus intents intact.
        focus_requested.set(false);
        month.set(anchor.month_start());
        focused.set(anchor);
    }));
    let selection = DateSelection { from, to };
    let visible = month();
    let previous = visible.add_months(-1);
    let next = visible.add_months(1);
    let week_start = week_start_index % 7;
    let rows = calendar_rows(visible, week_start);
    let month_names = labels.months.clone();
    let header = rsx! { div { class: "ix-calendar-header",
        crate::IconButton { icon: "chevron-left-small", label: labels.previous_month.clone(), variant: crate::ButtonVariant::Tertiary,
            disabled: disabled || previous.is_none_or(|date| !month_selectable(date, min_date, max_date)),
            onclick: move |_| { if let Some(previous) = previous { change_month.call(previous); } }
        }
        select { class: "ix-calendar-month", aria_label: labels.select_month.clone(), disabled,
            value: visible.month.to_string(),
            onchange: move |event| { if let Ok(value) = event.value().parse::<u8>() { if let Some(date) = IxDate::new(visible.year, value, 1) { change_month.call(date); } } },
            for (index, name) in month_names.into_iter().enumerate() { option { value: (index + 1).to_string(), selected: index + 1 == usize::from(visible.month), disabled: !month_selectable(IxDate::new(visible.year, index as u8 + 1, 1).unwrap(), min_date, max_date), "{name}" } }
        }
        input { class: "ix-calendar-year", r#type: "number", value: visible.year.to_string(), min: min_date.map_or(1, |date| date.year), max: max_date.map_or(9999, |date| date.year),
            aria_label: labels.select_year.clone(), disabled,
            onchange: move |event| { if let Ok(value) = event.value().parse::<u16>() { if let Some(date) = IxDate::new(value, visible.month, 1) { change_month.call(date); } } },
        }
        crate::IconButton { icon: "chevron-right-small", label: labels.next_month.clone(), variant: crate::ButtonVariant::Tertiary,
            disabled: disabled || next.is_none_or(|date| !month_selectable(date, min_date, max_date)),
            onclick: move |_| { if let Some(next) = next { change_month.call(next); } }
        }
    } };
    let footer = if !single_selection && !embedded {
        Some(rsx! { crate::Button {
        disabled: disabled || from.is_none_or(|date| !date.within(min_date, max_date)) || to.is_none_or(|date| !date.within(min_date, max_date)) || from > to,
            onclick: move |_| { if let Some(handler) = onselect { handler.call(selection); } }, "{labels.done}"
        } })
    } else {
        None
    };
    rsx! { DateTimeCard { header, footer, corners, embedded,
        div { class: "ix-calendar-grid", role: "grid", aria_label: format!("{} {}", labels.months[usize::from(visible.month - 1)], visible.year),
            onvisible: focus_initial_visible_day,
            div { class: "ix-calendar-row", role: "row",
                if show_week_numbers { span { class: "ix-calendar-weekday", role: "columnheader", aria_label: "Week number", "Wk" } }
                for index in 0..7u8 { span { class: "ix-calendar-weekday", role: "columnheader", title: labels.weekdays[usize::from((index + week_start) % 7)].clone(), "{labels.weekdays[usize::from((index + week_start) % 7)].chars().take(3).collect::<String>()}" } }
            }
            for row in rows {
                div { class: "ix-calendar-row", role: "row",
                    if show_week_numbers { span { class: "ix-calendar-week-number", role: "rowheader", "{row.iter().flatten().next().map(|date| date.iso_week().1).unwrap_or(0)}" } }
                    for day in row {
                        if let Some(date) = day {
                            button { key: "{date}", class: "ix-calendar-day", role: "gridcell", r#type: "button",
                                class: if date == today { "ix-calendar-today" }, class: if selection.is_endpoint(date) { "ix-calendar-selected" }, class: if selection.contains_interior(date) { "ix-calendar-in-range" },
                                disabled: disabled || !date.within(min_date, max_date), aria_selected: selection.is_endpoint(date), aria_current: if date == today { Some("date") } else { None },
                                aria_label: format!("{} {} {}", date.day, labels.months[usize::from(date.month - 1)], date.year),
                                tabindex: if focused() == date { 0 } else { -1 },
                                onmounted: move |event| register.call((date, event.data())),
                                onfocus: move |_| focused.set(date),
                                onclick: move |_| { let next = selection.select(date, single_selection); ondatechange.call(next); if single_selection { if let Some(handler) = onselect { handler.call(next); } } },
                                onkeydown: move |event| {
                                    let next = match event.key() {
                                        Key::ArrowLeft => date.add_days(-1), Key::ArrowRight => date.add_days(1),
                                        Key::ArrowUp => date.add_days(-7), Key::ArrowDown => date.add_days(7),
                                        Key::Home => date.add_days(-i64::from((date.weekday() + 7 - week_start) % 7)),
                                        Key::End => date.add_days(6 - i64::from((date.weekday() + 7 - week_start) % 7)),
                                        Key::PageUp => calendar_page_target(date, false, event.modifiers().shift()),
                                        Key::PageDown => calendar_page_target(date, true, event.modifiers().shift()), _ => None,
                                    };
                                    if let Some(next) = next { event.prevent_default(); focus_day.call(next); }
                                },
                                "{date.day}"
                            }
                        } else { span { class: "ix-calendar-empty", role: "gridcell" } }
                    }
                }
            }
        }
    } }
}

/// A validated time of day with millisecond precision, independent of timezone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IxTime {
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
}
impl IxTime {
    /// Construct a time of day, rejecting overflow instead of silently wrapping.
    pub const fn new(hour: u8, minute: u8, second: u8, millisecond: u16) -> Option<Self> {
        if hour < 24 && minute < 60 && second < 60 && millisecond < 1000 {
            Some(Self {
                hour,
                minute,
                second,
                millisecond,
            })
        } else {
            None
        }
    }
    pub const fn hour(self) -> u8 {
        self.hour
    }
    pub const fn minute(self) -> u8 {
        self.minute
    }
    pub const fn second(self) -> u8 {
        self.second
    }
    pub const fn millisecond(self) -> u16 {
        self.millisecond
    }
    /// Milliseconds since midnight.
    pub const fn milliseconds(self) -> u32 {
        ((self.hour as u32 * 60 + self.minute as u32) * 60 + self.second as u32) * 1000
            + self.millisecond as u32
    }
    fn within(self, min: Option<Self>, max: Option<Self>) -> bool {
        min.is_none_or(|min| self >= min) && max.is_none_or(|max| self <= max)
    }
    fn with_unit(self, unit: ClockUnit, value: u16) -> Self {
        match unit {
            ClockUnit::Hour => Self {
                hour: value as u8,
                ..self
            },
            ClockUnit::Minute => Self {
                minute: value as u8,
                ..self
            },
            ClockUnit::Second => Self {
                second: value as u8,
                ..self
            },
            ClockUnit::Millisecond => Self {
                millisecond: value,
                ..self
            },
        }
    }
    fn unit(self, unit: ClockUnit) -> u16 {
        match unit {
            ClockUnit::Hour => self.hour.into(),
            ClockUnit::Minute => self.minute.into(),
            ClockUnit::Second => self.second.into(),
            ClockUnit::Millisecond => self.millisecond,
        }
    }
}
impl fmt::Display for IxTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

// An inverted wall-clock window is ignored, matching iX's constraint parser;
// it does not describe an overnight range.
fn normalized_time_bounds(
    min: Option<IxTime>,
    max: Option<IxTime>,
) -> (Option<IxTime>, Option<IxTime>) {
    if min.zip(max).is_some_and(|(min, max)| min > max) {
        (None, None)
    } else {
        (min, max)
    }
}

// With a date window, its first and last dates own the corresponding clock
// bound. Interior dates retain the entire day. Without date bounds, the clock
// window applies every day. A missing selected date has no boundary to match.
fn date_scoped_time_bounds(
    date: Option<IxDate>,
    (min_date, max_date): (Option<IxDate>, Option<IxDate>),
    (min_time, max_time): (Option<IxTime>, Option<IxTime>),
) -> (Option<IxTime>, Option<IxTime>) {
    if min_date.is_none() && max_date.is_none() {
        return (min_time, max_time);
    }
    (
        min_time.filter(|_| date.is_some() && date == min_date),
        max_time.filter(|_| date.is_some() && date == max_date),
    )
}

/// Clock precision and 12/24-hour representation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeFormat {
    Hm,
    #[default]
    Hms,
    HmsMillis,
    /// Twelve-hour clock including milliseconds, without a meridiem suffix.
    HmsMillis12,
    Hm12,
    Hms12,
}
impl TimeFormat {
    pub const fn pattern(self) -> &'static str {
        match self {
            Self::Hm => "HH:MM",
            Self::Hms => "HH:MM:SS",
            Self::HmsMillis => "HH:MM:SS.mmm",
            Self::HmsMillis12 => "hh:mm:ss.mmm",
            Self::Hm12 => "hh:mm AM",
            Self::Hms12 => "hh:mm:ss AM",
        }
    }
    pub const fn is_twelve_hour(self) -> bool {
        matches!(self, Self::Hm12 | Self::Hms12 | Self::HmsMillis12)
    }
    fn has_meridiem(self) -> bool {
        matches!(self, Self::Hm12 | Self::Hms12)
    }
    fn has_milliseconds(self) -> bool {
        matches!(self, Self::HmsMillis | Self::HmsMillis12)
    }
    fn has_seconds(self) -> bool {
        !matches!(self, Self::Hm | Self::Hm12)
    }
    /// Format a validated time.
    pub fn format(self, time: IxTime) -> String {
        let hour = if self.is_twelve_hour() {
            if time.hour.is_multiple_of(12) {
                12
            } else {
                time.hour % 12
            }
        } else {
            time.hour
        };
        let mut value = format!("{hour:02}:{:02}", time.minute);
        if self.has_seconds() {
            value.push_str(&format!(":{:02}", time.second));
        }
        if self.has_milliseconds() {
            value.push_str(&format!(".{:03}", time.millisecond));
        }
        if self.has_meridiem() {
            value.push_str(if time.hour < 12 { " AM" } else { " PM" });
        }
        value
    }
    /// Parse this clock format, including midnight/noon conversion in 12-hour mode.
    pub fn parse(self, value: &str) -> Option<IxTime> {
        let value = value.trim();
        let (clock, reference) = if self.has_meridiem() {
            let (clock, reference) = value.rsplit_once(' ')?;
            (clock, Some(reference))
        } else {
            (value, None)
        };
        let (clock, millis) = if self.has_milliseconds() {
            let (clock, millis) = clock.split_once('.')?;
            if millis.len() != 3 || !millis.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            (clock, millis.parse().ok()?)
        } else {
            (clock, 0)
        };
        let parts: Vec<_> = clock.split(':').collect();
        if parts.len() != if self.has_seconds() { 3 } else { 2 }
            || !parts
                .iter()
                .all(|part| part.len() == 2 && part.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return None;
        }
        let mut hour: u8 = parts[0].parse().ok()?;
        if self.is_twelve_hour() && !(1..=12).contains(&hour) {
            return None;
        }
        if let Some(reference) = reference {
            if !(1..=12).contains(&hour) {
                return None;
            }
            hour = match reference {
                "AM" | "am" => hour % 12,
                "PM" | "pm" => hour % 12 + 12,
                _ => return None,
            };
        }
        IxTime::new(
            hour,
            parts[1].parse().ok()?,
            if self.has_seconds() {
                parts[2].parse().ok()?
            } else {
                0
            },
            millis,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ClockUnit {
    Hour,
    Minute,
    Second,
    Millisecond,
}
impl ClockUnit {
    fn name(self) -> &'static str {
        match self {
            Self::Hour => "Hours",
            Self::Minute => "Minutes",
            Self::Second => "Seconds",
            Self::Millisecond => "Milliseconds",
        }
    }
    fn end(self) -> u16 {
        match self {
            Self::Hour => 24,
            Self::Minute | Self::Second => 60,
            Self::Millisecond => 1000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClockKeyAction {
    Focus(u16),
    Commit,
    Unhandled,
}

fn clock_unit_values(unit: ClockUnit, interval: u16, format: TimeFormat, time: IxTime) -> Vec<u16> {
    if unit == ClockUnit::Hour && format.is_twelve_hour() {
        (1..=12)
            .step_by(usize::from(interval.max(1)))
            .map(|hour| hour % 12 + u16::from(time.hour / 12) * 12)
            .collect()
    } else {
        (0..unit.end())
            .step_by(usize::from(interval.max(1)))
            .collect()
    }
}

fn clock_key_action(values: &[u16], current: u16, key: &Key) -> ClockKeyAction {
    if values.is_empty() {
        return ClockKeyAction::Unhandled;
    }
    let index = values
        .iter()
        .position(|value| *value == current)
        .unwrap_or(0);
    match key {
        Key::ArrowDown => ClockKeyAction::Focus(values[(index + 1) % values.len()]),
        Key::ArrowUp => ClockKeyAction::Focus(values[(index + values.len() - 1) % values.len()]),
        Key::Home => ClockKeyAction::Focus(values[0]),
        Key::End => ClockKeyAction::Focus(values[values.len() - 1]),
        Key::Enter => ClockKeyAction::Commit,
        Key::Character(key) if key == " " => ClockKeyAction::Commit,
        _ => ClockKeyAction::Unhandled,
    }
}

fn select_clock_unit(
    base: IxTime,
    unit: ClockUnit,
    value: u16,
    min: Option<IxTime>,
    max: Option<IxTime>,
) -> Option<IxTime> {
    let (min, max) = normalized_time_bounds(min, max);
    if value >= unit.end() {
        return None;
    }
    let candidate = base.with_unit(unit, value);
    let (start, end) = match unit {
        ClockUnit::Hour => (
            IxTime {
                minute: 0,
                second: 0,
                millisecond: 0,
                ..candidate
            },
            IxTime {
                minute: 59,
                second: 59,
                millisecond: 999,
                ..candidate
            },
        ),
        ClockUnit::Minute => (
            IxTime {
                second: 0,
                millisecond: 0,
                ..candidate
            },
            IxTime {
                second: 59,
                millisecond: 999,
                ..candidate
            },
        ),
        ClockUnit::Second => (
            IxTime {
                millisecond: 0,
                ..candidate
            },
            IxTime {
                millisecond: 999,
                ..candidate
            },
        ),
        ClockUnit::Millisecond => (candidate, candidate),
    };
    if max.is_some_and(|max| start > max) || min.is_some_and(|min| end < min) {
        return None;
    }
    let candidate = min.map_or(candidate, |min| candidate.max(min));
    let candidate = max.map_or(candidate, |max| candidate.min(max));
    candidate.within(min, max).then_some(candidate)
}

/// Translatable time-picker text.
#[derive(Clone, Debug, PartialEq)]
pub struct TimeLabels {
    pub header: String,
    pub hour: String,
    pub minute: String,
    pub second: String,
    pub millisecond: String,
    pub am: String,
    pub pm: String,
    pub confirm: String,
}
impl Default for TimeLabels {
    fn default() -> Self {
        Self {
            header: "Time".into(),
            hour: "hr".into(),
            minute: "min".into(),
            second: "sec".into(),
            millisecond: "ms".into(),
            am: "AM".into(),
            pm: "PM".into(),
            confirm: "Confirm".into(),
        }
    }
}

fn clock_list_scroll_offset(
    container_top: f64,
    container_height: f64,
    option_top: f64,
    option_height: f64,
    current: f64,
) -> f64 {
    let option_bottom = option_top + option_height;
    let visible_bottom = container_top + container_height;
    let adjustment = if option_top < container_top {
        option_top - container_top
    } else if option_bottom > visible_bottom {
        option_bottom - visible_bottom
    } else {
        0.0
    };
    (current + adjustment).max(0.0)
}

async fn reveal_clock_option(container: Rc<MountedData>, option: Rc<MountedData>) -> bool {
    let Ok(container_rect) = container.get_client_rect().await else {
        return false;
    };
    let Ok(option_rect) = option.get_client_rect().await else {
        return false;
    };
    let Ok(offset) = container.get_scroll_offset().await else {
        return false;
    };
    let top = clock_list_scroll_offset(
        container_rect.origin.y,
        container_rect.size.height,
        option_rect.origin.y,
        option_rect.size.height,
        offset.y,
    );
    if (top - offset.y).abs() > 0.5 {
        // Unlike scrollIntoView, scrolling the list itself never moves the page
        // or any other ancestor when a standalone picker mounts off screen.
        container
            .scroll(
                dioxus::html::geometry::PixelsVector2D::new(offset.x, top),
                ScrollBehavior::Instant,
            )
            .await
            .is_ok()
    } else {
        true
    }
}

/// Scrollable iX clock columns with intervals, constraints, AM/PM selection, and
/// keyboard navigation. Arrows move provisional focus; Enter/Space or a click
/// commits it. Unit selection clamps subordinate units to valid bounds.
#[component]
pub fn TimePicker(
    #[props(default)] time: IxTime,
    #[props(default)] format: TimeFormat,
    min_time: Option<IxTime>,
    max_time: Option<IxTime>,
    #[props(default = 1)] hour_interval: u8,
    #[props(default = 1)] minute_interval: u8,
    #[props(default = 1)] second_interval: u8,
    #[props(default = 100)] millisecond_interval: u16,
    #[props(default)] embedded: bool,
    /// Override confirmation presence independently of embedded appearance.
    /// When omitted, standalone clocks show confirmation and embedded clocks omit it.
    has_footer: Option<bool>,
    #[props(default)] hide_header: bool,
    #[props(default)] disabled: bool,
    #[props(default)] auto_focus: bool,
    #[props(default)] corners: DateTimeCorners,
    #[props(default)] labels: TimeLabels,
    ontimechange: EventHandler<IxTime>,
    onselect: Option<EventHandler<IxTime>>,
) -> Element {
    let (min_time, max_time) = normalized_time_bounds(min_time, max_time);
    let mut focused_option = use_signal(|| None::<(ClockUnit, u16)>);
    let mut references = use_signal(BTreeMap::<(ClockUnit, u16), Rc<MountedData>>::new);
    let mut containers = use_signal(BTreeMap::<ClockUnit, Rc<MountedData>>::new);
    let register_container = use_callback(move |(unit, element): (ClockUnit, Rc<MountedData>)| {
        containers.write().insert(unit, element);
    });
    let register = use_callback(move |(key, element): ((ClockUnit, u16), Rc<MountedData>)| {
        references.write().insert(key, element);
    });
    let focus_option = use_callback(move |key: (ClockUnit, u16)| {
        focused_option.set(Some(key));
        let element = references.peek().get(&key).cloned();
        if let Some(element) = element {
            spawn(async move {
                let _ = element.set_focus(true).await;
            });
        }
    });
    let mut visible_columns = use_signal(BTreeSet::<ClockUnit>::new);
    type CompletedReveal = (u16, Rc<MountedData>, Rc<MountedData>);
    let completed_reveals =
        use_hook(|| Rc::new(RefCell::new(BTreeMap::<ClockUnit, CompletedReveal>::new())));
    let on_column_visible = {
        let completed_reveals = completed_reveals.clone();
        use_callback(move |(unit, visible): (ClockUnit, bool)| {
            if !visible {
                completed_reveals.borrow_mut().remove(&unit);
            }
            if visible_columns.peek().contains(&unit) != visible {
                if visible {
                    visible_columns.write().insert(unit);
                } else {
                    visible_columns.write().remove(&unit);
                }
            }
        })
    };
    let autofocus_pending = use_hook(|| Rc::new(Cell::new(auto_focus)));
    let on_option_focus = {
        let autofocus_pending = autofocus_pending.clone();
        use_callback(move |key: (ClockUnit, u16)| {
            // Native user focus supersedes optional initial focus, including
            // when the list has not become visible yet.
            autofocus_pending.set(false);
            focused_option.set(Some(key));
        })
    };
    let mut columns = vec![
        (
            ClockUnit::Hour,
            u16::from(hour_interval).clamp(1, 23),
            labels.hour.clone(),
        ),
        (
            ClockUnit::Minute,
            u16::from(minute_interval).clamp(1, 59),
            labels.minute.clone(),
        ),
    ];
    if format.has_seconds() {
        columns.push((
            ClockUnit::Second,
            u16::from(second_interval).clamp(1, 59),
            labels.second.clone(),
        ));
    }
    if format.has_milliseconds() {
        columns.push((
            ClockUnit::Millisecond,
            millisecond_interval.clamp(1, 999),
            labels.millisecond.clone(),
        ));
    }
    // Mounted ownership follows rendered options, including disabled values.
    // Period/interval/precision changes retire only keys and lists absent from
    // this view; valid provisional focus and current native handles stay owned.
    let rendered_units: BTreeSet<_> = columns.iter().map(|(unit, _, _)| *unit).collect();
    let rendered_keys: BTreeSet<_> = columns
        .iter()
        .flat_map(|(unit, interval, _)| {
            clock_unit_values(*unit, *interval, format, time)
                .into_iter()
                .map(|value| (*unit, value))
        })
        .collect();
    if references
        .peek()
        .keys()
        .any(|key| !rendered_keys.contains(key))
    {
        references
            .write()
            .retain(|key, _| rendered_keys.contains(key));
    }
    if containers
        .peek()
        .keys()
        .any(|unit| !rendered_units.contains(unit))
    {
        containers
            .write()
            .retain(|unit, _| rendered_units.contains(unit));
    }
    if visible_columns
        .peek()
        .iter()
        .any(|unit| !rendered_units.contains(unit))
    {
        visible_columns
            .write()
            .retain(|unit| rendered_units.contains(unit));
    }
    completed_reveals
        .borrow_mut()
        .retain(|unit, (value, _, _)| rendered_keys.contains(&(*unit, *value)));
    if focused_option
        .peek()
        .is_some_and(|key| !rendered_keys.contains(&key))
    {
        focused_option.set(None);
    }
    let columns: Vec<_> = columns
        .into_iter()
        .map(|(unit, interval, label)| {
            let selectable: Vec<_> = clock_unit_values(unit, interval, format, time)
                .into_iter()
                .filter(|value| select_clock_unit(time, unit, *value, min_time, max_time).is_some())
                .collect();
            let provisional = focused_option()
                .filter(|(focused_unit, value)| *focused_unit == unit && selectable.contains(value))
                .map(|(_, value)| value);
            let accepted = selectable
                .into_iter()
                .min_by_key(|value| value.abs_diff(time.unit(unit)));
            let roving = provisional.or(accepted);
            (unit, interval, label, roving, accepted)
        })
        .collect();
    let reveal_values: Vec<_> = columns
        .iter()
        .map(|(unit, _, _, _, accepted)| (*unit, *accepted))
        .collect();
    // One Dioxus-owned resource cancels replaced/retired work. Read mounted
    // handles reactively so newly rendered half-day/interval options retry with
    // their actual node; hidden units wait for their latest visibility event.
    let _reveal = use_resource(use_reactive!(|reveal_values, disabled| {
        let visible = visible_columns.read();
        let focused = *focused_option.read();
        let options = references.read();
        let lists = containers.read();
        let jobs: Vec<_> =
            reveal_values
                .into_iter()
                .filter_map(|(unit, value)| {
                    if focused.is_some_and(|(active, _)| active == unit) {
                        // The user can scroll this unit independently. On blur,
                        // reveal the accepted value even if it matches an earlier
                        // completed selection from before that interaction.
                        completed_reveals.borrow_mut().remove(&unit);
                        return None;
                    }
                    if !visible.contains(&unit) {
                        return None;
                    }
                    let value = value?;
                    let container = lists.get(&unit)?;
                    let option = options.get(&(unit, value))?;
                    if completed_reveals.borrow().get(&unit).is_some_and(
                        |(previous, list, node)| {
                            *previous == value
                                && Rc::ptr_eq(list, container)
                                && Rc::ptr_eq(node, option)
                        },
                    ) {
                        return None;
                    }
                    Some((unit, value, container.clone(), option.clone()))
                })
                .collect();
        let autofocus_pending = autofocus_pending.clone();
        let completed_reveals = completed_reveals.clone();
        async move {
            for (unit, value, container, option) in jobs {
                if !reveal_clock_option(container.clone(), option.clone()).await {
                    continue;
                }
                // Focus changes must not undo manual scrolling in another unit.
                // Only completed work is memoized; cancellation or failed native
                // geometry leaves this value available for a later retry.
                completed_reveals
                    .borrow_mut()
                    .insert(unit, (value, container, option.clone()));
                if unit == ClockUnit::Hour
                    && autofocus_pending.replace(false)
                    && !disabled
                    && focused.is_none()
                {
                    let _ = option.set_focus(true).await;
                }
            }
        }
    }));
    let period = format.is_twelve_hour().then(|| rsx! { div { class: "ix-clock-meridiem",
            crate::ToggleButton { aria_label: labels.am.clone(), pressed: time.hour < 12, disabled: disabled || !IxTime { hour: time.hour % 12, ..time }.within(min_time, max_time),
                onpressedchange: move |_| ontimechange.call(IxTime { hour: time.hour % 12, ..time }), "{labels.am}" }
            crate::ToggleButton { aria_label: labels.pm.clone(), pressed: time.hour >= 12, disabled: disabled || !IxTime { hour: time.hour % 12 + 12, ..time }.within(min_time, max_time),
                onpressedchange: move |_| ontimechange.call(IxTime { hour: time.hour % 12 + 12, ..time }), "{labels.pm}" }
    } });
    let header_period = if hide_header { None } else { period.clone() };
    let header = rsx! { div { class: "ix-clock-header", "{labels.header}" {header_period} } };
    let footer = if has_footer.unwrap_or(!embedded) {
        Some(
            rsx! { crate::Button { disabled: disabled || !time.within(min_time, max_time), onclick: move |_| { if let Some(handler) = onselect { handler.call(time); } }, "{labels.confirm}" } },
        )
    } else {
        None
    };
    rsx! { DateTimeCard { header, footer, corners, embedded, hide_header,
        if hide_header { if let Some(period) = period { div { class: "ix-clock-period-body", {period} } } }
        div { class: "ix-clock-columns",
            for (unit, interval, label, roving, _) in columns {
                div { class: "ix-clock-column",
                    div { class: "ix-clock-column-heading", "{label}" }
                    div { class: "ix-clock-options", role: "listbox", aria_label: unit.name(), aria_disabled: disabled,
                        onmounted: move |event| register_container.call((unit, event.data())),
                        onvisible: move |event: VisibleEvent| {
                            if let Ok(visible) = event.is_intersecting() {
                                on_column_visible.call((unit, visible));
                            }
                        },
                        for value in clock_unit_values(unit, interval, format, time) {
                            button { key: "{value}", class: "ix-clock-option", class: if time.unit(unit) == value { "ix-clock-selected" },
                                r#type: "button", role: "option", aria_selected: time.unit(unit) == value,
                                disabled: disabled || select_clock_unit(time, unit, value, min_time, max_time).is_none(),
                                tabindex: if roving == Some(value) { 0 } else { -1 },
                                onmounted: move |event| register.call(((unit, value), event.data())),
                                onfocus: move |_| on_option_focus.call((unit, value)),
                                onblur: move |_| {
                                    let current = *focused_option.peek();
                                    if current == Some((unit, value)) { focused_option.set(None); }
                                },
                                onclick: move |_| { if let Some(next) = select_clock_unit(time, unit, value, min_time, max_time) { ontimechange.call(next); } },
                                onkeydown: move |event| {
                                    let values: Vec<_> = clock_unit_values(unit, interval, format, time).into_iter().filter(|value| select_clock_unit(time, unit, *value, min_time, max_time).is_some()).collect();
                                    match clock_key_action(&values, value, &event.key()) {
                                        ClockKeyAction::Focus(next) => { event.prevent_default(); focus_option.call((unit, next)); },
                                        ClockKeyAction::Commit => {
                                            event.prevent_default();
                                            if let Some(next) = select_clock_unit(time, unit, value, min_time, max_time) { ontimechange.call(next); }
                                        },
                                        ClockKeyAction::Unhandled => {}
                                    }
                                },
                                if unit == ClockUnit::Hour && format.is_twelve_hour() { {format!("{:02}", if value % 12 == 0 { 12 } else { value % 12 })} }
                                else if unit == ClockUnit::Millisecond { "{value:03}" }
                                else { "{value:02}" }
                            }
                        }
                    }
                }
            }
        }
    } }
}

/// A confirmed calendar selection plus its time-of-day value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTimeSelection {
    pub dates: DateSelection,
    pub time: IxTime,
}

/// Combined calendar and clock with a single confirmation action.
/// In single-selection mode, time bounds apply every day without date bounds,
/// or only on their matching boundary dates when date bounds are present.
/// Range mode ignores time bounds, as in iX.
#[component]
pub fn DateTimePicker(
    today: IxDate,
    from: Option<IxDate>,
    to: Option<IxDate>,
    min_date: Option<IxDate>,
    max_date: Option<IxDate>,
    #[props(default)] time: IxTime,
    #[props(default)] single_selection: bool,
    #[props(default)] time_format: TimeFormat,
    min_time: Option<IxTime>,
    max_time: Option<IxTime>,
    #[props(default)] week_start_index: u8,
    #[props(default)] show_week_numbers: bool,
    #[props(default)] disabled: bool,
    #[props(default)] auto_focus: bool,
    #[props(default)] calendar_labels: CalendarLabels,
    #[props(default)] time_labels: TimeLabels,
    #[props(default = "Done".to_string())] done_label: String,
    ondatechange: EventHandler<DateSelection>,
    ontimechange: EventHandler<IxTime>,
    onselect: EventHandler<DateTimeSelection>,
) -> Element {
    let selection = DateSelection { from, to };
    let (min_time, max_time) = if single_selection {
        let (min, max) = date_scoped_time_bounds(from, (min_date, max_date), (min_time, max_time));
        normalized_time_bounds(min, max)
    } else {
        (None, None)
    };
    rsx! { div { class: "ix-datetime-picker",
        div { class: "ix-datetime-pickers",
            DatePicker { today, from, to, min_date, max_date, single_selection, disabled, auto_focus, week_start_index, show_week_numbers,
                embedded: true, labels: calendar_labels, ondatechange }
            TimePicker { time, format: time_format, min_time, max_time, embedded: true, disabled, labels: time_labels, ontimechange }
        }
        div { class: "ix-date-card-footer",
            crate::Button { disabled: disabled || from.is_none_or(|date| !date.within(min_date, max_date)) || (!single_selection && (to.is_none_or(|date| !date.within(min_date, max_date)) || from > to)) || !time.within(min_time, max_time),
                onclick: move |_| onselect.call(DateTimeSelection { dates: selection, time }), "{done_label}" }
        }
    } }
}

/// Result of parsing and checking an input against its required/range constraints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateInputValidity {
    pub value_missing: bool,
    pub bad_input: bool,
    pub range_underflow: bool,
    pub range_overflow: bool,
}
impl DateInputValidity {
    /// True when every constraint passes.
    pub const fn valid(self) -> bool {
        !self.value_missing && !self.bad_input && !self.range_underflow && !self.range_overflow
    }
    fn message(self, subject: &str) -> Option<String> {
        if self.value_missing {
            Some(format!("{subject} is required"))
        } else if self.bad_input {
            Some(format!("{subject} is not valid"))
        } else if self.range_underflow {
            Some(format!("{subject} is earlier than the minimum"))
        } else if self.range_overflow {
            Some(format!("{subject} is later than the maximum"))
        } else {
            None
        }
    }
}

/// Validate typed date text without browser or locale dependencies.
pub fn validate_date_input(
    value: &str,
    format: DateFormat,
    required: bool,
    min: Option<IxDate>,
    max: Option<IxDate>,
) -> DateInputValidity {
    date_input_validity(value, format.parse(value), required, min, max)
}
fn date_input_validity(
    value: &str,
    date: Option<IxDate>,
    required: bool,
    min: Option<IxDate>,
    max: Option<IxDate>,
) -> DateInputValidity {
    DateInputValidity {
        value_missing: required && value.trim().is_empty(),
        bad_input: !value.trim().is_empty() && date.is_none(),
        range_underflow: date.is_some_and(|date| min.is_some_and(|min| date < min)),
        range_overflow: date.is_some_and(|date| max.is_some_and(|max| date > max)),
    }
}
/// Validate typed time text, including 12-hour notation and bounds.
pub fn validate_time_input(
    value: &str,
    format: TimeFormat,
    required: bool,
    min: Option<IxTime>,
    max: Option<IxTime>,
) -> DateInputValidity {
    time_input_validity(value, format.parse(value), required, min, max)
}
fn time_input_validity(
    value: &str,
    time: Option<IxTime>,
    required: bool,
    min: Option<IxTime>,
    max: Option<IxTime>,
) -> DateInputValidity {
    let (min, max) = normalized_time_bounds(min, max);
    DateInputValidity {
        value_missing: required && value.trim().is_empty(),
        bad_input: !value.trim().is_empty() && time.is_none(),
        range_underflow: time.is_some_and(|time| min.is_some_and(|min| time < min)),
        range_overflow: time.is_some_and(|time| max.is_some_and(|max| time > max)),
    }
}

fn datetime_input_validity(
    value: &str,
    parsed: Option<IxDateTime>,
    required: bool,
    (min_date, max_date): (Option<IxDate>, Option<IxDate>),
    (min_time, max_time): (Option<IxTime>, Option<IxTime>),
) -> DateInputValidity {
    // The upstream input parses its time window before applying date scope;
    // the nested picker scopes first and then parses its effective window.
    let (min, max) = date_scoped_time_bounds(
        parsed.map(|value| value.date),
        (min_date, max_date),
        normalized_time_bounds(min_time, max_time),
    );
    DateInputValidity {
        value_missing: required && value.trim().is_empty(),
        bad_input: !value.trim().is_empty() && parsed.is_none(),
        range_underflow: parsed.is_some_and(|value| {
            min_date.is_some_and(|min| value.date < min) || min.is_some_and(|min| value.time < min)
        }),
        range_overflow: parsed.is_some_and(|value| {
            max_date.is_some_and(|max| value.date > max) || max.is_some_and(|max| value.time > max)
        }),
    }
}

/// Horizontal alignment for formatted date/time input text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateInputAlignment {
    #[default]
    Start,
    End,
}
impl DateInputAlignment {
    fn css(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

#[component]
fn TemporalField(
    id: String,
    name: Option<String>,
    form: Option<String>,
    label: String,
    value: String,
    placeholder: String,
    disabled: bool,
    readonly: bool,
    required: bool,
    alignment: DateInputAlignment,
    suppress_submit_on_enter: bool,
    error: Option<String>,
    helper_text: Option<String>,
    #[props(default)] state: crate::FieldState,
    info_text: Option<String>,
    warning_text: Option<String>,
    valid_text: Option<String>,
    invalid_text: Option<String>,
    start: Option<Element>,
    end: Option<Element>,
    range_binding: RangeBinding,
    button_label: String,
    icon: String,
    dropdown: crate::DropdownHandle,
    enable_top_layer: bool,
    #[props(default)] onprepare: EventHandler<()>,
    onvaluechange: EventHandler<String>,
    onmounted: EventHandler<MountedEvent>,
    children: Option<Element>,
) -> Element {
    let help_id = format!("{id}-help");
    let toggle_id = format!("{id}-toggle");
    let open = dropdown.is_open();
    let binding = crate::use_dropdown_trigger_binding(dropdown);
    let invalid = error.is_some() || state == crate::FieldState::Invalid;
    crate::use_custom_field_state(if invalid {
        crate::FieldState::Invalid
    } else {
        state
    });
    let constraint_invalid = error.is_some();
    let validation_title = error.clone();
    let feedback = error
        .or(match state {
            crate::FieldState::Info => info_text,
            crate::FieldState::Warning => warning_text,
            crate::FieldState::Valid => valid_text,
            crate::FieldState::Invalid => invalid_text,
            crate::FieldState::Default => None,
        })
        .or(helper_text);
    let state_class = match state {
        crate::FieldState::Default => "",
        crate::FieldState::Info => "ix-temporal-info",
        crate::FieldState::Warning => "ix-temporal-warning",
        crate::FieldState::Valid => "ix-temporal-valid",
        crate::FieldState::Invalid => "ix-temporal-invalid",
    };
    let pointer_binding = range_binding.clone();
    let key_binding = range_binding.clone();
    rsx! { div { class: "ix-temporal-field {state_class}", class: if invalid { "ix-temporal-invalid" }, class: if disabled { "ix-temporal-disabled" },
        onpointerdown: move |_| pointer_binding.input_mode(false),
        onkeydown: move |event| { key_binding.input_mode(true); if event.key() == Key::Escape && open { event.stop_propagation(); dropdown.close(true); } },
        if !label.is_empty() { label { r#for: id.clone(), "{label}" if required { span { aria_hidden: true, " *" } } } }
        div { class: "ix-temporal-control",
            if let Some(start) = start { span { class: "ix-temporal-slot", {start} } }
            input { id, name, form, r#type: "text", value, placeholder, disabled, readonly, required,
                // A never-matching native pattern blocks nonempty invalid date/time
                // values; `required` independently handles missing values.
                pattern: if constraint_invalid { Some("(?!)") } else { None }, title: validation_title,
                style: "text-align:{alignment.css()}", autocomplete: "off", spellcheck: false,
                aria_invalid: invalid, aria_describedby: help_id.clone(), aria_haspopup: "dialog", aria_controls: dropdown.menu_id(), aria_expanded: open,
                oninput: move |event| onvaluechange.call(event.value()),
                onmounted: move |event| {
                    *range_binding.input.borrow_mut() = Some(event.data());
                    onmounted.call(event);
                },
                onkeydown: move |event| {
                    if suppress_submit_on_enter && event.key() == Key::Enter { event.prevent_default(); }
                    if !disabled && !readonly && !open && matches!(event.key(), Key::ArrowUp | Key::ArrowDown) { event.prevent_default(); dropdown.open(); }
                },
            }
            if let Some(end) = end { span { class: "ix-temporal-slot", {end} } }
            if !readonly && !disabled { button { id: toggle_id.clone(), class: "ix-temporal-toggle", r#type: "button", disabled,
                aria_label: button_label.clone(), aria_expanded: open, aria_controls: dropdown.menu_id(), aria_haspopup: "dialog",
                onmounted: binding.onmounted, onvisible: binding.onvisible, onkeydown: binding.onkeydown, onclick: binding.onclick,
                ..binding.attributes(),
                crate::Icon { name: icon, size: 16 }
            } }
        }
        if let Some(message) = feedback { div { class: "ix-temporal-helper", id: help_id.clone(), role: if invalid { "alert" } else { "status" }, "{message}" } }
        crate::Dropdown { label: button_label, controller: dropdown, trigger: toggle_id.clone(),
            surface_role: crate::DropdownSurfaceRole::Dialog, close_behavior: crate::DropdownCloseBehavior::Outside,
            disable_focus_handling: true, enable_top_layer, disabled: disabled || readonly,
            // The text input also opens this surface; visibility of the separate
            // toggle is not a dismissal condition for a typed field.
            suppress_trigger_visibility_check: true,
            class: "ix-temporal-surface",
            onbeforechange: move |event: crate::DropdownChangeEvent| { if event.open { onprepare.call(()); } },
            if open && !disabled && !readonly { {children} }
        }
    } }
}

/// Formatted date text with iX calendar dropdown and typed validation callbacks.
/// `onvaluechange` receives raw text so invalid intermediate edits remain editable.
#[component]
pub fn DateInput(
    today: IxDate,
    #[props(default)] value: String,
    #[props(default)] format: DateFormat,
    /// Authoritative host text codec; calendar labels/options remain separate.
    /// Change its revision when stable callbacks acquire new configuration.
    #[props(default)]
    text_format: Option<DateTextFormat>,
    min_date: Option<IxDate>,
    max_date: Option<IxDate>,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    /// Render the shared picker surface in the native top layer when supported.
    #[props(default)]
    enable_top_layer: bool,
    #[props(default)] suppress_submit_on_enter: bool,
    #[props(default)] text_alignment: DateInputAlignment,
    #[props(default)] week_start_index: u8,
    #[props(default)] show_week_numbers: bool,
    #[props(default)] labels: CalendarLabels,
    #[props(default)] label: String,
    #[props(default = "Open calendar".to_string())] calendar_button_label: String,
    id: Option<String>,
    name: Option<String>,
    /// Explicit native form owner; omission retains normal ancestor association.
    #[props(default)]
    form: Option<String>,
    placeholder: Option<String>,
    helper_text: Option<String>,
    invalid_text: Option<String>,
    #[props(default)] state: crate::FieldState,
    info_text: Option<String>,
    warning_text: Option<String>,
    valid_text: Option<String>,
    start: Option<Element>,
    end: Option<Element>,
    /// Actual native input mount, after internal range registration.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
    onvaluechange: EventHandler<String>,
    ondatechange: Option<EventHandler<Option<IxDate>>>,
    onvaliditychange: Option<EventHandler<DateInputValidity>>,
) -> Element {
    let generated_id = use_hook(|| {
        format!(
            "ix-date-input-{}",
            NEXT_DATE_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let id = id.unwrap_or(generated_id);
    let dropdown = crate::use_dropdown();
    let selected = parse_text(&value, text_format, |value| format.parse(value));
    let validity = date_input_validity(&value, selected, required, min_date, max_date);
    let error = if validity.valid() {
        None
    } else {
        invalid_text.clone().or_else(|| validity.message("Date"))
    };
    let range_open = use_callback(move |_| dropdown.open());
    let range_binding = use_range_binding(range_open, !disabled && !readonly);
    rsx! {
        TemporalField { id, name, form, label,onmounted, value, placeholder: placeholder.unwrap_or_default(),
            disabled, readonly, required, alignment: text_alignment, suppress_submit_on_enter, error, helper_text, state, info_text, warning_text, valid_text, invalid_text, start, end, range_binding: range_binding.clone(),
            button_label: calendar_button_label, icon: "calendar", dropdown, enable_top_layer,
            onvaluechange: move |value: String| {
                let parsed = parse_text(&value, text_format, |value| format.parse(value));
                let validity = date_input_validity(&value, parsed, required, min_date, max_date);
                if let Some(handler) = onvaliditychange { handler.call(validity); }
                if let Some(handler) = ondatechange { handler.call(if validity.valid() { parsed } else { None }); }
                onvaluechange.call(value);
            },
            DatePicker { today, from: selected, min_date, max_date, single_selection: true, auto_focus: true, week_start_index, show_week_numbers, labels,
                ondatechange: |_| {},
                onselect: move |selection: DateSelection| {
                    if let Some(date) = selection.from {
                        onvaluechange.call(format_text(date, text_format, |date| format.format(date)));
                        if let Some(handler) = ondatechange { handler.call(Some(date)); }
                        if let Some(handler) = onvaliditychange { handler.call(DateInputValidity::default()); }
                        dropdown.close(!range_binding.advance());
                    }
                }
            }
        }
    }
}

/// Formatted time text with iX clock dropdown. The clock uses a draft until Confirm.
#[component]
pub fn TimeInput(
    #[props(default)] value: String,
    #[props(default)] format: TimeFormat,
    /// Authoritative host text codec; the preset still controls picker layout.
    /// Change its revision when stable callbacks acquire new configuration.
    #[props(default)]
    text_format: Option<TimeTextFormat>,
    min_time: Option<IxTime>,
    max_time: Option<IxTime>,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    /// Render the shared picker surface in the native top layer when supported.
    #[props(default)]
    enable_top_layer: bool,
    #[props(default)] suppress_submit_on_enter: bool,
    #[props(default)] text_alignment: DateInputAlignment,
    #[props(default = 1)] hour_interval: u8,
    #[props(default = 1)] minute_interval: u8,
    #[props(default = 1)] second_interval: u8,
    #[props(default = 100)] millisecond_interval: u16,
    #[props(default)] labels: TimeLabels,
    #[props(default)] label: String,
    #[props(default = "Toggle time picker".to_string())] time_button_label: String,
    id: Option<String>,
    name: Option<String>,
    /// Explicit native form owner; omission retains normal ancestor association.
    #[props(default)]
    form: Option<String>,
    placeholder: Option<String>,
    helper_text: Option<String>,
    invalid_text: Option<String>,
    #[props(default)] state: crate::FieldState,
    info_text: Option<String>,
    warning_text: Option<String>,
    valid_text: Option<String>,
    start: Option<Element>,
    end: Option<Element>,
    /// Actual native input mount, after internal range registration.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
    onvaluechange: EventHandler<String>,
    ontimechange: Option<EventHandler<Option<IxTime>>>,
    onvaliditychange: Option<EventHandler<DateInputValidity>>,
) -> Element {
    let generated_id = use_hook(|| {
        format!(
            "ix-time-input-{}",
            NEXT_DATE_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let id = id.unwrap_or(generated_id);
    let dropdown = crate::use_dropdown();
    let parsed = parse_text(&value, text_format, |value| format.parse(value));
    let selected = parsed.unwrap_or_default();
    let mut draft = use_signal(|| selected);
    let range_open = use_callback(move |_| dropdown.open());
    let range_binding = use_range_binding(range_open, !disabled && !readonly);
    let validity = time_input_validity(&value, parsed, required, min_time, max_time);
    let error = if validity.valid() {
        None
    } else {
        invalid_text.clone().or_else(|| validity.message("Time"))
    };
    rsx! {
        TemporalField { id, name, form, label,onmounted, value, placeholder: placeholder.unwrap_or_default(),
            disabled, readonly, required, alignment: text_alignment, suppress_submit_on_enter, error, helper_text, state, info_text, warning_text, valid_text, invalid_text, start, end, range_binding: range_binding.clone(),
            button_label: time_button_label, icon: "clock", dropdown, enable_top_layer,
            onprepare: move |_| draft.set(selected),
            onvaluechange: move |value: String| {
                let parsed = parse_text(&value, text_format, |value| format.parse(value));
                let validity = time_input_validity(&value, parsed, required, min_time, max_time);
                if let Some(handler) = onvaliditychange { handler.call(validity); }
                if let Some(handler) = ontimechange { handler.call(if validity.valid() { parsed } else { None }); }
                onvaluechange.call(value);
            },
            TimePicker { time: draft(), format, min_time, max_time, hour_interval, minute_interval, second_interval, millisecond_interval, labels, auto_focus: true,
                ontimechange: move |time| draft.set(time),
                onselect: move |time| {
                    onvaluechange.call(format_text(time, text_format, |time| format.format(time)));
                    if let Some(handler) = ontimechange { handler.call(Some(time)); }
                    if let Some(handler) = onvaliditychange { handler.call(DateInputValidity::default()); }
                    dropdown.close(!range_binding.advance());
                }
            }
        }
    }
}

/// A local date and time, deliberately not an instant or timezone conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IxDateTime {
    pub date: IxDate,
    pub time: IxTime,
}
impl IxDateTime {
    pub fn format(self, date_format: DateFormat, time_format: TimeFormat) -> String {
        format!(
            "{} {}",
            date_format.format(self.date),
            time_format.format(self.time)
        )
    }
    pub fn parse(value: &str, date_format: DateFormat, time_format: TimeFormat) -> Option<Self> {
        let value = value.trim();
        // A named date and a twelve-hour clock can both contain spaces. Let
        // each typed parser recognize its portion instead of splitting at the
        // first space, which is part of a localized date.
        value
            .char_indices()
            .filter(|(_, ch)| ch.is_whitespace())
            .find_map(|(index, separator)| {
                Some(Self {
                    date: date_format.parse(&value[..index])?,
                    time: time_format.parse(&value[index + separator.len_utf8()..])?,
                })
            })
    }
}

/// Formatted date/time input with a combined calendar and clock dropdown.
#[component]
pub fn DateTimeInput(
    today: IxDate,
    #[props(default)] value: String,
    #[props(default)] date_format: DateFormat,
    #[props(default)] time_format: TimeFormat,
    /// Authoritative host text codec; `time_format` still controls clock columns.
    /// Change its revision when stable callbacks acquire new configuration.
    #[props(default)]
    text_format: Option<DateTimeTextFormat>,
    min_date: Option<IxDate>,
    max_date: Option<IxDate>,
    min_time: Option<IxTime>,
    max_time: Option<IxTime>,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    /// Render the shared picker surface in the native top layer when supported.
    #[props(default)]
    enable_top_layer: bool,
    #[props(default)] suppress_submit_on_enter: bool,
    #[props(default)] text_alignment: DateInputAlignment,
    #[props(default)] week_start_index: u8,
    #[props(default)] show_week_numbers: bool,
    #[props(default)] calendar_labels: CalendarLabels,
    #[props(default)] time_labels: TimeLabels,
    #[props(default = "Done".to_string())] done_label: String,
    #[props(default = "Date and time".to_string())] label: String,
    #[props(default = "Toggle calendar".to_string())] calendar_button_label: String,
    id: Option<String>,
    name: Option<String>,
    /// Explicit native form owner; omission retains normal ancestor association.
    #[props(default)]
    form: Option<String>,
    placeholder: Option<String>,
    helper_text: Option<String>,
    invalid_text: Option<String>,
    #[props(default)] state: crate::FieldState,
    info_text: Option<String>,
    warning_text: Option<String>,
    valid_text: Option<String>,
    start: Option<Element>,
    end: Option<Element>,
    /// Actual native input mount, after internal range registration.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
    onvaluechange: EventHandler<String>,
    ondatetimechange: Option<EventHandler<Option<IxDateTime>>>,
    onvaliditychange: Option<EventHandler<DateInputValidity>>,
) -> Element {
    let generated_id = use_hook(|| {
        format!(
            "ix-datetime-input-{}",
            NEXT_DATE_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let id = id.unwrap_or(generated_id);
    let selected = parse_text(&value, text_format, |value| {
        IxDateTime::parse(value, date_format, time_format)
    });
    let dropdown = crate::use_dropdown();
    let mut draft_date = use_signal(|| Some(selected.map_or(today, |value| value.date)));
    let mut draft_time = use_signal(|| selected.map_or(IxTime::default(), |value| value.time));
    let range_open = use_callback(move |_| dropdown.open());
    let range_binding = use_range_binding(range_open, !disabled && !readonly);
    let validity = datetime_input_validity(
        &value,
        selected,
        required,
        (min_date, max_date),
        (min_time, max_time),
    );
    let error = if validity.valid() {
        None
    } else {
        invalid_text
            .clone()
            .or_else(|| validity.message("Date time"))
    };
    rsx! {
        TemporalField { id, name, form, label,onmounted, value, placeholder: placeholder.unwrap_or_default(),
            disabled, readonly, required, alignment: text_alignment, suppress_submit_on_enter, error, helper_text, state, info_text, warning_text, valid_text, invalid_text, start, end, range_binding: range_binding.clone(),
            button_label: calendar_button_label, icon: "calendar", dropdown, enable_top_layer,
            onprepare: move |_| { draft_date.set(Some(selected.map_or(today, |value| value.date))); draft_time.set(selected.map_or(IxTime::default(), |value| value.time)); },
            onvaluechange: move |value: String| {
                let parsed = parse_text(&value, text_format, |value| IxDateTime::parse(value, date_format, time_format));
                let validity = datetime_input_validity(&value, parsed, required, (min_date, max_date), (min_time, max_time));
                if let Some(handler) = onvaliditychange { handler.call(validity); }
                if let Some(handler) = ondatetimechange { handler.call(if validity.valid() { parsed } else { None }); }
                onvaluechange.call(value);
            },
            DateTimePicker { today, from: draft_date(), time: draft_time(), single_selection: true, auto_focus: true, time_format, min_date, max_date, min_time, max_time, week_start_index, show_week_numbers, calendar_labels, time_labels, done_label,
                ondatechange: move |selection: DateSelection| draft_date.set(selection.from),
                ontimechange: move |time| draft_time.set(time),
                onselect: move |selection: DateTimeSelection| {
                    if let Some(date) = selection.dates.from {
                        let datetime = IxDateTime { date, time: selection.time };
                        onvaluechange.call(format_text(datetime, text_format, |datetime| datetime.format(date_format, time_format)));
                        if let Some(handler) = ondatetimechange { handler.call(Some(datetime)); }
                        if let Some(handler) = onvaliditychange { handler.call(DateInputValidity::default()); }
                        dropdown.close(!range_binding.advance());
                    }
                }
            }
        }
    }
}

/// A named preset date range in a date dropdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateDropdownOption {
    pub id: String,
    pub label: String,
    pub from: IxDate,
    pub to: IxDate,
}
/// A selected named/custom date range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateRangeChange {
    pub id: String,
    pub dates: DateSelection,
}

/// Date-range dropdown with an embedded calendar and optional preset sidebar.
/// Presets publish immediately and keep the dropdown open. Done, Escape and
/// outside dismissal also publish the current range, including an unchanged
/// selection, matching upstream. Calendar edits publish when dismissed. An empty
/// preset list displays only the calendar. Done publishes its explicit selection
/// event followed by the dismissal event, as in the source component.
/// The persistent shared Dropdown surface owns dismissal, focus restoration and
/// cleanup. Enable its native top layer to escape a clipped parent layout.
/// Attach a [`DateDropdownController`] to query the current range, including
/// unpublished calendar edits, without publishing or dismissing it.
#[component]
pub fn DateDropdown(
    today: IxDate,
    from: Option<IxDate>,
    to: Option<IxDate>,
    min_date: Option<IxDate>,
    max_date: Option<IxDate>,
    #[props(default)] options: Vec<DateDropdownOption>,
    #[props(default = "custom".to_string())] date_range_id: String,
    #[props(default)] format: DateFormat,
    #[props(default)] single_selection: bool,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    #[props(default)] controller: Option<DateDropdownController>,
    #[props(default)] enable_top_layer: bool,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] week_start_index: u8,
    #[props(default)] show_week_numbers: bool,
    #[props(default)] variant: crate::ButtonVariant,
    #[props(default)] labels: CalendarLabels,
    #[props(default = "No range set".to_string())] empty_label: String,
    #[props(default = "Custom range".to_string())] custom_label: String,
    onrangechange: EventHandler<DateRangeChange>,
) -> Element {
    let dropdown = crate::use_dropdown();
    let binding = crate::use_dropdown_trigger_binding(dropdown);
    let preset = options.iter().find(|option| option.id == date_range_id);
    let committed = DateSelection {
        from: from.or_else(|| preset.map(|option| option.from)),
        to: to.or_else(|| preset.map(|option| option.to)),
    };
    let mut draft = use_signal(|| committed);
    let mut draft_id = use_signal(|| date_range_id.clone());
    crate::date_dropdown_controller::use_date_dropdown_binding(
        controller.as_ref(),
        draft,
        draft_id,
    );
    let mut picker_epoch = use_signal(|| 0_u64);
    use_effect(use_reactive!(|committed, date_range_id| {
        // The initial effect already agrees with the initial signals. Avoid
        // scheduling an extra render, and do not subscribe this effect to local
        // preset/calendar updates when checking external prop synchronization.
        if *draft.peek() != committed {
            draft.set(committed);
        }
        if *draft_id.peek() != date_range_id {
            draft_id.set(date_range_id);
        }
    }));
    let current = draft();
    let button_label = match (current.from, current.to) {
        (Some(from), Some(to)) if !single_selection && from != to => {
            format!("{} - {}", format.format(from), format.format(to))
        }
        (Some(from), _) => format.format(from),
        _ => empty_label,
    };
    let changed = use_callback(move |open: bool| {
        if open {
            // A new opening presents the committed/current range's month,
            // while the shared surface itself keeps its mounted identity.
            picker_epoch += 1;
        } else {
            onrangechange.call(DateRangeChange {
                id: draft_id(),
                dates: draft(),
            });
        }
    });
    let done = use_callback(move |_| {
        if disabled || loading {
            return;
        }
        onrangechange.call(DateRangeChange {
            id: draft_id(),
            dates: draft(),
        });
        dropdown.close(true);
    });
    let trigger_id = use_hook(|| {
        format!(
            "ix-date-dropdown-trigger-{}",
            NEXT_DATE_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let button_class = format!("ix-button ix-button-{} ix-align-center", variant.as_str());
    rsx! { div { id, class: "ix-date-dropdown {class}",
        button { id: trigger_id.clone(), class: button_class, r#type: "button", disabled: disabled || loading,
            aria_expanded: dropdown.is_open(), aria_controls: dropdown.menu_id(), aria_haspopup: "dialog", aria_busy: loading,
            onmounted: binding.onmounted, onvisible: binding.onvisible, onkeydown: binding.onkeydown,
            onclick: binding.onclick,
            ..binding.attributes(),
            if loading { crate::Spinner { size: crate::SpinnerSize::Sm } }
            else { crate::Icon { name: "history".to_string(), size: 24 } }
            span { "{button_label}" }
            crate::Icon { name: "chevron-down-small".to_string(), size: 24 }
        }
        crate::Dropdown { label: "Date range", controller: dropdown, trigger: trigger_id.clone(),
            surface_role: crate::DropdownSurfaceRole::Dialog, close_behavior: crate::DropdownCloseBehavior::Outside,
            disable_focus_handling: true, enable_top_layer, disabled, loading,
            class: "ix-date-dropdown-surface", onchange: changed,
                div { class: "ix-date-dropdown-content",
                    if options.len() > 1 { div { class: "ix-date-preset-list",
                        for option in options {
                            button { key: "{option.id}", class: "ix-date-preset", r#type: "button",
                                aria_label: format!("{}: {} to {}", option.label, format.format(option.from), format.format(option.to)),
                                aria_pressed: draft_id() == option.id,
                                disabled: option.from > option.to || !option.from.within(min_date, max_date) || !option.to.within(min_date, max_date),
                                onclick: move |_| {
                                    if disabled || loading || !dropdown.is_open() { return; }
                                    let selection = DateSelection { from: Some(option.from), to: if single_selection { None } else { Some(option.to) } };
                                    draft_id.set(option.id.clone());
                                    draft.set(selection);
                                    // A repeated preset selection also restores its viewed
                                    // year/month after navigating the calendar elsewhere.
                                    picker_epoch += 1;
                                    onrangechange.call(DateRangeChange { id: option.id.clone(), dates: selection });
                                },
                                "{option.label}"
                            }
                        }
                    } }
                    div { class: "ix-date-dropdown-calendar", aria_label: custom_label,
                        DatePicker { key: "{picker_epoch}", today, from: draft().from, to: draft().to, min_date, max_date, single_selection, week_start_index, show_week_numbers, labels: labels.clone(), embedded: true,
                            ondatechange: move |selection| { draft_id.set("custom".into()); draft.set(selection); } }
                        div { class: "ix-date-card-footer", crate::Button { aria_label: labels.done.clone(), onclick: move |_| done.call(()), "{labels.done}" } }
                    }
                }
        }
    } }
}

pub(crate) const STYLES: &str = include_str!("styles/datetime.css");

#[cfg(test)]
#[path = "datetime/date_dropdown_controller_tests.rs"]
mod date_dropdown_controller_tests;

#[cfg(test)]
#[path = "datetime/date_dropdown_surface_tests.rs"]
mod date_dropdown_surface_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "datetime/card_tests.rs"]
mod card_tests;

#[cfg(test)]
#[path = "datetime/text_format_tests.rs"]
mod text_format_tests;

#[cfg(test)]
#[path = "datetime/clock_composition_tests.rs"]
mod clock_composition_tests;

#[cfg(test)]
#[path = "datetime/clock_reveal_tests.rs"]
mod clock_reveal_tests;

#[cfg(test)]
#[path = "datetime/calendar_mount_tests.rs"]
mod calendar_mount_tests;

#[cfg(test)]
#[path = "datetime/clock_mount_tests.rs"]
mod clock_mount_tests;
