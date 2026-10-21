//! Pure client-side page projection over the grid's full typed row model.
use crate::{DataGridColumn, DataGridRow, DataGridState};
use std::ops::Range;

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
