//! Host-owned text parsing and formatting for the temporal inputs.
use crate::{IxDate, IxDateTime, IxTime};
use dioxus::prelude::Callback;

/// A pure, synchronous text codec for a typed local date or time.
///
/// Construct its callbacks in a Dioxus component (for example, with
/// `use_callback`) so they observe current host settings. Change the codec's
/// [`revision`](Self::revision) with [`with_revision`](Self::with_revision) when
/// those settings change the meaning of stable callbacks. Replacing a callback
/// closure alone does not notify Dioxus to refresh unchanged input text.
/// The application owns
/// locale, pattern syntax and any parsing library. A rejected parse is final;
/// inputs never retry a preset parser when this codec is supplied. Invalid raw
/// text remains editable and is still delivered through `onvaluechange`.
///
/// `format` output must parse back to the same typed value. Do not mutate state
/// or perform I/O from either callback: parsing also runs while rendering to
/// derive validation and picker selection. Custom text does not change picker
/// unit layout, labels or typed constraint values.
pub struct TemporalTextFormat<T> {
    pub parse: Callback<String, Option<T>>,
    pub format: Callback<T, String>,
    /// Host configuration revision, included in prop equality. Defaults to zero
    /// in [`Self::new`]; change it when stable callbacks acquire new semantics.
    pub revision: u64,
}
impl<T> TemporalTextFormat<T> {
    pub const fn new(parse: Callback<String, Option<T>>, format: Callback<T, String>) -> Self {
        Self {
            parse,
            format,
            revision: 0,
        }
    }
    /// Refresh render-derived validity and picker preparation after a host
    /// locale, pattern or parser configuration change, without changing text.
    pub const fn with_revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }
}
impl<T> Copy for TemporalTextFormat<T> {}
impl<T> Clone for TemporalTextFormat<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> PartialEq for TemporalTextFormat<T> {
    fn eq(&self, other: &Self) -> bool {
        self.revision == other.revision && self.parse == other.parse && self.format == other.format
    }
}
/// Host-owned text codec for [`crate::DateInput`].
pub type DateTextFormat = TemporalTextFormat<IxDate>;
/// Host-owned text codec for [`crate::TimeInput`].
pub type TimeTextFormat = TemporalTextFormat<IxTime>;
/// Host-owned text codec for [`crate::DateTimeInput`].
pub type DateTimeTextFormat = TemporalTextFormat<IxDateTime>;

pub(crate) fn parse_text<T: 'static>(
    value: &str,
    custom: Option<TemporalTextFormat<T>>,
    preset: impl FnOnce(&str) -> Option<T>,
) -> Option<T> {
    match custom {
        Some(custom) => custom.parse.call(value.to_string()),
        None => preset(value),
    }
}
pub(crate) fn format_text<T: 'static>(
    value: T,
    custom: Option<TemporalTextFormat<T>>,
    preset: impl FnOnce(T) -> String,
) -> String {
    match custom {
        Some(custom) => custom.format.call(value),
        None => preset(value),
    }
}
