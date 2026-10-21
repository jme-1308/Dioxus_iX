//! Pure controlled grouping and explicit host-page group summaries.
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
/// Full matching descendants of a host-page group, including unloaded and
/// collapsed records. Accept these summaries atomically with the host page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataGridPageGroup {
    pub path: DataGridGroupPath,
    pub count: usize,
    pub selected_count: usize,
}
/// Host-owned selection request. The application owns global selection policy
/// and accepts updated rows, state and summaries; the grid never fetches members.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataGridPageGroupSelection {
    pub path: DataGridGroupPath,
    pub checked: bool,
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
    /// Summarize this explicitly supplied full client model in grouped display
    /// order, after the current typed query. Includes collapsed/off-page leaves
    /// and current stable-ID selection; this helper never fetches host records.
    pub fn page_group_metadata(
        &self,
        rows: &[DataGridRow],
        columns: &[DataGridColumn],
    ) -> Vec<DataGridPageGroup> {
        let mut positions = BTreeMap::new();
        let mut summaries: Vec<DataGridPageGroup> = Vec::new();
        let grouping_columns = self.grouping.columns();
        for row in self.visible_rows_with_columns(rows, columns) {
            let mut path = DataGridGroupPath::default();
            for column in &grouping_columns {
                path.0.push(((*column).into(), row.value(column).into()));
                let index = *positions.entry(path.clone()).or_insert_with(|| {
                    let index = summaries.len();
                    summaries.push(DataGridPageGroup {
                        path: path.clone(),
                        count: 0,
                        selected_count: 0,
                    });
                    index
                });
                summaries[index].count += 1;
                summaries[index].selected_count += usize::from(self.selected.contains(&row.id));
            }
        }
        summaries
    }
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
/// Validate the supplied page itself without refiltering, sorting or clustering
/// it. Off-page summaries are allowed; every loaded ancestor must be covered.
pub(crate) fn validate_page_groups(
    rows: &[DataGridRow],
    grouping: &DataGridGrouping,
    summaries: &[DataGridPageGroup],
) -> Result<BTreeMap<DataGridGroupPath, DataGridPageGroup>, &'static str> {
    let mut metadata = BTreeMap::new();
    for summary in summaries {
        if !grouping.accepts(&summary.path) {
            return Err("Host group metadata contains an invalid path.");
        }
        if summary.selected_count > summary.count {
            return Err("Host group selection exceeds its row count.");
        }
        if metadata
            .insert(summary.path.clone(), summary.clone())
            .is_some()
        {
            return Err("Host group metadata contains duplicate paths.");
        }
    }
    let columns = grouping.columns();
    let mut last = vec![None::<DataGridGroupPath>; columns.len()];
    let mut seen = BTreeSet::new();
    let mut counts = BTreeMap::<DataGridGroupPath, usize>::new();
    for row in rows {
        let mut path = DataGridGroupPath::default();
        for (depth, column) in columns.iter().enumerate() {
            path.0.push(((*column).into(), row.value(column).into()));
            if last[depth].as_ref() != Some(&path) {
                if !seen.insert(path.clone()) {
                    return Err("Host page groups must be contiguous.");
                }
                last[depth] = Some(path.clone());
            }
            *counts.entry(path.clone()).or_default() += 1;
        }
    }
    for (path, loaded) in counts {
        let summary = metadata
            .get(&path)
            .ok_or("Host group metadata is missing a loaded ancestor.")?;
        if summary.count < loaded {
            return Err("Host group count is smaller than its loaded records.");
        }
    }
    Ok(metadata)
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

/// Expanded visual-slot intervals over the complete logical projection. A slot
/// is present on a page exactly when its descendant interval overlaps it.
/// Prefix counts make each candidate window's cost constant-time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GroupAutoCapacity {
    starts: Vec<usize>,
    ends: Vec<usize>,
    total_rows: usize,
}
impl GroupAutoCapacity {
    pub(crate) fn new(rows: &[DataGridRow], grouping: &DataGridGrouping) -> Self {
        let total_rows = rows.len();
        let mut starts = vec![0usize; total_rows + 1];
        let mut ends = vec![0usize; total_rows + 1];
        for slot in group_slots(rows, grouping, 0..total_rows).0 {
            let range = match slot {
                GroupSlot::Group { range, .. } => range,
                GroupSlot::Record { row, .. } => row..row + 1,
            };
            starts[range.start] += 1;
            ends[range.end] += 1;
        }
        for index in 1..=total_rows {
            starts[index] += starts[index - 1];
            ends[index] += ends[index - 1];
        }
        Self {
            starts,
            ends,
            total_rows,
        }
    }
    /// A fixed logical size that fits *every* contiguous window, rather than
    /// only the current page alignment. This conservative bound cannot oscillate
    /// when the owner accepts a different page size. One record remains the
    /// minimum when even its ancestry exceeds the measured slot budget.
    pub(crate) fn page_size(&self, budget: usize) -> usize {
        if self.total_rows == 0 {
            return budget.max(1);
        }
        let fits = |size: usize| {
            (0..=self.total_rows - size)
                .all(|start| self.starts[start + size - 1] - self.ends[start] <= budget)
        };
        let mut low = 1;
        let mut high = self.total_rows;
        while low < high {
            let middle = low + (high - low).div_ceil(2);
            if fits(middle) {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        low
    }
}
