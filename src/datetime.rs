//! Calendar and clock controls implemented entirely in Rust.
//!
//! Date values have no timezone. Supply the user's current local date through
//! `today`; the library deliberately avoids guessing a timezone on the server.
use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
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
    handoff_from: Rc<Cell<Option<usize>>>,
}

/// Install the range's local focus scope without any browser-specific API.
pub(crate) fn provide_range_navigation() {
    use_context_provider(|| TemporalRangeContext {
        targets: Rc::new(RefCell::new(BTreeMap::new())),
        next: Rc::new(Cell::new(0)),
        keyboard: Rc::new(Cell::new(false)),
        handoff_from: Rc::new(Cell::new(None)),
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
    fn suppress_return_focus(&self) -> bool {
        self.context.as_ref().is_some_and(|context| {
            if context.handoff_from.get() == Some(self.id) {
                context.handoff_from.set(None);
                true
            } else {
                false
            }
        })
    }
    fn advance(&self) {
        let Some(context) = &self.context else {
            return;
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
            return;
        };
        context.handoff_from.set(Some(self.id));
        spawn(async move {
            let input = next.input.borrow().clone();
            if let Some(input) = input {
                let _ = input.set_focus(true).await;
            }
            if next.enabled.get() {
                next.activate.call(());
            }
        });
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
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
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
        }
    }
    /// Strictly parse this format; impossible dates are rejected.
    pub fn parse(self, value: &str) -> Option<IxDate> {
        let separator = match self {
            Self::Iso => '-',
            Self::DmyDot => '.',
            _ => '/',
        };
        let parts: Vec<_> = value.trim().split(separator).collect();
        if parts.len() != 3
            || !parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return None;
        }
        let (year, month, day) = match self {
            Self::YmdSlash | Self::Iso => (parts[0], parts[1], parts[2]),
            Self::DmyDot | Self::DmySlash => (parts[2], parts[1], parts[0]),
            Self::MdySlash => (parts[2], parts[0], parts[1]),
        };
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
        if single || self.from.is_none() || self.to.is_some() {
            Self {
                from: Some(date),
                to: None,
            }
        } else {
            let from = self.from.unwrap();
            Self {
                from: Some(from.min(date)),
                to: Some(from.max(date)),
            }
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
#[component]
pub fn DateTimeCard(
    children: Element,
    header: Option<Element>,
    footer: Option<Element>,
    #[props(default)] embedded: bool,
    #[props(default)] hide_header: bool,
    #[props(default)] no_padding: bool,
    #[props(default)] corners: DateTimeCorners,
) -> Element {
    rsx! { div { class: "ix-date-time-card ix-date-corners-{corners.class()}",
        class: if embedded { "ix-date-embedded" }, class: if no_padding { "ix-date-no-padding" },
        if !hide_header { if let Some(header) = header { div { class: "ix-date-card-header", {header} } } }
        div { class: "ix-date-card-content", {children} }
        if let Some(footer) = footer { div { class: "ix-date-card-footer", {footer} } }
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
    use_effect(use_reactive!(|from, min_date, max_date| {
        let anchor = from.unwrap_or(today);
        let anchor = min_date.map_or(anchor, |min| anchor.max(min));
        let anchor = max_date.map_or(anchor, |max| anchor.min(max));
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
            for (index, name) in month_names.into_iter().enumerate() { option { value: (index + 1).to_string(), disabled: !month_selectable(IxDate::new(visible.year, index as u8 + 1, 1).unwrap(), min_date, max_date), "{name}" } }
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
            if time.hour % 12 == 0 {
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

async fn reveal_clock_option(container: Rc<MountedData>, option: Rc<MountedData>) {
    let Ok(container_rect) = container.get_client_rect().await else {
        return;
    };
    let Ok(option_rect) = option.get_client_rect().await else {
        return;
    };
    let Ok(offset) = container.get_scroll_offset().await else {
        return;
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
        let _ = container
            .scroll(
                dioxus::html::geometry::PixelsVector2D::new(offset.x, top),
                ScrollBehavior::Instant,
            )
            .await;
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
    let references = use_hook(|| {
        Rc::new(RefCell::new(
            BTreeMap::<(ClockUnit, u16), Rc<MountedData>>::new(),
        ))
    });
    let containers =
        use_hook(|| Rc::new(RefCell::new(BTreeMap::<ClockUnit, Rc<MountedData>>::new())));
    let reveal_option = {
        let references = references.clone();
        let containers = containers.clone();
        use_callback(move |key: (ClockUnit, u16)| {
            let option = references.borrow().get(&key).cloned();
            let container = containers.borrow().get(&key.0).cloned();
            if let (Some(container), Some(option)) = (container, option) {
                spawn(reveal_clock_option(container, option));
            }
        })
    };
    let register_container = use_callback(
        move |(unit, element, selected): (ClockUnit, Rc<MountedData>, Option<u16>)| {
            containers.borrow_mut().insert(unit, element);
            if let Some(value) = selected {
                reveal_option.call((unit, value));
            }
        },
    );
    let register = {
        let references = references.clone();
        use_callback(move |(key, element): ((ClockUnit, u16), Rc<MountedData>)| {
            references.borrow_mut().insert(key, element);
        })
    };
    let focus_option = use_callback(move |key: (ClockUnit, u16)| {
        focused_option.set(Some(key));
        let element = references.borrow().get(&key).cloned();
        if let Some(element) = element {
            spawn(async move {
                let _ = element.set_focus(true).await;
            });
        }
    });
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
            let roving = provisional.or_else(|| {
                selectable
                    .into_iter()
                    .min_by_key(|value| value.abs_diff(time.unit(unit)))
            });
            (unit, interval, label, roving)
        })
        .collect();
    let header = rsx! { div { class: "ix-clock-header", "{labels.header}"
        if format.is_twelve_hour() { div { class: "ix-clock-meridiem",
            crate::ToggleButton { pressed: time.hour < 12, disabled: disabled || !IxTime { hour: time.hour % 12, ..time }.within(min_time, max_time),
                onpressedchange: move |_| ontimechange.call(IxTime { hour: time.hour % 12, ..time }), "{labels.am}" }
            crate::ToggleButton { pressed: time.hour >= 12, disabled: disabled || !IxTime { hour: time.hour % 12 + 12, ..time }.within(min_time, max_time),
                onpressedchange: move |_| ontimechange.call(IxTime { hour: time.hour % 12 + 12, ..time }), "{labels.pm}" }
        } }
    } };
    let footer = if !embedded {
        Some(
            rsx! { crate::Button { disabled: disabled || !time.within(min_time, max_time), onclick: move |_| { if let Some(handler) = onselect { handler.call(time); } }, "{labels.confirm}" } },
        )
    } else {
        None
    };
    rsx! { DateTimeCard { header, footer, corners, embedded, hide_header,
        div { class: "ix-clock-columns",
            for (unit, interval, label, roving) in columns {
                div { class: "ix-clock-column",
                    div { class: "ix-clock-column-heading", "{label}" }
                    div { class: "ix-clock-options", role: "listbox", aria_label: unit.name(), aria_disabled: disabled,
                        onmounted: move |event| register_container.call((unit, event.data(), roving)),
                        for value in clock_unit_values(unit, interval, format, time) {
                            button { key: "{value}", class: "ix-clock-option", class: if time.unit(unit) == value { "ix-clock-selected" },
                                r#type: "button", role: "option", aria_selected: time.unit(unit) == value,
                                disabled: disabled || select_clock_unit(time, unit, value, min_time, max_time).is_none(),
                                tabindex: if roving == Some(value) { 0 } else { -1 },
                                onmounted: move |event| {
                                    let element = event.data();
                                    register.call(((unit, value), element.clone()));
                                    if roving == Some(value) {
                                        reveal_option.call((unit, value));
                                        if auto_focus && unit == ClockUnit::Hour { focus_option.call((unit, value)); }
                                    }
                                },
                                onfocus: move |_| focused_option.set(Some((unit, value))),
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
    let date = format.parse(value);
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
    let (min, max) = normalized_time_bounds(min, max);
    let time = format.parse(value);
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
    open: bool,
    onvaluechange: EventHandler<String>,
    onopen: EventHandler<()>,
    onclose: EventHandler<()>,
    children: Option<Element>,
) -> Element {
    let help_id = format!("{id}-help");
    let popup_id = format!("{id}-picker");
    let anchor = use_hook(|| {
        format!(
            "--ix-temporal-anchor-{}",
            NEXT_DATE_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let invalid = error.is_some() || state == crate::FieldState::Invalid;
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
    let trigger_ref = use_hook(|| Rc::new(RefCell::new(None::<Rc<MountedData>>)));
    let mut was_open = use_signal(|| open);
    {
        let trigger_ref = trigger_ref.clone();
        let range_binding = range_binding.clone();
        use_effect(use_reactive!(|open| {
            // Observe the state transition itself: confirmation, Escape, outside
            // dismissal, and a host-controlled close all share this restoration.
            if *was_open.peek() && !open && !range_binding.suppress_return_focus() {
                let element = trigger_ref.borrow().clone();
                if let Some(element) = element {
                    spawn(async move {
                        let _ = element.set_focus(true).await;
                    });
                }
            }
            was_open.set(open);
        }));
    }
    let pointer_binding = range_binding.clone();
    let key_binding = range_binding.clone();
    rsx! { div { class: "ix-temporal-field {state_class}", style: "anchor-name:{anchor}", class: if invalid { "ix-temporal-invalid" }, class: if disabled { "ix-temporal-disabled" },
        onpointerdown: move |_| pointer_binding.input_mode(false),
        onkeydown: move |event| { key_binding.input_mode(true); if event.key() == Key::Escape && open { event.stop_propagation(); onclose.call(()); } },
        if !label.is_empty() { label { r#for: id.clone(), "{label}" if required { span { aria_hidden: true, " *" } } } }
        div { class: "ix-temporal-control",
            if let Some(start) = start { span { class: "ix-temporal-slot", {start} } }
            input { id, name, r#type: "text", value, placeholder, disabled, readonly, required,
                // A never-matching native pattern blocks nonempty invalid date/time
                // values; `required` independently handles missing values.
                pattern: if constraint_invalid { Some("(?!)") } else { None }, title: validation_title,
                style: "text-align:{alignment.css()}", autocomplete: "off", spellcheck: false,
                aria_invalid: invalid, aria_describedby: help_id.clone(), aria_haspopup: "dialog", aria_controls: popup_id.clone(), aria_expanded: open,
                oninput: move |event| onvaluechange.call(event.value()),
                onmounted: move |event| *range_binding.input.borrow_mut() = Some(event.data()),
                onkeydown: move |event| {
                    if suppress_submit_on_enter && event.key() == Key::Enter { event.prevent_default(); }
                    if !disabled && !readonly && !open && matches!(event.key(), Key::ArrowUp | Key::ArrowDown) { event.prevent_default(); onopen.call(()); }
                },
            }
            if let Some(end) = end { span { class: "ix-temporal-slot", {end} } }
            if !readonly && !disabled { button { class: "ix-temporal-toggle", r#type: "button", disabled,
                aria_label: button_label.clone(), aria_expanded: open, aria_controls: popup_id.clone(), aria_haspopup: "dialog",
                onmounted: move |event| *trigger_ref.borrow_mut() = Some(event.data()),
                onclick: move |_| onopen.call(()), crate::Icon { name: icon, size: 16 }
            } }
        }
        if let Some(message) = feedback { div { class: "ix-temporal-helper", id: help_id.clone(), role: if invalid { "alert" } else { "status" }, "{message}" } }
        if open && !disabled && !readonly {
            button { class: "ix-temporal-backdrop", tabindex: -1, r#type: "button", aria_label: "Close picker", onclick: move |_| onclose.call(()) }
            div { class: "ix-temporal-popover", style: "position-anchor:{anchor}", role: "dialog", id: popup_id.clone(), aria_label: button_label,
                {children}
            }
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
    min_date: Option<IxDate>,
    max_date: Option<IxDate>,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] suppress_submit_on_enter: bool,
    #[props(default)] text_alignment: DateInputAlignment,
    #[props(default)] week_start_index: u8,
    #[props(default)] show_week_numbers: bool,
    #[props(default)] labels: CalendarLabels,
    #[props(default)] label: String,
    #[props(default = "Open calendar".to_string())] calendar_button_label: String,
    id: Option<String>,
    name: Option<String>,
    placeholder: Option<String>,
    helper_text: Option<String>,
    invalid_text: Option<String>,
    #[props(default)] state: crate::FieldState,
    info_text: Option<String>,
    warning_text: Option<String>,
    valid_text: Option<String>,
    start: Option<Element>,
    end: Option<Element>,
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
    let mut open = use_signal(|| false);
    let validity = validate_date_input(&value, format, required, min_date, max_date);
    let error = if validity.valid() {
        None
    } else {
        invalid_text.clone().or_else(|| validity.message("Date"))
    };
    let selected = format.parse(&value);
    let range_open = use_callback(move |_| open.set(true));
    let range_binding = use_range_binding(range_open, !disabled && !readonly);
    rsx! {
        TemporalField { id, name, label, value, placeholder: placeholder.unwrap_or_default(),
            disabled, readonly, required, alignment: text_alignment, suppress_submit_on_enter, error, helper_text, state, info_text, warning_text, valid_text, invalid_text, start, end, range_binding: range_binding.clone(),
            button_label: calendar_button_label, icon: "calendar", open: open(),
            onopen: move |_| open.toggle(), onclose: move |_| open.set(false),
            onvaluechange: move |value: String| {
                let validity = validate_date_input(&value, format, required, min_date, max_date);
                if let Some(handler) = onvaliditychange { handler.call(validity); }
                if let Some(handler) = ondatechange { handler.call(if validity.valid() { format.parse(&value) } else { None }); }
                onvaluechange.call(value);
            },
            DatePicker { today, from: selected, min_date, max_date, single_selection: true, auto_focus: true, week_start_index, show_week_numbers, labels,
                ondatechange: |_| {},
                onselect: move |selection: DateSelection| {
                    if let Some(date) = selection.from {
                        onvaluechange.call(format.format(date));
                        if let Some(handler) = ondatechange { handler.call(Some(date)); }
                        if let Some(handler) = onvaliditychange { handler.call(DateInputValidity::default()); }
                        open.set(false); range_binding.advance();
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
    min_time: Option<IxTime>,
    max_time: Option<IxTime>,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
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
    placeholder: Option<String>,
    helper_text: Option<String>,
    invalid_text: Option<String>,
    #[props(default)] state: crate::FieldState,
    info_text: Option<String>,
    warning_text: Option<String>,
    valid_text: Option<String>,
    start: Option<Element>,
    end: Option<Element>,
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
    let mut open = use_signal(|| false);
    let mut draft = use_signal(|| format.parse(&value).unwrap_or_default());
    let selected = format.parse(&value).unwrap_or_default();
    let range_open = use_callback(move |_| {
        draft.set(selected);
        open.set(true);
    });
    let range_binding = use_range_binding(range_open, !disabled && !readonly);
    let validity = validate_time_input(&value, format, required, min_time, max_time);
    let error = if validity.valid() {
        None
    } else {
        invalid_text.clone().or_else(|| validity.message("Time"))
    };
    rsx! {
        TemporalField { id, name, label, value, placeholder: placeholder.unwrap_or_default(),
            disabled, readonly, required, alignment: text_alignment, suppress_submit_on_enter, error, helper_text, state, info_text, warning_text, valid_text, invalid_text, start, end, range_binding: range_binding.clone(),
            button_label: time_button_label, icon: "clock", open: open(),
            onopen: move |_| { draft.set(selected); open.toggle(); }, onclose: move |_| open.set(false),
            onvaluechange: move |value: String| {
                let validity = validate_time_input(&value, format, required, min_time, max_time);
                if let Some(handler) = onvaliditychange { handler.call(validity); }
                if let Some(handler) = ontimechange { handler.call(if validity.valid() { format.parse(&value) } else { None }); }
                onvaluechange.call(value);
            },
            TimePicker { time: draft(), format, min_time, max_time, hour_interval, minute_interval, second_interval, millisecond_interval, labels, auto_focus: true,
                ontimechange: move |time| draft.set(time),
                onselect: move |time| {
                    onvaluechange.call(format.format(time));
                    if let Some(handler) = ontimechange { handler.call(Some(time)); }
                    if let Some(handler) = onvaliditychange { handler.call(DateInputValidity::default()); }
                    open.set(false); range_binding.advance();
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
        let (date, time) = value.trim().split_once(' ')?;
        Some(Self {
            date: date_format.parse(date)?,
            time: time_format.parse(time)?,
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
    min_date: Option<IxDate>,
    max_date: Option<IxDate>,
    min_time: Option<IxTime>,
    max_time: Option<IxTime>,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] suppress_submit_on_enter: bool,
    #[props(default)] text_alignment: DateInputAlignment,
    #[props(default)] week_start_index: u8,
    #[props(default)] show_week_numbers: bool,
    #[props(default)] calendar_labels: CalendarLabels,
    #[props(default)] time_labels: TimeLabels,
    #[props(default = "Date and time".to_string())] label: String,
    #[props(default = "Toggle calendar".to_string())] calendar_button_label: String,
    id: Option<String>,
    name: Option<String>,
    placeholder: Option<String>,
    helper_text: Option<String>,
    invalid_text: Option<String>,
    #[props(default)] state: crate::FieldState,
    info_text: Option<String>,
    warning_text: Option<String>,
    valid_text: Option<String>,
    start: Option<Element>,
    end: Option<Element>,
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
    let selected = IxDateTime::parse(&value, date_format, time_format);
    let mut open = use_signal(|| false);
    let mut draft_date = use_signal(|| Some(selected.map_or(today, |value| value.date)));
    let mut draft_time = use_signal(|| selected.map_or(IxTime::default(), |value| value.time));
    let range_open = use_callback(move |_| {
        draft_date.set(Some(selected.map_or(today, |value| value.date)));
        draft_time.set(selected.map_or(IxTime::default(), |value| value.time));
        open.set(true);
    });
    let range_binding = use_range_binding(range_open, !disabled && !readonly);
    let validate = use_callback(move |value: String| {
        let parsed = IxDateTime::parse(&value, date_format, time_format);
        datetime_input_validity(
            &value,
            parsed,
            required,
            (min_date, max_date),
            (min_time, max_time),
        )
    });
    let validity = validate.call(value.clone());
    let error = if validity.valid() {
        None
    } else {
        invalid_text
            .clone()
            .or_else(|| validity.message("Date time"))
    };
    rsx! {
        TemporalField { id, name, label, value, placeholder: placeholder.unwrap_or_default(),
            disabled, readonly, required, alignment: text_alignment, suppress_submit_on_enter, error, helper_text, state, info_text, warning_text, valid_text, invalid_text, start, end, range_binding: range_binding.clone(),
            button_label: calendar_button_label, icon: "calendar", open: open(),
            onopen: move |_| { draft_date.set(Some(selected.map_or(today, |value| value.date))); draft_time.set(selected.map_or(IxTime::default(), |value| value.time)); open.toggle(); },
            onclose: move |_| open.set(false),
            onvaluechange: move |value: String| {
                let validity = validate.call(value.clone());
                if let Some(handler) = onvaliditychange { handler.call(validity); }
                if let Some(handler) = ondatetimechange { handler.call(if validity.valid() { IxDateTime::parse(&value, date_format, time_format) } else { None }); }
                onvaluechange.call(value);
            },
            DateTimePicker { today, from: draft_date(), time: draft_time(), single_selection: true, auto_focus: true, time_format, min_date, max_date, min_time, max_time, week_start_index, show_week_numbers, calendar_labels, time_labels,
                ondatechange: move |selection: DateSelection| draft_date.set(selection.from),
                ontimechange: move |time| draft_time.set(time),
                onselect: move |selection: DateTimeSelection| {
                    if let Some(date) = selection.dates.from {
                        let datetime = IxDateTime { date, time: selection.time };
                        onvaluechange.call(datetime.format(date_format, time_format));
                        if let Some(handler) = ondatetimechange { handler.call(Some(datetime)); }
                        if let Some(handler) = onvaliditychange { handler.call(DateInputValidity::default()); }
                        open.set(false); range_binding.advance();
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
    #[props(default)] week_start_index: u8,
    #[props(default)] show_week_numbers: bool,
    #[props(default)] variant: crate::ButtonVariant,
    #[props(default)] labels: CalendarLabels,
    #[props(default = "No range set".to_string())] empty_label: String,
    #[props(default = "Custom range".to_string())] custom_label: String,
    onrangechange: EventHandler<DateRangeChange>,
) -> Element {
    let mut open = use_signal(|| false);
    let preset = options.iter().find(|option| option.id == date_range_id);
    let committed = DateSelection {
        from: from.or_else(|| preset.map(|option| option.from)),
        to: to.or_else(|| preset.map(|option| option.to)),
    };
    let mut draft = use_signal(|| committed);
    let mut draft_id = use_signal(|| date_range_id.clone());
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
    let close = use_callback(move |_| {
        onrangechange.call(DateRangeChange {
            id: draft_id(),
            dates: draft(),
        });
        open.set(false);
    });
    let done = use_callback(move |_| {
        onrangechange.call(DateRangeChange {
            id: draft_id(),
            dates: draft(),
        });
        close.call(());
    });
    let popup_id = use_hook(|| {
        format!(
            "ix-date-dropdown-{}",
            NEXT_DATE_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let mut trigger = use_signal(|| None::<Rc<MountedData>>);
    let mut was_open = use_signal(|| false);
    use_effect(move || {
        let is_open = open();
        if *was_open.peek() && !is_open {
            let element = trigger.peek().clone();
            spawn(async move {
                if let Some(element) = element {
                    let _ = element.set_focus(true).await;
                }
            });
        }
        if *was_open.peek() != is_open {
            was_open.set(is_open);
        }
    });
    let anchor = format!("--{popup_id}");
    let button_class = format!("ix-button ix-button-{} ix-align-center", variant.as_str());
    rsx! { div { class: "ix-date-dropdown", style: "anchor-name:{anchor}",
        onkeydown: move |event| { if event.key() == Key::Escape && open() { event.prevent_default(); event.stop_propagation(); close.call(()); } },
        button { class: button_class, r#type: "button", disabled: disabled || loading,
            aria_expanded: open(), aria_controls: popup_id.clone(), aria_haspopup: "dialog", aria_busy: loading,
            onmounted: move |event| trigger.set(Some(event.data())),
            onclick: move |_| { if open() { close.call(()); } else { open.set(true); } },
            if loading { crate::Spinner { size: crate::SpinnerSize::Sm } }
            else { crate::Icon { name: "history".to_string(), size: 24 } }
            span { "{button_label}" }
            crate::Icon { name: "chevron-down-small".to_string(), size: 24 }
        }
        if open() && !disabled && !loading {
            button { class: "ix-temporal-backdrop", r#type: "button", tabindex: -1, aria_label: "Close date ranges", onclick: move |_| close.call(()) }
            div { class: "ix-temporal-popover ix-date-dropdown-popover", style: "position-anchor:{anchor}", role: "dialog", id: popup_id.clone(), aria_label: "Date range",
                div { class: "ix-date-dropdown-content",
                    if options.len() > 1 { div { class: "ix-date-preset-list",
                        for option in options {
                            button { key: "{option.id}", class: "ix-date-preset", r#type: "button",
                                aria_label: format!("{}: {} to {}", option.label, format.format(option.from), format.format(option.to)),
                                aria_pressed: draft_id() == option.id,
                                disabled: option.from > option.to || !option.from.within(min_date, max_date) || !option.to.within(min_date, max_date),
                                onclick: move |_| {
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
                        div { class: "ix-date-card-footer", crate::Button { onclick: move |_| done.call(()), "{labels.done}" } }
                    }
                }
            }
        }
    } }
}

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-date-time-card{display:inline-flex;flex-direction:column;vertical-align:top;min-width:280px;max-width:100%;background:var(--ix-color-surface-1);color:var(--ix-color-text);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);overflow:hidden;font:14px/20px var(--ix-font)}
.ix-theme .ix-date-embedded{border:0;border-radius:0;background:transparent}.ix-theme .ix-date-corners-straight{border-radius:0}.ix-theme .ix-date-corners-left{border-top-right-radius:0;border-bottom-right-radius:0}.ix-theme .ix-date-corners-right{border-top-left-radius:0;border-bottom-left-radius:0}
.ix-theme .ix-date-card-header{padding:8px;border-bottom:1px solid var(--ix-color-border)}.ix-theme .ix-date-card-content{padding:12px}.ix-theme .ix-date-no-padding>.ix-date-card-content{padding:0}.ix-theme .ix-date-card-footer{padding:12px;display:flex;justify-content:flex-end;gap:8px;border-top:1px solid var(--ix-color-border)}
.ix-theme .ix-calendar-header{display:flex;align-items:center;justify-content:space-between;gap:4px}.ix-theme .ix-calendar-header select,.ix-theme .ix-calendar-header input{height:32px;background:transparent;color:var(--ix-color-text);border:0;min-width:0;font:600 14px/20px var(--ix-font);padding:4px}.ix-theme .ix-calendar-month{flex:1;max-width:140px}.ix-theme .ix-calendar-year{width:70px}.ix-theme .ix-calendar-header option{background:var(--ix-color-surface-2);color:var(--ix-color-text)}
.ix-theme .ix-calendar-row{display:grid;grid-template-columns:repeat(var(--ix-calendar-cols,7),minmax(28px,1fr));gap:2px}.ix-theme .ix-calendar-grid:has(.ix-calendar-week-number){--ix-calendar-cols:8}.ix-theme .ix-calendar-day,.ix-theme .ix-calendar-empty,.ix-theme .ix-calendar-weekday,.ix-theme .ix-calendar-week-number{display:flex;align-items:center;justify-content:center;min-width:28px;min-height:32px;font:14px/20px var(--ix-font);font-variant-numeric:tabular-nums}.ix-theme .ix-calendar-weekday{font-size:12px;color:var(--ix-color-text-secondary);text-overflow:ellipsis;overflow:hidden}.ix-theme .ix-calendar-week-number{color:var(--ix-color-text-secondary);font-size:12px}
.ix-theme .ix-calendar-day{background:transparent;color:var(--ix-color-text);border:1px solid transparent;border-radius:var(--ix-radius);padding:4px;cursor:pointer}.ix-theme .ix-calendar-day:hover:not(:disabled){background:var(--ix-color-surface-2)}.ix-theme .ix-calendar-day.ix-calendar-today{border-color:var(--ix-color-primary);font-weight:600}.ix-theme .ix-calendar-day.ix-calendar-in-range{background:color-mix(in srgb,var(--ix-color-primary) 15%,transparent);border-radius:0}.ix-theme .ix-calendar-day.ix-calendar-selected{background:var(--ix-color-primary);color:var(--ix-color-on-primary);font-weight:600}.ix-theme .ix-calendar-day:disabled{opacity:.35;cursor:default}.ix-theme .ix-calendar-day:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}
.ix-theme .ix-clock-header{display:flex;align-items:center;justify-content:space-between;gap:16px;min-height:32px;font-weight:600}.ix-theme .ix-clock-meridiem{display:flex;gap:2px}.ix-theme .ix-clock-meridiem .ix-button{min-width:40px;padding-inline:6px}.ix-theme .ix-clock-columns{display:flex;gap:8px;min-width:200px}.ix-theme .ix-clock-column{flex:1;min-width:48px}.ix-theme .ix-clock-column-heading{text-align:center;color:var(--ix-color-text-secondary);font-size:12px;margin-bottom:8px}.ix-theme .ix-clock-options{display:flex;flex-direction:column;max-height:200px;overflow-y:auto;overscroll-behavior:contain;scrollbar-width:thin;padding:2px}.ix-theme .ix-clock-option{display:block;width:100%;min-height:32px;border:1px solid transparent;border-radius:var(--ix-radius);background:transparent;color:var(--ix-color-text);font:14px/20px var(--ix-font);font-variant-numeric:tabular-nums;padding:5px 8px;text-align:center;cursor:pointer}.ix-theme .ix-clock-option:hover:not(:disabled){background:var(--ix-color-surface-2)}.ix-theme .ix-clock-option.ix-clock-selected{background:var(--ix-color-primary);color:var(--ix-color-on-primary)}.ix-theme .ix-clock-option:disabled{opacity:.3;cursor:default}.ix-theme .ix-clock-option:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}
.ix-theme .ix-datetime-picker{display:inline-flex;flex-direction:column;max-width:100%;background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.ix-theme .ix-datetime-pickers{display:flex;flex-wrap:wrap}.ix-theme .ix-datetime-pickers>.ix-date-time-card+*{border-left:1px solid var(--ix-color-border)}
.ix-theme .ix-temporal-field{display:flex;flex-direction:column;gap:6px;position:relative;min-width:200px;max-width:100%;font:14px/20px var(--ix-font)}.ix-theme .ix-temporal-field>label{font-weight:600}.ix-theme .ix-temporal-control{display:flex;align-items:center;min-height:var(--ix-control-height,40px);background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.ix-theme .ix-temporal-control:focus-within{border-color:var(--ix-color-primary);outline:1px solid var(--ix-color-primary)}.ix-theme .ix-temporal-control input{flex:1;min-width:0;width:100%;height:100%;padding:8px 12px;border:0;border-radius:inherit;background:transparent;color:var(--ix-color-text);font:14px/20px var(--ix-font);font-variant-numeric:tabular-nums;outline:none}.ix-theme .ix-temporal-control input::placeholder{color:var(--ix-color-text-secondary);opacity:1}.ix-theme .ix-temporal-control input:read-only{cursor:default}.ix-theme .ix-temporal-toggle{display:flex;align-items:center;justify-content:center;align-self:stretch;min-width:36px;background:transparent;border:0;color:var(--ix-color-primary);cursor:pointer;border-radius:var(--ix-radius)}.ix-theme .ix-temporal-toggle:hover:not(:disabled){background:var(--ix-color-surface-2)}.ix-theme .ix-temporal-helper{font-size:12px;color:var(--ix-color-text-secondary)}.ix-theme .ix-temporal-invalid .ix-temporal-control{border-color:var(--ix-color-danger-text)}.ix-theme .ix-temporal-invalid .ix-temporal-helper{color:var(--ix-color-danger-text)}.ix-theme .ix-temporal-disabled .ix-temporal-control{opacity:.5}.ix-theme .ix-temporal-disabled input,.ix-theme .ix-temporal-disabled button{cursor:default}
.ix-theme .ix-temporal-backdrop{position:fixed;inset:0;z-index:70;border:0;background:transparent;cursor:default}.ix-theme .ix-temporal-popover{position:absolute;left:0;top:calc(100% + 4px);z-index:71;max-width:min(660px,calc(100vw - 32px));background:var(--ix-color-surface-1);border-radius:var(--ix-radius);box-shadow:0 6px 24px #0004}.ix-theme .ix-temporal-popover>.ix-date-time-card{border:0}.ix-theme .ix-date-dropdown{display:inline-block;position:relative}.ix-theme .ix-date-preset-list{display:flex;flex-direction:column;padding:4px;min-width:200px}.ix-theme .ix-date-preset{border:0;background:transparent;color:var(--ix-color-text);font:14px/20px var(--ix-font);padding:8px 12px;text-align:start;border-radius:var(--ix-radius);cursor:pointer}.ix-theme .ix-date-preset:hover:not(:disabled){background:var(--ix-color-surface-2)}.ix-theme .ix-date-preset:disabled{opacity:.4;cursor:default}
.ix-theme .ix-temporal-field>label{font-weight:400}.ix-theme .ix-temporal-slot{display:flex;align-items:center;flex:none;padding-inline:8px;color:var(--ix-color-text-secondary)}.ix-theme .ix-temporal-info .ix-temporal-control{border-color:var(--ix-color-info-text)}.ix-theme .ix-temporal-info .ix-temporal-helper{color:var(--ix-color-info-text)}.ix-theme .ix-temporal-warning .ix-temporal-control{border-color:var(--ix-color-warning-text)}.ix-theme .ix-temporal-warning .ix-temporal-helper{color:var(--ix-color-warning-text)}.ix-theme .ix-temporal-valid .ix-temporal-control{border-color:var(--ix-color-success)}.ix-theme .ix-temporal-valid .ix-temporal-helper{color:var(--ix-color-success)}.ix-theme .ix-date-dropdown-popover{width:max-content;max-height:calc(100dvh - 32px);overflow:auto}.ix-theme .ix-date-dropdown-content{display:flex;align-items:stretch;max-width:100%}.ix-theme .ix-date-dropdown-content>.ix-date-preset-list{border-inline-end:1px solid var(--ix-color-border);min-width:160px}.ix-theme .ix-date-dropdown-calendar{min-width:0}.ix-theme .ix-date-preset[aria-pressed=true]{color:var(--ix-color-primary);background:var(--ix-color-surface-2)}
@media(max-width:600px){.ix-theme .ix-datetime-pickers{flex-direction:column}.ix-theme .ix-datetime-pickers>.ix-date-time-card+*{border-left:0;border-top:1px solid var(--ix-color-border)}.ix-theme .ix-temporal-popover{max-width:calc(100vw - 24px)}}
@media(max-width:480px){.ix-theme .ix-date-dropdown-content{flex-direction:column}.ix-theme .ix-date-dropdown-content>.ix-date-preset-list{border-inline-end:0;border-bottom:1px solid var(--ix-color-border);max-height:144px;overflow:auto}}
.ix-theme .ix-temporal-popover{position:fixed;inset:50% auto auto 50%;transform:translate(-50%,-50%);margin:0;box-sizing:border-box;width:max-content;height:max-content;max-width:min(660px,calc(100vw - 32px));max-height:calc(100dvh - 32px);overflow:auto}
@supports (position-anchor:--example) and (position-area:bottom span-right){
 .ix-theme .ix-temporal-popover{inset:auto;transform:none;position-area:bottom span-right;margin:4px;position-try-fallbacks:flip-inline,flip-block,flip-inline flip-block,--ix-temporal-contained}
 @position-try --ix-temporal-contained{position-area:none;inset:16px;margin:auto;height:max-content;max-width:calc(100% - 32px);max-height:calc(100dvh - 32px)}
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

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
        node: &VNode,
        marker: &str,
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
                    if attribute.name == "onclick" {
                        if let dioxus::dioxus_core::AttributeValue::Listener(listener) = &attribute.value { return Some(listener.clone()); }
                    }
                }
            }
        }
        node.dynamic_node_values()
            .iter()
            .find_map(|node| match node {
                dioxus::dioxus_core::DynamicNode::Fragment(children) => children
                    .iter()
                    .find_map(|node| dropdown_listener(node, marker)),
                _ => None,
            })
    }
    fn click_dropdown(dom: &mut VirtualDom, marker: &str) {
        let root = dom.get_scope(ScopeId::APP).unwrap().root_node();
        let listener = dropdown_listener(root, marker).unwrap_or_else(|| {
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
        dom.render_immediate_to_vec();
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
        assert_eq!(
            completed.select(earlier, true),
            DateSelection {
                from: Some(earlier),
                to: None
            }
        );
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
}
