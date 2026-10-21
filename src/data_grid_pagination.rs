//! Controlled page metadata and client or host-owned row projection.
use crate::{DataGridColumn, DataGridRow, DataGridState};
use std::ops::Range;

/// Source of the grid's accepted row projection.
///
/// `Page` receives one already filtered, sorted and ordered page. Supply
/// `DataGridState::pagination`; `total_rows` is the full matching result count.
/// Accept query state and its rows together. The application owns fetching,
/// pending/error presentation, stale-response rejection and cancellation.
/// Selection remains stable-ID based across pages; header selection and
/// autosizing cover only the loaded page. Managed row dragging is unavailable.
/// Grouped pages additionally supply full matching group summaries; group
/// selection and CSV/global member access remain application-owned. Treegrid
/// positions are page-local, while pagination announces global logical rows.
/// Client-side projection and CSV helpers remain client-side operations on
/// their explicitly supplied rows; they do not fetch unloaded host pages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataGridRowModel {
    #[default]
    Client,
    Page {
        total_rows: usize,
    },
}
impl DataGridRowModel {
    pub(crate) fn projected_rows<'a>(
        self,
        state: &DataGridState,
        rows: &'a [DataGridRow],
        columns: &[DataGridColumn],
    ) -> Vec<&'a DataGridRow> {
        match self {
            Self::Client => state.expanded_page_rows_with_columns(rows, columns),
            Self::Page { .. } => self
                .selection_rows(state, rows, columns)
                .into_iter()
                .filter(|row| state.grouping.row_visible(row))
                .collect(),
        }
    }
    pub(crate) fn selection_rows<'a>(
        self,
        state: &DataGridState,
        rows: &'a [DataGridRow],
        columns: &[DataGridColumn],
    ) -> Vec<&'a DataGridRow> {
        match self {
            Self::Client => state.visible_rows_with_columns(rows, columns),
            Self::Page { total_rows } => state.pagination.map_or_else(Vec::new, |request| {
                rows.iter()
                    .take(request.info(total_rows).range.len())
                    .collect()
            }),
        }
    }
}

/// Controlled zero-based page and requested page size. Invalid sizes and pages
/// are clamped when projected; the consumer's stored request is not rewritten.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataGridPagination {
    pub page: usize,
    pub page_size: usize,
}
impl Default for DataGridPagination {
    fn default() -> Self {
        Self::new(0, 100)
    }
}
impl DataGridPagination {
    pub const fn new(page: usize, page_size: usize) -> Self {
        Self { page, page_size }
    }
    pub fn info(self, total_rows: usize) -> DataGridPage {
        let page_size = self.page_size.max(1);
        let page_count = total_rows.div_ceil(page_size);
        let page = self.page.min(page_count.saturating_sub(1));
        let start = page.saturating_mul(page_size).min(total_rows);
        DataGridPage {
            page,
            page_size,
            total_rows,
            page_count,
            range: start..start.saturating_add(page_size).min(total_rows),
        }
    }
}

/// Effective page of the full filtered/sorted row model. Empty results have
/// page zero, zero pages and range `0..0`; `range` is zero-based and end-exclusive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataGridPage {
    pub page: usize,
    pub page_size: usize,
    pub total_rows: usize,
    pub page_count: usize,
    pub range: Range<usize>,
}
impl DataGridState {
    /// Resolve a page without mutating controlled state. Without pagination the
    /// entire result is one page (or zero pages when empty).
    pub fn pagination_info(&self, total_rows: usize) -> DataGridPage {
        self.pagination
            .unwrap_or_else(|| DataGridPagination::new(0, total_rows.max(1)))
            .info(total_rows)
    }
    /// Current page after typed filtering, sorting and managed row ordering.
    /// `visible_rows_with_columns` deliberately remains the full projection for
    /// filtered selection, CSV export and autosizing across pages.
    pub fn paged_rows_with_columns<'a>(
        &self,
        rows: &'a [DataGridRow],
        columns: &[DataGridColumn],
    ) -> Vec<&'a DataGridRow> {
        let visible = self.visible_rows_with_columns(rows, columns);
        let page = self.pagination_info(visible.len());
        visible
            .into_iter()
            .skip(page.range.start)
            .take(page.range.len())
            .collect()
    }
    pub(crate) fn reset_pagination_page(&mut self) {
        if let Some(pagination) = &mut self.pagination {
            pagination.page = 0;
        }
    }
}

// A real scroll-surface measurement includes the fixed 48px header. Keep the
// first effective logical row within the resized page; owners still accept the
// resulting controlled request before the public projection changes.
pub(crate) fn auto_page_request(
    pagination: DataGridPagination,
    total_rows: usize,
    measured_height: f64,
    row_height: u32,
) -> Option<DataGridPagination> {
    let page_size = auto_page_capacity(measured_height, row_height)?;
    auto_page_request_for_size(pagination, total_rows, page_size)
}
pub(crate) fn auto_page_capacity(measured_height: f64, row_height: u32) -> Option<usize> {
    if !measured_height.is_finite() || measured_height <= 0.0 {
        return None;
    }
    Some(
        ((measured_height - 48.0) / f64::from(row_height.max(24)))
            .floor()
            .max(1.0) as usize,
    )
}
pub(crate) fn auto_page_request_for_size(
    pagination: DataGridPagination,
    total_rows: usize,
    page_size: usize,
) -> Option<DataGridPagination> {
    let page_size = page_size.max(1);
    if pagination.page_size == page_size {
        return None;
    }
    let first = pagination.info(total_rows).range.start;
    let effective = DataGridPagination::new(first / page_size, page_size).info(total_rows);
    Some(DataGridPagination::new(effective.page, page_size))
}

// Each loaded record can require all of its ancestors. This conservative bound
// does not change when the owner loads a different page or collapses groups.
pub(crate) fn host_group_page_size(slot_budget: usize, group_depth: usize) -> usize {
    (slot_budget / group_depth.saturating_add(1)).max(1)
}
