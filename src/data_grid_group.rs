//! Pure controlled grouping for client-side grid rows.
use crate::{DataGridColumn, DataGridRow, DataGridState};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

/// A group identity is its complete column/value ancestry, independent of row IDs.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataGridGroupPath(pub Vec<(String, String)>);
impl DataGridGroupPath {
    pub fn new<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self(
            pairs
                .into_iter()
                .map(|(column, value)| (column.into(), value.into()))
                .collect(),
        )
    }
    pub fn as_slice(&self) -> &[(String, String)] {
        &self.0
    }
    pub(crate) fn key(&self) -> String {
        self.0
            .iter()
            .map(|(column, value)| format!("{}:{column}{}:{value}", column.len(), value.len()))
            .collect()
    }
}
/// Group by raw cell values in these columns. Duplicate column IDs use their
/// first occurrence; missing/blank values form an explicit empty-string group.
/// Collapsed paths hide presentation rows, never filtered selection or CSV data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataGridGrouping {
    pub columns: Vec<String>,
    pub collapsed: BTreeSet<DataGridGroupPath>,
}
impl DataGridGrouping {
    pub fn new<I, S>(columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            columns: columns.into_iter().map(Into::into).collect(),
            collapsed: BTreeSet::new(),
        }
    }
    pub(crate) fn columns(&self) -> Vec<&str> {
        let mut seen = BTreeSet::new();
        self.columns
            .iter()
            .map(String::as_str)
            .filter(|column| seen.insert(*column))
            .collect()
    }
    fn accepts(&self, path: &DataGridGroupPath) -> bool {
        let columns = self.columns();
        !path.0.is_empty()
            && path.0.len() <= columns.len()
            && path
                .0
                .iter()
                .zip(columns)
                .all(|((column, _), expected)| column.as_str() == expected)
    }
    pub(crate) fn row_visible(&self, row: &DataGridRow) -> bool {
        let mut path = DataGridGroupPath::default();
        for column in self.columns() {
            path.0.push((column.into(), row.value(column).into()));
            if self.collapsed.contains(&path) {
                return false;
            }
        }
        true
    }
}
/// Cluster an already filtered/stably sorted model in first-encounter group order.
pub(crate) fn cluster_rows<'a>(
    rows: Vec<&'a DataGridRow>,
    grouping: &DataGridGrouping,
) -> Vec<&'a DataGridRow> {
    fn cluster<'a>(rows: Vec<&'a DataGridRow>, columns: &[&str]) -> Vec<&'a DataGridRow> {
        let Some((column, tail)) = columns.split_first() else {
            return rows;
        };
        let mut indexes = BTreeMap::new();
        let mut buckets: Vec<Vec<&DataGridRow>> = Vec::new();
        for row in rows {
            let next = buckets.len();
            let index = *indexes.entry(row.value(column)).or_insert(next);
            if index == buckets.len() {
                buckets.push(Vec::new());
            }
            buckets[index].push(row);
        }
        buckets
            .into_iter()
            .flat_map(|rows| cluster(rows, tail))
            .collect()
    }
    cluster(rows, &grouping.columns())
}
impl DataGridState {
    /// All matching group paths in display order, including collapsed groups.
    pub fn group_paths(
        &self,
        rows: &[DataGridRow],
        columns: &[DataGridColumn],
    ) -> Vec<DataGridGroupPath> {
        let mut seen = BTreeSet::new();
        let mut paths = Vec::new();
        let grouping_columns = self.grouping.columns();
        for row in self.visible_rows_with_columns(rows, columns) {
            let mut path = DataGridGroupPath::default();
            for column in &grouping_columns {
                path.0.push(((*column).into(), row.value(column).into()));
                if seen.insert(path.clone()) {
                    paths.push(path.clone());
                }
            }
        }
        paths
    }
    /// Full filtered descendants of a valid path, including collapsed/off-page leaves.
    /// Empty paths and paths that do not follow current grouping columns have
    /// no members; they never mean the whole dataset.
    pub fn group_rows<'a>(
        &self,
        rows: &'a [DataGridRow],
        columns: &[DataGridColumn],
        path: &DataGridGroupPath,
    ) -> Vec<&'a DataGridRow> {
        if !self.grouping.accepts(path) {
            return Vec::new();
        }
        self.visible_rows_with_columns(rows, columns)
            .into_iter()
            .filter(|row| {
                path.0
                    .iter()
                    .all(|(column, value)| row.value(column) == value.as_str())
            })
            .collect()
    }
    pub fn group_expanded(&self, path: &DataGridGroupPath) -> bool {
        self.grouping.accepts(path) && !self.grouping.collapsed.contains(path)
    }
    pub fn set_group_expanded(&mut self, path: &DataGridGroupPath, expanded: bool) {
        if !self.grouping.accepts(path) {
            return;
        }
        if expanded {
            self.grouping.collapsed.remove(path);
        } else {
            self.grouping.collapsed.insert(path.clone());
        }
    }
    pub fn expand_all_groups(&mut self) {
        self.grouping.collapsed.clear();
    }
    pub fn collapse_all_groups(&mut self, rows: &[DataGridRow], columns: &[DataGridColumn]) {
        self.grouping.collapsed = self.group_paths(rows, columns).into_iter().collect();
    }
    pub fn group_selection(
        &self,
        rows: &[DataGridRow],
        columns: &[DataGridColumn],
        path: &DataGridGroupPath,
    ) -> (bool, bool) {
        let members = self.group_rows(rows, columns, path);
        let selected = members
            .iter()
            .filter(|row| self.selected.contains(&row.id))
            .count();
        (
            !members.is_empty() && selected == members.len(),
            selected > 0 && selected < members.len(),
        )
    }
    pub fn select_group(
        &mut self,
        rows: &[DataGridRow],
        columns: &[DataGridColumn],
        path: &DataGridGroupPath,
        checked: bool,
    ) {
        let ids: Vec<_> = self
            .group_rows(rows, columns, path)
            .into_iter()
            .map(|row| row.id.clone())
            .collect();
        for id in ids {
            if checked {
                self.selected.insert(id);
            } else {
                self.selected.remove(&id);
            }
        }
    }
    /// Current logical page's expanded leaves. Pagination totals and CSV still
    /// use every matching logical record, independent of collapsed groups.
    pub fn expanded_page_rows_with_columns<'a>(
        &self,
        rows: &'a [DataGridRow],
        columns: &[DataGridColumn],
    ) -> Vec<&'a DataGridRow> {
        self.paged_rows_with_columns(rows, columns)
            .into_iter()
            .filter(|row| self.grouping.row_visible(row))
            .collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GroupSlot {
    Group {
        path: DataGridGroupPath,
        count: usize,
        aria_index: usize,
        range: Range<usize>,
    },
    Record {
        row: usize,
        aria_index: usize,
    },
}
/// Derive visual slots from the complete grouped logical model. Page ancestry
/// uses full descendant counts; global ARIA positions count expanded tree rows.
pub(crate) fn group_slots(
    rows: &[DataGridRow],
    grouping: &DataGridGrouping,
    page: Range<usize>,
) -> (Vec<GroupSlot>, usize) {
    struct Walk<'a> {
        rows: &'a [DataGridRow],
        columns: Vec<&'a str>,
        grouping: &'a DataGridGrouping,
        page: Range<usize>,
        slots: Vec<GroupSlot>,
        total: usize,
    }
    impl Walk<'_> {
        fn walk(&mut self, range: Range<usize>, depth: usize, parent: &DataGridGroupPath) {
            if depth == self.columns.len() {
                for row in range {
                    self.total = self.total.saturating_add(1);
                    if self.page.contains(&row) {
                        self.slots.push(GroupSlot::Record {
                            row,
                            aria_index: self.total.saturating_add(1),
                        });
                    }
                }
                return;
            }
            let column = self.columns[depth];
            let mut start = range.start;
            while start < range.end {
                let value = self.rows[start].value(column);
                let mut end = start + 1;
                while end < range.end && self.rows[end].value(column) == value {
                    end += 1;
                }
                let mut path = parent.clone();
                path.0.push((column.into(), value.into()));
                self.total = self.total.saturating_add(1);
                if start < self.page.end && end > self.page.start {
                    self.slots.push(GroupSlot::Group {
                        path: path.clone(),
                        count: end - start,
                        aria_index: self.total.saturating_add(1),
                        range: start..end,
                    });
                }
                if !self.grouping.collapsed.contains(&path) {
                    self.walk(start..end, depth + 1, &path);
                }
                start = end;
            }
        }
    }
    let mut walk = Walk {
        rows,
        columns: grouping.columns(),
        grouping,
        page,
        slots: Vec::new(),
        total: 0,
    };
    walk.walk(0..rows.len(), 0, &DataGridGroupPath::default());
    (walk.slots, walk.total)
}
