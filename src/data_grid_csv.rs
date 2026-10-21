//! Owned CSV snapshots of committed grid data, independent of DOM virtualization.
use crate::{DataGridColumn, DataGridRow, DataGridState, DataGridValueType};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::fmt;

/// Rows included in a CSV snapshot. Every scope retains current typed sorting
/// and managed row order; selection is resolved using stable row IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataGridCsvRowScope {
    /// All matching rows, including those outside the rendered viewport.
    #[default]
    Displayed,
    /// Selected rows that also match the current filters.
    SelectedDisplayed,
    /// Selected rows regardless of the current filters.
    SelectedAll,
    /// All supplied rows regardless of filters or selection.
    All,
}

/// Spreadsheet interpretation policy. Ordinary CSV quoting only preserves CSV
/// structure; quoting alone does not prevent a spreadsheet evaluating formulas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataGridCsvFormulaPolicy {
    /// Prefix potentially formula-like text/header values with an apostrophe.
    /// Finite values in explicitly numeric columns retain their committed text.
    #[default]
    Escape,
    /// Preserve raw committed values, for consumers that need exact data and
    /// handle spreadsheet formula interpretation themselves.
    Preserve,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataGridCsvOptions {
    pub row_scope: DataGridCsvRowScope,
    pub include_headers: bool,
    /// Optional stable column IDs. This filters the current column order;
    /// unknown/repeated IDs are ignored. `None` includes all logical columns.
    pub columns: Option<Vec<String>>,
    pub formula_policy: DataGridCsvFormulaPolicy,
}
impl Default for DataGridCsvOptions {
    fn default() -> Self {
        Self {
            row_scope: DataGridCsvRowScope::Displayed,
            include_headers: true,
            columns: None,
            formula_policy: DataGridCsvFormulaPolicy::Escape,
        }
    }
}

/// Fully owned snapshot. Later row edits, filters, selection or reorderings do
/// not modify it. `content` uses comma separators and CRLF records without a BOM;
/// the downloadable UTF-8 file includes a BOM for spreadsheet compatibility.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataGridCsv {
    pub content: String,
    pub row_ids: Vec<String>,
    pub column_ids: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataGridCsvError {
    Renderer(String),
}
impl fmt::Display for DataGridCsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Renderer(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for DataGridCsvError {}
impl DataGridCsv {
    /// Dispatch the host's normal download flow. Success reports dispatch, not
    /// a confirmed file save; the browser/WebView owns its download policy.
    pub async fn download(&self, file_name: &str) -> Result<(), DataGridCsvError> {
        crate::native_download::download(
            &self.data_url(),
            &crate::native_download::file_name(file_name, "csv", "grid"),
        )
        .await
        .map_err(DataGridCsvError::Renderer)
    }
    fn data_url(&self) -> String {
        let mut bytes = Vec::with_capacity(3 + self.content.len());
        bytes.extend_from_slice(b"\xef\xbb\xbf");
        bytes.extend_from_slice(self.content.as_bytes());
        format!(
            "data:text/csv;charset=utf-8;base64,{}",
            STANDARD.encode(bytes)
        )
    }
}
impl DataGridState {
    /// Export committed raw cells, using the same typed row projection and
    /// logical column order as the grid. No viewport, cell renderer, unsaved
    /// editor draft or DOM value participates. An empty column subset produces
    /// empty content while retaining the projected row IDs in the snapshot.
    pub fn export_csv(
        &self,
        rows: &[DataGridRow],
        columns: &[DataGridColumn],
        options: &DataGridCsvOptions,
    ) -> DataGridCsv {
        let ordered_columns: Vec<_> = self
            .ordered_columns(columns)
            .into_iter()
            .filter(|column| {
                options
                    .columns
                    .as_ref()
                    .is_none_or(|subset| subset.contains(&column.id))
            })
            .collect();
        let projected = match options.row_scope {
            DataGridCsvRowScope::All | DataGridCsvRowScope::SelectedAll => {
                let mut unfiltered = self.clone();
                unfiltered.filters.clear();
                unfiltered.visible_rows_with_columns(rows, columns)
            }
            _ => self.visible_rows_with_columns(rows, columns),
        };
        let selected_only = matches!(
            options.row_scope,
            DataGridCsvRowScope::SelectedDisplayed | DataGridCsvRowScope::SelectedAll
        );
        let projected: Vec<_> = projected
            .into_iter()
            .filter(|row| !selected_only || self.selected.contains(&row.id))
            .collect();
        let mut content = String::new();
        if !ordered_columns.is_empty() {
            if options.include_headers {
                record(
                    &mut content,
                    ordered_columns
                        .iter()
                        .map(|column| (column.title.as_str(), DataGridValueType::Text)),
                    options.formula_policy,
                );
            }
            for row in &projected {
                record(
                    &mut content,
                    ordered_columns
                        .iter()
                        .map(|column| (row.value(&column.id), column.value_type)),
                    options.formula_policy,
                );
            }
        }
        DataGridCsv {
            content,
            row_ids: projected.iter().map(|row| row.id.clone()).collect(),
            column_ids: ordered_columns
                .iter()
                .map(|column| column.id.clone())
                .collect(),
        }
    }
}
fn record<'a>(
    output: &mut String,
    fields: impl Iterator<Item = (&'a str, DataGridValueType)>,
    policy: DataGridCsvFormulaPolicy,
) {
    for (index, (value, value_type)) in fields.enumerate() {
        if index > 0 {
            output.push(',');
        }
        let finite_number = value_type == DataGridValueType::Number
            && value.trim().parse::<f64>().is_ok_and(f64::is_finite);
        let escape =
            policy == DataGridCsvFormulaPolicy::Escape && !finite_number && formula_like(value);
        let quoted = value.contains([',', '"', '\r', '\n']);
        if quoted {
            output.push('"');
        }
        if escape {
            output.push('\'');
        }
        for character in value.chars() {
            if character == '"' {
                output.push('"');
            }
            output.push(character);
        }
        if quoted {
            output.push('"');
        }
    }
    output.push_str("\r\n");
}
fn formula_like(value: &str) -> bool {
    value.trim_start().starts_with(['=', '+', '-', '@'])
        || value
            .chars()
            .take_while(|character| character.is_whitespace())
            .any(|character| matches!(character, '\t' | '\r' | '\n'))
}

#[cfg(test)]
#[path = "data_grid_csv_tests.rs"]
mod tests;
