//! A controlled Rust data grid for the iX AG Grid example's community features.
//!
//! This implements typed sorting/filtering, filtered multi-selection, column
//! resizing, content autosizing and stable-ID row/column movement. It is an independent Dioxus component;
//! it does not embed the AG Grid JavaScript engine or implement its plugin API.
use crate::data_grid_columns::{ColumnTrack, column_widths, column_window};
use crate::{
    DataGridCellContext, DataGridCellEditor, DataGridCellRender, DataGridEditorContext,
    DataGridValueType, Icon,
};
use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

static GRID_ID: AtomicUsize = AtomicUsize::new(1);

/// A string-valued column. IDs must remain unique and stable when rows change.
#[derive(Clone, Debug, PartialEq)]
pub struct DataGridColumn {
    pub id: String,
    pub title: String,
    pub sortable: bool,
    pub filterable: bool,
    /// Interpretation for sorting and filtering, independent of the editor kind.
    pub value_type: DataGridValueType,
    pub resizable: bool,
    /// Allow header dragging and Alt+Left / Alt+Right column movement.
    pub reorderable: bool,
    pub row_drag: bool,
    /// Opt in to built-in editing with controlled string change requests.
    pub editable: bool,
    /// Shared required and maximum-length constraints, applied before kind validation.
    pub editor: DataGridTextEditor,
    /// Native editor kind. Text is the backward-compatible default.
    pub cell_editor: DataGridCellEditor,
    /// Minimum width in CSS pixels, excluding the separate selection column.
    pub min_width: u32,
    pub max_width: u32,
}
impl DataGridColumn {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            sortable: false,
            filterable: false,
            value_type: DataGridValueType::Text,
            resizable: true,
            reorderable: true,
            row_drag: false,
            editable: false,
            editor: DataGridTextEditor::default(),
            cell_editor: DataGridCellEditor::default(),
            min_width: 80,
            max_width: 2000,
        }
    }
    fn clamp_width(&self, width: f64) -> u32 {
        let min = self.min_width.max(24);
        if !width.is_finite() {
            return min;
        }
        width
            .round()
            .clamp(min as f64, self.max_width.max(min) as f64) as u32
    }
}

/// One stable-keyed record. Missing column values render as empty strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataGridRow {
    pub id: String,
    pub cells: BTreeMap<String, String>,
    /// Per-column tooltip overrides. Without an override the cell's text is used.
    pub tooltips: BTreeMap<String, String>,
    /// Stable column IDs that remain read-only even in editable columns.
    pub read_only_cells: BTreeSet<String>,
}
impl DataGridRow {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            cells: BTreeMap::new(),
            tooltips: BTreeMap::new(),
            read_only_cells: BTreeSet::new(),
        }
    }
    pub fn cell(mut self, column: impl Into<String>, value: impl Into<String>) -> Self {
        self.cells.insert(column.into(), value.into());
        self
    }
    pub fn tooltip(mut self, column: impl Into<String>, text: impl Into<String>) -> Self {
        self.tooltips.insert(column.into(), text.into());
        self
    }
    pub fn value(&self, column: &str) -> &str {
        self.cells
            .get(column)
            .map(String::as_str)
            .unwrap_or_default()
    }
}

/// Constraints for the independent Rust text editor. Length counts Unicode
/// scalar values. Invalid edits remain open with an accessible error message.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataGridTextEditor {
    pub required: bool,
    pub max_length: Option<usize>,
}
impl DataGridTextEditor {
    pub fn validate(&self, value: &str) -> Result<(), String> {
        if self.required && value.trim().is_empty() {
            return Err("Enter a value before saving.".into());
        }
        if let Some(max) = self.max_length {
            if value.chars().count() > max {
                return Err(format!("Use {max} characters or fewer before saving."));
            }
        }
        Ok(())
    }
}

/// An accepted edit request. Consumers own the rows and apply accepted changes.
/// Cancellation, invalid input and unchanged input do not emit this event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataGridCellValueChanged {
    pub row_id: String,
    pub column_id: String,
    pub old_value: String,
    pub new_value: String,
}
impl DataGridCellValueChanged {
    /// Apply by stable IDs without overwriting a concurrent update. Returns
    /// false for a removed record, read-only cell or conflicting old value.
    pub fn apply_to(&self, rows: &mut [DataGridRow]) -> bool {
        let Some(row) = rows.iter_mut().find(|row| row.id == self.row_id) else {
            return false;
        };
        if row.read_only_cells.contains(&self.column_id)
            || row.value(&self.column_id) != self.old_value
        {
            return false;
        }
        row.cells
            .insert(self.column_id.clone(), self.new_value.clone());
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GridEditSession {
    id: usize,
    row: String,
    column: String,
    original: String,
    draft: String,
    error: Option<String>,
    select_all: bool,
}
#[derive(Clone)]
enum GridEditAction {
    Start {
        row: String,
        column: String,
        initial: Option<String>,
    },
    Cancel,
    Commit {
        tab: Option<bool>,
        restore_focus: bool,
    },
}
fn editable_cell(row: &DataGridRow, column: &DataGridColumn, read_only: bool) -> bool {
    !read_only && column.editable && !row.read_only_cells.contains(&column.id)
}
fn next_editable_cell(
    rows: &[DataGridRow],
    columns: &[DataGridColumn],
    state: &DataGridState,
    row: &str,
    column: &str,
    backwards: bool,
) -> Option<(String, String)> {
    let displayed = state.paged_rows_with_columns(rows, columns);
    let columns = state.ordered_columns(columns);
    let current_row = displayed.iter().position(|item| item.id == row)?;
    let current_column = columns.iter().position(|item| item.id == column)?;
    let current = current_row * columns.len() + current_column;
    let cells = displayed.len().checked_mul(columns.len())?;
    let indices: Box<dyn Iterator<Item = usize>> = if backwards {
        Box::new((0..current).rev())
    } else {
        Box::new(current.saturating_add(1)..cells)
    };
    for index in indices {
        let row = displayed[index / columns.len()];
        let column = columns[index % columns.len()];
        if editable_cell(row, column, false) {
            return Some((row.id.clone(), column.id.clone()));
        }
    }
    None
}
fn finish_edit(
    session: &GridEditSession,
    rows: &[DataGridRow],
    columns: &[DataGridColumn],
    read_only: bool,
) -> Result<Option<DataGridCellValueChanged>, String> {
    let row = rows
        .iter()
        .find(|row| row.id == session.row)
        .ok_or("This row was removed. Cancel this edit.")?;
    let column = columns
        .iter()
        .find(|column| column.id == session.column)
        .ok_or("This column was removed. Cancel this edit.")?;
    if !editable_cell(row, column, read_only) {
        return Err("This cell is read-only. Cancel this edit.".into());
    }
    if row.value(&session.column) != session.original {
        return Err("This value changed while editing. Cancel and edit the latest value.".into());
    }
    column.editor.validate(&session.draft)?;
    column.cell_editor.validate(&session.draft)?;
    Ok(
        (session.draft != session.original).then(|| DataGridCellValueChanged {
            row_id: session.row.clone(),
            column_id: session.column.clone(),
            old_value: session.original.clone(),
            new_value: session.draft.clone(),
        }),
    )
}

/// Filter operations for text and explicitly numeric columns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataGridFilterOperator {
    #[default]
    Contains,
    NotContains,
    Equals,
    NotEqual,
    StartsWith,
    EndsWith,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Blank,
    NotBlank,
}
impl DataGridFilterOperator {
    const ALL: [Self; 8] = [
        Self::Contains,
        Self::NotContains,
        Self::Equals,
        Self::NotEqual,
        Self::StartsWith,
        Self::EndsWith,
        Self::Blank,
        Self::NotBlank,
    ];
    const NUMBER: [Self; 8] = [
        Self::Equals,
        Self::NotEqual,
        Self::LessThan,
        Self::LessThanOrEqual,
        Self::GreaterThan,
        Self::GreaterThanOrEqual,
        Self::Blank,
        Self::NotBlank,
    ];
    fn choices(value_type: DataGridValueType) -> &'static [Self] {
        match value_type {
            DataGridValueType::Text => &Self::ALL,
            DataGridValueType::Number => &Self::NUMBER,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Contains => "Contains",
            Self::NotContains => "Does not contain",
            Self::Equals => "Equals",
            Self::NotEqual => "Does not equal",
            Self::StartsWith => "Begins with",
            Self::EndsWith => "Ends with",
            Self::LessThan => "Less than",
            Self::LessThanOrEqual => "Less than or equal",
            Self::GreaterThan => "Greater than",
            Self::GreaterThanOrEqual => "Greater than or equal",
            Self::Blank => "Blank",
            Self::NotBlank => "Not blank",
        }
    }
    fn token(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::NotContains => "notContains",
            Self::Equals => "equals",
            Self::NotEqual => "notEqual",
            Self::StartsWith => "startsWith",
            Self::EndsWith => "endsWith",
            Self::LessThan => "lessThan",
            Self::LessThanOrEqual => "lessThanOrEqual",
            Self::GreaterThan => "greaterThan",
            Self::GreaterThanOrEqual => "greaterThanOrEqual",
            Self::Blank => "blank",
            Self::NotBlank => "notBlank",
        }
    }
    fn from_token(value: &str) -> Self {
        Self::ALL
            .into_iter()
            .chain(Self::NUMBER)
            .find(|operator| operator.token() == value)
            .unwrap_or_default()
    }
    fn needs_value(self) -> bool {
        !matches!(self, Self::Blank | Self::NotBlank)
    }
}

/// An empty value has no effect except with `Blank` or `NotBlank`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataGridFilterCondition {
    pub operator: DataGridFilterOperator,
    pub value: String,
}
impl DataGridFilterCondition {
    pub fn is_active(&self) -> bool {
        !self.operator.needs_value() || !self.value.is_empty()
    }
    /// Match using the backward-compatible case-insensitive text interpretation.
    pub fn matches(&self, value: &str) -> bool {
        self.matches_with_type(value, DataGridValueType::Text)
    }
    /// Match using the column's explicit interpretation without changing stored values.
    pub fn matches_with_type(&self, value: &str, value_type: DataGridValueType) -> bool {
        value_type.matches(value, self)
    }
}

/// Two filter conditions combine with AND by default, or OR when `match_any` is true.
/// The historical name is retained; `matches_with_type` also supports numeric columns.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataGridTextFilter {
    pub first: DataGridFilterCondition,
    pub second: DataGridFilterCondition,
    pub match_any: bool,
}
impl DataGridTextFilter {
    pub fn is_active(&self) -> bool {
        self.first.is_active() || self.second.is_active()
    }
    /// Match using the backward-compatible text interpretation.
    pub fn matches(&self, value: &str) -> bool {
        self.matches_with_type(value, DataGridValueType::Text)
    }
    pub fn matches_with_type(&self, value: &str, value_type: DataGridValueType) -> bool {
        match (self.first.is_active(), self.second.is_active()) {
            (false, false) => true,
            (true, false) => self.first.matches_with_type(value, value_type),
            (false, true) => self.second.matches_with_type(value, value_type),
            (true, true) if self.match_any => {
                self.first.matches_with_type(value, value_type)
                    || self.second.matches_with_type(value, value_type)
            }
            (true, true) => {
                self.first.matches_with_type(value, value_type)
                    && self.second.matches_with_type(value, value_type)
            }
        }
    }
}

/// Ascending / descending / unsorted is the default header activation cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataGridSortDirection {
    Ascending,
    Descending,
}

/// All persistent state is owned by the consumer. Selection uses row IDs rather
/// than displayed indices, so filtering, sorting and reordering preserve identity.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataGridState {
    /// Optional client pagination. Selection, CSV export and autosizing retain
    /// the full filtered model; keyboard/edit navigation uses the current page.
    pub pagination: Option<crate::DataGridPagination>,
    pub selected: BTreeSet<String>,
    /// Most recently selected row, used by Shift+checkbox range selection.
    pub selection_anchor: Option<String>,
    pub sort: Option<(String, DataGridSortDirection)>,
    pub filters: BTreeMap<String, DataGridTextFilter>,
    /// IDs absent here follow the rows' input order. Stale IDs are ignored.
    pub row_order: Vec<String>,
    /// Stable column IDs in display order. Unknown and repeated IDs are ignored;
    /// newly supplied columns follow in their input order.
    pub column_order: Vec<String>,
    /// User-resized widths. Other columns share the available grid width equally.
    pub column_widths: BTreeMap<String, u32>,
}
impl DataGridState {
    pub fn ordered_columns<'a>(&self, columns: &'a [DataGridColumn]) -> Vec<&'a DataGridColumn> {
        let mut seen = BTreeSet::new();
        self.column_order
            .iter()
            .filter_map(|id| columns.iter().find(|column| column.id == *id))
            .chain(columns.iter())
            .filter(|column| seen.insert(column.id.as_str()))
            .collect()
    }
    /// Move a column relative to another without changing sort, widths or row state.
    pub fn move_column(
        &mut self,
        columns: &[DataGridColumn],
        moving: &str,
        target: &str,
        after: bool,
    ) -> bool {
        if moving == target
            || !columns
                .iter()
                .any(|column| column.id == moving && column.reorderable)
        {
            return false;
        }
        let mut ids: Vec<_> = self
            .ordered_columns(columns)
            .into_iter()
            .map(|column| column.id.clone())
            .collect();
        let before = ids.clone();
        let Some(index) = ids.iter().position(|id| id == moving) else {
            return false;
        };
        if !ids.iter().any(|id| id == target) {
            return false;
        }
        let moving = ids.remove(index);
        let target = ids
            .iter()
            .position(|id| id == target)
            .expect("distinct target remains");
        ids.insert(target + usize::from(after), moving);
        if ids == before {
            return false;
        }
        self.column_order = ids;
        true
    }
    /// Legacy text projection. Use `visible_rows_with_columns` for typed columns.
    pub fn visible_rows<'a>(&self, rows: &'a [DataGridRow]) -> Vec<&'a DataGridRow> {
        self.visible_rows_with_columns(rows, &[])
    }
    /// Filter and stably sort by the supplied columns' value types. Unknown column
    /// IDs retain text semantics. Equal values retain the managed/input row order.
    pub fn visible_rows_with_columns<'a>(
        &self,
        rows: &'a [DataGridRow],
        columns: &[DataGridColumn],
    ) -> Vec<&'a DataGridRow> {
        let value_type = |id: &str| {
            columns
                .iter()
                .find(|column| column.id == id)
                .map(|column| column.value_type)
                .unwrap_or_default()
        };
        let mut visible: Vec<_> = rows
            .iter()
            .filter(|row| {
                self.filters.iter().all(|(column, filter)| {
                    filter.matches_with_type(row.value(column), value_type(column))
                })
            })
            .collect();
        let ranks: BTreeMap<_, _> = self
            .row_order
            .iter()
            .enumerate()
            .map(|(rank, id)| (id.as_str(), rank))
            .collect();
        visible.sort_by_key(|row| ranks.get(row.id.as_str()).copied().unwrap_or(usize::MAX));
        if let Some((column, direction)) = &self.sort {
            let value_type = value_type(column);
            visible
                .sort_by(|a, b| value_type.compare(a.value(column), b.value(column), *direction));
        }
        visible
    }
    pub fn cycle_sort(&mut self, column: &str) {
        self.reset_pagination_page();
        self.sort = match &self.sort {
            Some((current, DataGridSortDirection::Ascending)) if current == column => {
                Some((column.into(), DataGridSortDirection::Descending))
            }
            Some((current, DataGridSortDirection::Descending)) if current == column => None,
            _ => Some((column.into(), DataGridSortDirection::Ascending)),
        };
    }
    /// Select or deselect visible rows. Hidden selections remain unchanged.
    pub fn select_filtered(&mut self, rows: &[DataGridRow], checked: bool) {
        self.select_filtered_with_columns(rows, &[], checked);
    }
    /// Select the typed filtered result, preserving selections hidden by filters.
    pub fn select_filtered_with_columns(
        &mut self,
        rows: &[DataGridRow],
        columns: &[DataGridColumn],
        checked: bool,
    ) {
        let ids: Vec<_> = self
            .visible_rows_with_columns(rows, columns)
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
    /// Select a visible row, or the displayed range from the previous anchor.
    /// A missing or filtered-out anchor starts a new range at this row.
    pub fn select_row(
        &mut self,
        rows: &[DataGridRow],
        id: &str,
        checked: bool,
        extend_range: bool,
    ) {
        self.select_row_with_columns(rows, &[], id, checked, extend_range);
    }
    /// Select a row or range using the typed, displayed row order.
    pub fn select_row_with_columns(
        &mut self,
        rows: &[DataGridRow],
        columns: &[DataGridColumn],
        id: &str,
        checked: bool,
        extend_range: bool,
    ) {
        let visible = self.visible_rows_with_columns(rows, columns);
        let Some(index) = visible.iter().position(|row| row.id == id) else {
            return;
        };
        let anchor = self
            .selection_anchor
            .as_deref()
            .and_then(|anchor| visible.iter().position(|row| row.id == anchor));
        let from = if extend_range {
            anchor.unwrap_or(index)
        } else {
            index
        };
        let ids: Vec<_> = visible[from.min(index)..=from.max(index)]
            .iter()
            .map(|row| row.id.clone())
            .collect();
        for id in ids {
            if checked {
                self.selected.insert(id);
            } else {
                self.selected.remove(&id);
            }
        }
        if !extend_range || anchor.is_none() {
            self.selection_anchor = Some(id.into());
        }
    }
    /// `(all, mixed)` describes only the currently visible rows. An empty result
    /// is unchecked and cannot be selected by the header control.
    pub fn filtered_selection(&self, rows: &[DataGridRow]) -> (bool, bool) {
        self.filtered_selection_with_columns(rows, &[])
    }
    /// `(all, mixed)` for the typed filtered result, excluding hidden selections.
    pub fn filtered_selection_with_columns(
        &self,
        rows: &[DataGridRow],
        columns: &[DataGridColumn],
    ) -> (bool, bool) {
        let visible = self.visible_rows_with_columns(rows, columns);
        let checked = visible
            .iter()
            .filter(|row| self.selected.contains(&row.id))
            .count();
        (
            !visible.is_empty() && checked == visible.len(),
            checked > 0 && checked < visible.len(),
        )
    }
    /// Managed row movement is disabled during sorting or active filtering,
    /// matching the source configuration's `rowDragManaged` restriction.
    pub fn can_reorder(&self) -> bool {
        self.sort.is_none() && !self.filters.values().any(DataGridTextFilter::is_active)
    }
    /// Move one row before/after another by stable ID. Returns whether order changed.
    pub fn move_row(
        &mut self,
        rows: &[DataGridRow],
        moving: &str,
        target: &str,
        after: bool,
    ) -> bool {
        if !self.can_reorder() || moving == target {
            return false;
        }
        let mut ids: Vec<_> = self
            .visible_rows(rows)
            .into_iter()
            .map(|row| row.id.clone())
            .collect();
        let before = ids.clone();
        let Some(index) = ids.iter().position(|id| id == moving) else {
            return false;
        };
        if !ids.iter().any(|id| id == target) {
            return false;
        }
        let moving = ids.remove(index);
        let target = ids
            .iter()
            .position(|id| id == target)
            .expect("target remains after removing distinct ID");
        ids.insert(target + usize::from(after), moving);
        if ids == before {
            return false;
        }
        self.row_order = ids;
        true
    }
    pub fn resize_column(&mut self, column: &DataGridColumn, width: f64) {
        if column.resizable {
            self.column_widths
                .insert(column.id.clone(), column.clamp_width(width));
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ResizeSession {
    id: usize,
    column: DataGridColumn,
    start_x: f64,
    current_x: f64,
    width: u32,
    start_width: f64,
    measured: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
struct GridViewport {
    top: f64,
    height: f64,
}

fn grid_window(
    total: usize,
    viewport: GridViewport,
    row_height: u32,
    buffer: usize,
) -> std::ops::Range<usize> {
    let row_height = f64::from(row_height.max(24));
    let height = (viewport.height - 48.0).max(row_height);
    let top = if viewport.top.is_finite() {
        viewport
            .top
            .clamp(0.0, (total as f64 * row_height - height).max(0.0))
    } else {
        0.0
    };
    let first = (top / row_height).floor() as usize;
    let last = ((top + height) / row_height).ceil() as usize;
    first.saturating_sub(buffer)..last.saturating_add(buffer).min(total)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GridLane {
    Selection,
    Header(String),
    Cell(String),
    Drag(String),
    Editor(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct GridFocus {
    row: String,
    lane: GridLane,
}

/// One stable validation target per grid, outside the horizontally scrolling
/// cell plane. Editors can reference it without passing IDs through every row.
#[derive(Clone)]
struct GridValidationContext {
    id: String,
}

/// One pending Dioxus focus intent, consumed by its live row/editor mount.
/// `None` targets the mounted Header checkbox, never an arbitrary DOM query.
#[derive(Clone)]
struct GridFocusRequest {
    serial: u64,
    target: Option<GridFocus>,
    top: Option<f64>,
    claimed: bool,
}
#[derive(Clone, Default)]
struct GridMountState {
    serial: u64,
    pending: Option<GridFocusRequest>,
    active: Option<GridFocus>,
    origin: Option<String>,
    applying: bool,
}
/// One last-focused native header/composed control and one native filter.
/// Conservative retention survives focusout -> focusin without a DOM focus runtime.
#[derive(Clone, Copy)]
struct GridColumnContext {
    focused: Signal<Option<String>>,
    filter: Signal<Option<String>>,
}

#[derive(Clone, Copy)]
struct GridMountContext {
    root: Signal<Option<Rc<MountedData>>>,
    state: Signal<GridMountState>,
    width: Signal<f64>,
}
impl GridMountContext {
    fn cancel(mut self) {
        if let Ok(mut state) = self.state.try_write() {
            state.serial = state.serial.wrapping_add(1);
            state.pending = None;
            state.origin = None;
            state.applying = false;
        }
    }
    fn request(mut self, target: Option<GridFocus>, top: Option<f64>) {
        let mut state = self.state.write();
        state.serial = state.serial.wrapping_add(1);
        state.origin = state.active.as_ref().map(|active| active.row.clone());
        state.pending = Some(GridFocusRequest {
            serial: state.serial,
            target,
            top,
            claimed: false,
        });
        state.applying = false;
    }
    fn current(self, serial: u64) -> bool {
        self.state.try_peek().is_ok_and(|state| {
            state
                .pending
                .as_ref()
                .is_some_and(|request| request.serial == serial)
        })
    }
    fn focused(mut self, focus: Option<GridFocus>) {
        let keep = self
            .state
            .peek()
            .pending
            .as_ref()
            .is_some_and(|request| request.target == focus);
        if !keep {
            self.cancel();
        }
        let mut state = self.state.write();
        state.active = focus;
        state.origin = None;
    }
    fn blurred(mut self) {
        // A requested mounted focus causes its own normal blur/focus pair.
        // Other blur cancels a waiting target before it can steal outside focus.
        if !self.state.peek().applying {
            self.cancel();
        }
        self.state.write().active = None;
    }
    fn finish(mut self, serial: u64) {
        if self.current(serial) {
            let mut state = self.state.write();
            state.pending = None;
            state.origin = None;
            state.applying = false;
        }
    }
}

fn use_grid_focus_lane(
    focus: Option<GridFocus>,
    selection: Option<(String, bool)>,
) -> EventHandler<MountedEvent> {
    let context = use_context::<GridMountContext>();
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let drop_alive = alive.clone();
    let drop_focus = focus.clone();
    use_drop(move || {
        drop_alive.set(false);
        if context.state.try_peek().is_ok_and(|state| {
            state
                .pending
                .as_ref()
                .is_some_and(|request| request.claimed && request.target == drop_focus)
        }) {
            context.cancel();
        }
    });
    use_effect(use_reactive!(|focus, selection| {
        let Some(request) = context
            .state
            .read()
            .pending
            .clone()
            .filter(|request| !request.claimed && request.target == focus)
        else {
            return;
        };
        let Some(target) = mounted.read().clone() else {
            return;
        };
        let Some(root) = context.root.read().clone() else {
            return;
        };
        let mut context = context;
        context.state.write().pending.as_mut().unwrap().claimed = true;
        let alive = alive.clone();
        let selection = selection.clone();
        spawn(async move {
            let checked_context = context;
            let valid = || alive.get() && checked_context.current(request.serial);
            if !valid() {
                return;
            }
            if let Some(top) = request.top {
                let Ok(offset) = root.get_scroll_offset().await else {
                    context.finish(request.serial);
                    return;
                };
                if !valid() {
                    return;
                }
                let _ = root
                    .scroll(
                        dioxus::html::geometry::PixelsVector2D::new(offset.x, top),
                        dioxus::html::ScrollBehavior::Instant,
                    )
                    .await;
                if !valid() {
                    return;
                }
            }
            context.state.write().applying = true;
            // Pinned desktop may apply setFocus and then fail its bool response
            // decode. Make one mounted call; typed focus/native selection facts
            // confirm its result rather than retrying through JavaScript.
            let _ = target.set_focus(true).await;
            if !valid() {
                return;
            }
            context.state.write().applying = false;
            if let Some((id, select_all)) = selection {
                let _ = crate::input_selection::select_focused_input_text(&id, select_all).await;
                if !valid() {
                    return;
                }
            }
            let Ok(bounds) = root.get_client_rect().await else {
                context.finish(request.serial);
                return;
            };
            if !valid() {
                return;
            }
            let Ok(rect) = target.get_client_rect().await else {
                context.finish(request.serial);
                return;
            };
            if !valid() {
                return;
            }
            let Ok(offset) = root.get_scroll_offset().await else {
                context.finish(request.serial);
                return;
            };
            if !valid() {
                return;
            }
            let width = *context.width.peek();
            let right = bounds.origin.x
                + if width > 0.0 {
                    width
                } else {
                    bounds.size.width
                };
            let adjustment = if rect.origin.x < bounds.origin.x {
                rect.origin.x - bounds.origin.x
            } else if rect.origin.x + rect.size.width > right {
                rect.origin.x + rect.size.width - right
            } else {
                0.0
            };
            if adjustment.abs() > 0.5 {
                let _ = root
                    .scroll(
                        dioxus::html::geometry::PixelsVector2D::new(
                            (offset.x + adjustment).max(0.0),
                            offset.y,
                        ),
                        dioxus::html::ScrollBehavior::Instant,
                    )
                    .await;
            }
            context.finish(request.serial);
        });
    }));
    EventHandler::new(move |event: MountedEvent| mounted.set(Some(event.data())))
}

fn retain_outgoing_grid_editor(session: &GridEditSession, state: &GridMountState) -> bool {
    let own = GridFocus {
        row: session.row.clone(),
        lane: GridLane::Editor(session.column.clone()),
    };
    state.origin.as_deref() == Some(session.row.as_str())
        && state.active.as_ref() == Some(&own)
        && state
            .pending
            .as_ref()
            .is_some_and(|request| request.target.as_ref() != Some(&own))
}

fn grid_focus_target_live(
    target: &GridFocus,
    rows: &[DataGridRow],
    columns: &[DataGridColumn],
    editing: Option<&GridEditSession>,
    suppress_cell_focus: bool,
    can_reorder: bool,
) -> bool {
    if let GridLane::Header(column) = &target.lane {
        return columns.iter().any(|item| item.id == *column);
    }
    if !rows.iter().any(|row| row.id == target.row) {
        return false;
    }
    match &target.lane {
        GridLane::Header(_) => unreachable!("header membership checked above"),
        GridLane::Selection => true,
        GridLane::Cell(column) => {
            !suppress_cell_focus && columns.iter().any(|item| item.id == *column)
        }
        GridLane::Drag(column) => {
            can_reorder
                && columns
                    .iter()
                    .any(|item| item.id == *column && item.row_drag)
        }
        GridLane::Editor(column) => {
            !suppress_cell_focus
                && editing
                    .is_some_and(|session| session.row == target.row && session.column == *column)
                && columns
                    .iter()
                    .any(|item| item.id == *column && item.editable)
        }
    }
}

fn reveal_grid_row(
    context: GridMountContext,
    focus: GridFocus,
    index: usize,
    total: usize,
    row_height: u32,
    mut viewport: Signal<GridViewport>,
    mut focused: Signal<Option<GridFocus>>,
) {
    let mut next = *viewport.peek();
    let height = (next.height - 48.0).max(f64::from(row_height));
    let top = index as f64 * f64::from(row_height);
    next.top = if top < next.top {
        top
    } else if top + f64::from(row_height) > next.top + height {
        top + f64::from(row_height) - height
    } else {
        next.top
    };
    next.top = next.top.clamp(
        0.0,
        (total as f64 * f64::from(row_height) - height).max(0.0),
    );
    context.request(Some(focus.clone()), Some(next.top));
    viewport.set(next);
    focused.set(Some(focus));
}

fn autosize_values(rows: &[DataGridRow], column: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    rows.iter()
        .map(|row| row.value(column))
        .filter(|value| seen.insert((*value).to_string()))
        .map(str::to_string)
        .collect()
}

#[derive(Clone, PartialEq)]
struct GridAutosizeRequest {
    serial: u64,
    column: DataGridColumn,
    values: Vec<String>,
    sort: Option<DataGridSortDirection>,
    filtered: bool,
}
#[derive(Default)]
struct GridAutosizeInputs {
    columns: Vec<DataGridColumn>,
    rows: Rc<Vec<DataGridRow>>,
    state: DataGridState,
}
impl GridAutosizeInputs {
    fn request(&self, id: &str, serial: u64) -> Option<GridAutosizeRequest> {
        let column = self
            .columns
            .iter()
            .find(|column| column.id == id && column.resizable)?
            .clone();
        Some(GridAutosizeRequest {
            serial,
            values: autosize_values(&self.rows, id),
            sort: self
                .state
                .sort
                .as_ref()
                .filter(|(column, _)| column == id)
                .map(|(_, direction)| *direction),
            filtered: self
                .state
                .filters
                .get(id)
                .is_some_and(DataGridTextFilter::is_active),
            column,
        })
    }
    fn current(&self, request: &GridAutosizeRequest) -> bool {
        self.request(&request.column.id, request.serial).as_ref() == Some(request)
    }
}
#[derive(Clone)]
struct GridSeparatorMount {
    node: Rc<MountedData>,
    owner: Rc<Cell<bool>>,
}
#[derive(Clone)]
struct GridAutosizeContext {
    pending: Signal<Option<GridAutosizeRequest>>,
    inputs: Rc<RefCell<GridAutosizeInputs>>,
    separators: Rc<RefCell<BTreeMap<String, GridSeparatorMount>>>,
    serial: Rc<Cell<u64>>,
    alive: Rc<Cell<bool>>,
}
impl GridAutosizeContext {
    fn cancel(&self) {
        self.serial.set(self.serial.get().wrapping_add(1));
        let mut pending = self.pending;
        if let Ok(mut pending) = pending.try_write() {
            *pending = None;
        }
    }
    fn fit(&self, id: &str) {
        self.cancel();
        if !self.alive.get() {
            return;
        }
        let request = self.inputs.borrow().request(id, self.serial.get());
        let mut pending = self.pending;
        pending.set(request);
    }
    fn current(&self, serial: u64) -> bool {
        self.alive.get()
            && self.serial.get() == serial
            && self.pending.try_peek().is_ok_and(|pending| {
                pending
                    .as_ref()
                    .is_some_and(|request| request.serial == serial)
            })
    }
    fn complete(&self, serial: u64, width: Option<f64>, onchange: EventHandler<DataGridState>) {
        if !self.current(serial) {
            return;
        }
        let request = self.pending.peek().clone().unwrap();
        let next = {
            let inputs = self.inputs.borrow();
            width
                .filter(|width| width.is_finite() && *width > 0.0)
                .filter(|_| inputs.current(&request))
                .map(|width| {
                    let mut next = inputs.state.clone();
                    next.resize_column(&request.column, width.ceil());
                    next
                })
        };
        self.cancel();
        if let Some(next) = next {
            onchange.call(next);
        }
    }
    fn fit_at(&self, x: f64, y: f64) {
        self.cancel();
        let serial = self.serial.get();
        let context = self.clone();
        let separators = self
            .separators
            .borrow()
            .iter()
            .map(|(id, mount)| (id.clone(), mount.clone()))
            .collect::<Vec<_>>();
        spawn(async move {
            for (id, mount) in separators {
                if !context.alive.get() || context.serial.get() != serial {
                    return;
                }
                if !mount.owner.get() {
                    continue;
                }
                let Ok(rect) = mount.node.get_client_rect().await else {
                    continue;
                };
                if !context.alive.get() || context.serial.get() != serial {
                    return;
                }
                let current = context.separators.borrow().get(&id).is_some_and(|current| {
                    Rc::ptr_eq(&current.node, &mount.node) && current.owner.get()
                });
                if current
                    && rect.size.width > 0.0
                    && rect.size.height > 0.0
                    && x >= rect.origin.x
                    && x <= rect.origin.x + rect.size.width
                    && y >= rect.origin.y
                    && y <= rect.origin.y + rect.size.height
                {
                    context.fit(&id);
                    return;
                }
            }
        });
    }
}

fn grid_header_title(title: &str, sort: Option<DataGridSortDirection>) -> Element {
    rsx! { "{title}" if let Some(direction) = sort { Icon { name: if direction == DataGridSortDirection::Ascending { "sort-ascending" } else { "sort-descending" }, size:16 } } }
}
fn grid_cell_text(text: &str, intrinsic: bool) -> Element {
    rsx! { span { class:"ix-data-grid-cell-text", style:intrinsic.then_some("width:max-content;min-width:max-content;overflow:visible;flex:none"), "{text}" } }
}

/// Inert, temporary Dioxus markup inherits the grid's font and the same Header/
/// cell classes. Max-content layout measures every unique value, including rows
/// outside the virtual window; one mounted rectangle supplies their maximum.
#[component]
fn GridAutosizeProbe(
    request: GridAutosizeRequest,
    onmeasure: EventHandler<Option<f64>>,
) -> Element {
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let drop_alive = alive.clone();
    use_drop(move || drop_alive.set(false));
    let column = &request.column;
    let title_class = if column.sortable {
        "ix-data-grid-sort"
    } else if column.reorderable {
        "ix-data-grid-header-label ix-data-grid-title-button"
    } else {
        "ix-data-grid-header-label"
    };
    rsx! {
        div { "data-grid-autosize-probe":request.serial, inert:true, "aria-hidden":"true", style:"position:fixed;left:-100000px;top:0;width:max-content;visibility:hidden;pointer-events:none",
            onmounted:move |event| { let node=event.data();let alive=alive.clone();spawn(async move { let width=node.get_client_rect().await.ok().map(|rect|rect.size.width);if alive.get(){onmeasure.call(width);} }); },
            div { class:"ix-data-grid-header", style:"position:static;display:block;height:auto;min-width:0;width:max-content",
                div { class:"ix-data-grid-heading", style:"position:static;min-width:0;width:max-content",
                    div { class:"ix-data-grid-title", style:"width:max-content;min-width:max-content;overflow:visible;flex:none",
                        if column.sortable || column.reorderable { button { class:title_class,r#type:"button",tabindex:"-1",style:"width:max-content;min-width:max-content;overflow:visible;flex:none",{grid_header_title(&column.title,request.sort)} } }
                        else { span { class:title_class,style:"width:max-content;min-width:max-content;overflow:visible;flex:none",{grid_header_title(&column.title,None)} } }
                    }
                    if column.filterable { button {class:"ix-data-grid-filter-trigger",r#type:"button",tabindex:"-1","data-active":request.filtered.to_string(),Icon {name:if request.filtered{"filter-filled"}else{"filter"},size:16}} }
                }
            }
            for value in &request.values {
                div {class:"ix-data-grid-cell",style:"position:static;width:max-content;min-width:0;overflow:visible",
                    if column.row_drag {button {class:"ix-data-grid-drag",r#type:"button",tabindex:"-1",Icon {name:"drag-gripper",size:16}}}
                    {grid_cell_text(value,true)}
                }
            }
        }
    }
}

fn grid_template(
    columns: &[DataGridColumn],
    state: &DataGridState,
    resizing: Option<&ResizeSession>,
) -> String {
    let mut tracks = vec!["48px".to_string()];
    tracks.extend(columns.iter().map(|column| {
        resizing
            .filter(|session| session.measured && session.column.id == column.id)
            .map(|session| session.width)
            .or_else(|| state.column_widths.get(&column.id).copied())
            .map(|width| format!("{}px", column.clamp_width(width as f64)))
            .unwrap_or_else(|| format!("minmax({}px,1fr)", column.min_width.max(24)))
    }));
    tracks.join(" ")
}

fn grid_min_width(
    columns: &[DataGridColumn],
    state: &DataGridState,
    resizing: Option<&ResizeSession>,
) -> u64 {
    48 + columns
        .iter()
        .map(|column| {
            resizing
                .filter(|session| session.measured && session.column.id == column.id)
                .map(|session| session.width)
                .or_else(|| state.column_widths.get(&column.id).copied())
                .map(|width| column.clamp_width(width as f64))
                .unwrap_or_else(|| column.min_width.max(24)) as u64
        })
        .sum::<u64>()
}

/// A scrollable, controlled data grid. With `suppress_cell_focus: true`, as in
/// the pinned iX examples, cells do not receive focus. Row checkboxes support
/// Up/Down, Home/End and Page Up/Down across unmounted rows. The default
/// `suppress_cell_focus: false` gives cells the same navigation and Left/Right.
/// Resize with a pointer or Left/Right on a separator (Shift uses 25px steps).
/// Move a row with its drag handle, or Alt+Up / Alt+Down on that handle.
/// Drag column titles or use Alt+Left / Alt+Right on a title to reorder columns.
/// Double-click a separator (or press Enter on it) to fit its header and all
/// currently filtered rows, including rows outside the scroll viewport.
#[component]
pub fn DataGrid(
    columns: Vec<DataGridColumn>,
    rows: Vec<DataGridRow>,
    state: DataGridState,
    onstatechange: EventHandler<DataGridState>,
    /// Accepted edits are requests; apply them to the controlled `rows`.
    /// Updated rows are sorted/filtered again. Enter/F2, double-click and
    /// printable characters start text/number editing. Choice and checkbox
    /// editors use Enter/F2 or double-click. Escape discards, Enter saves,
    /// Tab/Shift+Tab saves and edits the next/previous editable cell.
    /// At the first/last editable cell, Tab finishes editing in that cell.
    #[props(default)]
    oncellvaluechanged: EventHandler<DataGridCellValueChanged>,
    /// Compose visible cells with Dioxus content. Returning `None` retains text;
    /// active editors take priority. Autosizing still measures source text.
    #[props(default)]
    render_cell: Option<Callback<DataGridCellContext, Option<DataGridCellRender>>>,
    /// Compose the active editor with ordinary Dioxus content. Attach its
    /// context's mounted handler to the primary control. `None` preserves the
    /// native editor, including its existing text-selection behavior.
    #[props(default)]
    render_editor: Option<Callback<DataGridEditorContext, Option<Element>>>,
    /// Disable editing while preserving selection and navigation.
    #[props(default)]
    read_only: bool,
    #[props(default)] striped: bool,
    #[props(default = 192)] height: u32,
    /// Fixed row height in CSS pixels (minimum 24).
    #[props(default = 42)]
    row_height: u32,
    /// Rows retained on each side of the viewport, matching AG Grid's default 10.
    #[props(default = 10)]
    row_buffer: usize,
    /// Mount the whole filtered dataset instead of windowing it.
    #[props(default)]
    suppress_row_virtualisation: bool,
    /// Mount all columns instead of windowing the horizontal viewport.
    #[props(default)]
    suppress_column_virtualisation: bool,
    /// Extra columns mounted on either side of the horizontal viewport.
    #[props(default = 2)]
    column_buffer: usize,
    #[props(default)] suppress_cell_focus: bool,
    #[props(default = 500)] tooltip_delay_ms: u32,
    #[props(default = "Data grid".into())] label: String,
    #[props(default)] class: String,
    #[props(default)] pagination_labels: crate::PaginationLabels,
    #[props(default = vec![10,25,50,100])] page_size_options: Vec<usize>,
    #[props(default)] hide_pagination_controls: bool,
    /// Request a controlled page size that fits the measured scroll surface.
    /// Active edits defer requests until saved/canceled. Disabling auto keeps
    /// the current controlled size; no manual preference is stored internally.
    #[props(default)]
    auto_page_size: bool,
) -> Element {
    let mut editing = use_signal(|| None::<GridEditSession>);
    let dragging = use_signal(|| None::<String>);
    let drop_target = use_signal(|| None::<String>);
    let column_dragging = use_signal(|| None::<String>);
    let column_drop_target = use_signal(|| None::<String>);
    let mut resizing = use_signal(|| None::<ResizeSession>);
    let grid_id = use_hook(|| format!("ix-grid-{}", GRID_ID.fetch_add(1, Ordering::Relaxed)));
    let validation = use_context_provider(|| GridValidationContext {
        id: format!("{grid_id}-edit-error"),
    });
    let scroll_id = format!("{grid_id}-scroll");
    let row_height = row_height.max(24);
    let mut viewport = use_signal(|| GridViewport {
        top: 0.0,
        height: f64::from(height),
    });
    let mut focused = use_signal(|| None::<GridFocus>);
    let mounted_root = use_signal(|| None::<Rc<MountedData>>);
    let mount_state = use_signal(GridMountState::default);
    let client_width = use_signal(|| 0.0_f64);
    let mut scroll_left = use_signal(|| 0.0_f64);
    let column_focus = use_signal(|| None::<String>);
    let mut column_filter = use_signal(|| None::<String>);
    use_context_provider(|| GridColumnContext {
        focused: column_focus,
        filter: column_filter,
    });
    let mut focus_context = use_context_provider(|| GridMountContext {
        root: mounted_root,
        state: mount_state,
        width: client_width,
    });
    let header_mount = use_grid_focus_lane(None, None);
    use_drop(move || focus_context.cancel());

    let fit_request = use_signal(|| None::<GridAutosizeRequest>);
    let autosize = use_context_provider(|| GridAutosizeContext {
        pending: fit_request,
        inputs: Rc::new(RefCell::new(GridAutosizeInputs::default())),
        separators: Rc::new(RefCell::new(BTreeMap::new())),
        serial: Rc::new(Cell::new(0)),
        alive: Rc::new(Cell::new(true)),
    });
    let drop_autosize = autosize.clone();
    use_drop(move || {
        drop_autosize.alive.set(false);
        drop_autosize.cancel();
        drop_autosize.separators.borrow_mut().clear();
    });
    let row_label_column = columns.first().map(|column| column.id.clone());
    let columns: Vec<_> = state
        .ordered_columns(&columns)
        .into_iter()
        .cloned()
        .collect();
    let full_visible = Rc::new(
        state
            .visible_rows_with_columns(&rows, &columns)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
    );
    let full_count = full_visible.len();
    let measured_height = use_hook(|| Rc::new(Cell::new(None::<f64>)));
    let auto_state = state.clone();
    let report_capacity = use_callback(move |measurement: Option<f64>| {
        if let Some(height) = measurement {
            if !height.is_finite() || height <= 0.0 {
                return;
            }
            measured_height.set(Some(height));
        }
        if !auto_page_size || editing.peek().is_some() {
            return;
        }
        let Some((pagination, height)) = auto_state.pagination.zip(measured_height.get()) else {
            return;
        };
        let Some(request) = crate::data_grid_pagination::auto_page_request(
            pagination, full_count, height, row_height,
        ) else {
            return;
        };
        let mut next = auto_state.clone();
        next.pagination = Some(request);
        onstatechange.call(next);
    });
    let edit_active = editing().is_some();
    use_effect(use_reactive(
        (
            &auto_page_size,
            &row_height,
            &state.pagination,
            &full_count,
            &edit_active,
        ),
        move |_| report_capacity.call(None),
    ));
    let page = state.pagination_info(full_count);
    let visible = if state.pagination.is_some() {
        Rc::new(full_visible[page.range.clone()].to_vec())
    } else {
        full_visible.clone()
    };
    let visible_count = visible.len();
    let page_identity = state.pagination.map(|_| (page.page, page.page_size));
    let previous_page = use_hook(|| Rc::new(Cell::new(page_identity)));
    use_effect(use_reactive!(|page_identity| {
        if previous_page.replace(page_identity) == page_identity {
            return;
        }
        focus_context.cancel();
        let mut current = *viewport.peek();
        current.top = 0.0;
        viewport.set(current);
        let node = mounted_root.peek().clone();
        let previous_page = previous_page.clone();
        spawn(async move {
            let Some(node) = node else {
                return;
            };
            let Ok(offset) = node.get_scroll_offset().await else {
                return;
            };
            if previous_page.get() != page_identity
                || !mounted_root.try_peek().is_ok_and(|current| {
                    current
                        .as_ref()
                        .is_some_and(|current| Rc::ptr_eq(current, &node))
                })
            {
                return;
            }
            let _ = node
                .scroll(
                    dioxus::html::geometry::PixelsVector2D::new(offset.x, 0.0),
                    dioxus::html::ScrollBehavior::Instant,
                )
                .await;
        });
    }));
    *autosize.inputs.borrow_mut() = GridAutosizeInputs {
        columns: columns.clone(),
        rows: full_visible.clone(),
        state: state.clone(),
    };
    let stale_fit = autosize
        .pending
        .peek()
        .as_ref()
        .is_some_and(|request| !autosize.inputs.borrow().current(request));
    if stale_fit {
        autosize.cancel();
    }
    let window = if suppress_row_virtualisation {
        0..visible_count
    } else {
        grid_window(visible_count, viewport(), row_height, row_buffer)
    };
    let mut rendered: Vec<_> = window.clone().collect();
    // Keep the currently focused row alive when wheel scrolling moves the
    // viewport away. This adds at most one row and avoids dropping native focus.
    if let Some(index) =
        focused().and_then(|focus| visible.iter().position(|row| row.id == focus.row))
    {
        if !window.contains(&index) {
            rendered.push(index);
            rendered.sort_unstable();
        }
    }
    if let Some(index) =
        editing().and_then(|edit| visible.iter().position(|row| row.id == edit.row))
    {
        if !rendered.contains(&index) {
            rendered.push(index);
            rendered.sort_unstable();
        }
    }
    if let Some(index) = focus_context
        .state
        .read()
        .origin
        .as_ref()
        .and_then(|origin| visible.iter().position(|row| &row.id == origin))
    {
        if !rendered.contains(&index) {
            rendered.push(index);
            rendered.sort_unstable();
        }
    }
    let callback_rows = Rc::new(rows.clone());
    let record_state = Rc::new(state.clone());
    let keyboard_rows = visible.clone();
    let keyboard_columns = columns.clone();
    let navigate = move |(index, key, command, backwards): (usize, Key, bool, bool)| {
        let Some(mut focus) = focused.peek().clone() else {
            return;
        };
        let page = ((viewport.peek().height - 48.0) / f64::from(row_height))
            .floor()
            .max(1.0) as usize;
        let target = match key {
            Key::ArrowUp => index.saturating_sub(1),
            Key::ArrowDown => (index + 1).min(visible_count.saturating_sub(1)),
            Key::Home | Key::End => {
                let end = key == Key::End;
                if matches!(focus.lane, GridLane::Cell(_)) {
                    if let Some(column) = if end {
                        keyboard_columns.last()
                    } else {
                        keyboard_columns.first()
                    } {
                        focus.lane = GridLane::Cell(column.id.clone());
                    }
                }
                if end {
                    visible_count.saturating_sub(1)
                } else {
                    0
                }
            }
            Key::PageUp => index.saturating_sub(page),
            Key::PageDown => index
                .saturating_add(page)
                .min(visible_count.saturating_sub(1)),
            Key::ArrowLeft | Key::ArrowRight => {
                let GridLane::Cell(ref id) = focus.lane else {
                    return;
                };
                let Some(current) = keyboard_columns.iter().position(|column| column.id == *id)
                else {
                    return;
                };
                let next = if key == Key::ArrowLeft {
                    if command {
                        0
                    } else {
                        current.saturating_sub(1)
                    }
                } else if command {
                    keyboard_columns.len().saturating_sub(1)
                } else {
                    (current + 1).min(keyboard_columns.len().saturating_sub(1))
                };
                focus.lane = GridLane::Cell(keyboard_columns[next].id.clone());
                index
            }
            Key::Tab => {
                let GridLane::Cell(ref id) = focus.lane else {
                    return;
                };
                let Some(current) = keyboard_columns.iter().position(|column| column.id == *id)
                else {
                    return;
                };
                let (row, column) = if backwards {
                    if current > 0 {
                        (index, current - 1)
                    } else {
                        (
                            index.saturating_sub(1),
                            keyboard_columns.len().saturating_sub(1),
                        )
                    }
                } else if current + 1 < keyboard_columns.len() {
                    (index, current + 1)
                } else {
                    ((index + 1).min(visible_count.saturating_sub(1)), 0)
                };
                focus.lane = GridLane::Cell(keyboard_columns[column].id.clone());
                row
            }
            _ => return,
        };
        if let Some(row) = keyboard_rows.get(target) {
            focus.row = row.id.clone();
            reveal_grid_row(
                focus_context,
                focus,
                target,
                visible_count,
                row_height,
                viewport,
                focused,
            );
        }
    };
    {
        let ids: Vec<_> = visible.iter().map(|row| row.id.clone()).collect();
        use_effect(use_reactive!(|ids| {
            if focused
                .peek()
                .as_ref()
                .is_some_and(|focus| !ids.contains(&focus.row))
            {
                let owned = focus_context
                    .state
                    .peek()
                    .active
                    .as_ref()
                    .is_some_and(|focus| !ids.contains(&focus.row));
                focused.set(None);
                focus_context.cancel();
                // Typed ownership proves the removed row still owned grid focus.
                // A filter editor/outside blur clears it and cannot be stolen.
                if owned {
                    focus_context.request(None, None);
                }
            }
        }));
    }
    let current_edit = editing.peek().clone();
    if let Some(session) = current_edit {
        if suppress_cell_focus
            || !visible.iter().any(|row| row.id == session.row)
            || !rows
                .iter()
                .find(|row| row.id == session.row)
                .zip(columns.iter().find(|column| column.id == session.column))
                .is_some_and(|(row, column)| editable_cell(row, column, read_only))
        {
            focus_context.cancel();
            editing.set(None);
            let current_focus = focused.peek().clone();
            if current_focus.as_ref().is_some_and(|focus| {
                focus.row == session.row && matches!(focus.lane, GridLane::Editor(_))
            }) {
                focused.set((!suppress_cell_focus).then_some(GridFocus {
                    row: session.row,
                    lane: GridLane::Cell(session.column),
                }));
            }
        }
    }
    let invalid_target = focus_context
        .state
        .peek()
        .pending
        .as_ref()
        .and_then(|request| request.target.as_ref())
        .is_some_and(|target| {
            !grid_focus_target_live(
                target,
                &visible,
                &columns,
                editing.peek().as_ref(),
                suppress_cell_focus,
                state.can_reorder(),
            )
        });
    if invalid_target {
        focus_context.cancel();
    }
    let edit_rows = rows.clone();
    let edit_columns = columns.clone();
    let edit_state = state.clone();
    let edit_action = EventHandler::new(move |action: GridEditAction| {
        let reveal = |session: &GridEditSession, records: &[DataGridRow], editor: bool| {
            let displayed = edit_state.paged_rows_with_columns(records, &edit_columns);
            if let Some(index) = displayed.iter().position(|row| row.id == session.row) {
                reveal_grid_row(
                    focus_context,
                    GridFocus {
                        row: session.row.clone(),
                        lane: if editor {
                            GridLane::Editor(session.column.clone())
                        } else {
                            GridLane::Cell(session.column.clone())
                        },
                    },
                    index,
                    displayed.len(),
                    row_height,
                    viewport,
                    focused,
                );
            }
        };
        match action {
            GridEditAction::Start {
                row,
                column,
                initial,
            } => {
                let Some(record) = edit_rows.iter().find(|record| record.id == row) else {
                    return;
                };
                let Some(config) = edit_columns.iter().find(|config| config.id == column) else {
                    return;
                };
                if suppress_cell_focus || !editable_cell(record, config, read_only) {
                    return;
                }
                let previous = editing.peek().clone();
                if let Some(mut session) = previous {
                    if session.row == row && session.column == column {
                        return;
                    }
                    match finish_edit(&session, &edit_rows, &edit_columns, read_only) {
                        Ok(Some(change)) => oncellvaluechanged.call(change),
                        Ok(None) => {}
                        Err(error) => {
                            session.error = Some(error);
                            reveal(&session, &edit_rows, true);
                            editing.set(Some(session));
                            return;
                        }
                    }
                }
                let original = record.value(&column).to_string();
                let session = GridEditSession {
                    id: GRID_ID.fetch_add(1, Ordering::Relaxed),
                    row,
                    column,
                    select_all: initial.is_none(),
                    draft: initial.unwrap_or_else(|| original.clone()),
                    original,
                    error: None,
                };
                reveal(&session, &edit_rows, true);
                editing.set(Some(session));
            }
            GridEditAction::Cancel => {
                let current = editing.peek().clone();
                if let Some(session) = current {
                    editing.set(None);
                    reveal(&session, &edit_rows, false);
                }
            }
            GridEditAction::Commit { tab, restore_focus } => {
                let Some(mut session) = editing.peek().clone() else {
                    return;
                };
                match finish_edit(&session, &edit_rows, &edit_columns, read_only) {
                    Err(error) => {
                        session.error = Some(error);
                        reveal(&session, &edit_rows, true);
                        editing.set(Some(session));
                    }
                    Ok(change) => {
                        let next = tab.and_then(|backwards| {
                            next_editable_cell(
                                &edit_rows,
                                &edit_columns,
                                &edit_state,
                                &session.row,
                                &session.column,
                                backwards,
                            )
                        });
                        let mut projected = edit_rows.clone();
                        if let Some(change) = change {
                            change.apply_to(&mut projected);
                            oncellvaluechanged.call(change);
                        }
                        editing.set(None);
                        if let Some((row, column)) = next {
                            if let Some(record) = projected.iter().find(|record| record.id == row) {
                                if edit_state
                                    .paged_rows_with_columns(&projected, &edit_columns)
                                    .iter()
                                    .any(|record| record.id == row)
                                {
                                    let original = record.value(&column).to_string();
                                    let next = GridEditSession {
                                        id: GRID_ID.fetch_add(1, Ordering::Relaxed),
                                        row,
                                        column,
                                        original: original.clone(),
                                        draft: original,
                                        error: None,
                                        select_all: true,
                                    };
                                    reveal(&next, &projected, true);
                                    editing.set(Some(next));
                                }
                            }
                        } else if restore_focus {
                            reveal(&session, &projected, false);
                        }
                    }
                }
            }
        }
    });
    let (checked, mixed) = state.filtered_selection_with_columns(&rows, &columns);
    let template = grid_template(&columns, &state, resizing.read().as_ref());
    let min_width = grid_min_width(&columns, &state, resizing.read().as_ref());
    let tracks: Vec<_> = columns
        .iter()
        .map(|column| ColumnTrack {
            min_width: column.min_width.max(24),
            fixed_width: resizing
                .read()
                .as_ref()
                .filter(|session| session.measured && session.column.id == column.id)
                .map(|session| session.width)
                .or_else(|| state.column_widths.get(&column.id).copied())
                .map(|width| column.clamp_width(f64::from(width))),
        })
        .collect();
    let width = client_width();
    let width = if width.is_finite() && width > 0.0 {
        width
    } else {
        720.0
    };
    let widths = column_widths(&tracks, width);
    let column_range = if suppress_column_virtualisation {
        0..columns.len()
    } else {
        column_window(&widths, scroll_left(), width, column_buffer)
    };
    let mut column_indices: BTreeSet<usize> = column_range.collect();
    // Preserve keyboard entry even when the first logical column is offscreen.
    if !columns.is_empty() {
        column_indices.insert(0);
    }
    let mounts = focus_context.state.read();
    let mut keep = BTreeSet::new();
    for focus in [
        focused(),
        mounts.active.clone(),
        mounts
            .pending
            .as_ref()
            .and_then(|request| request.target.clone()),
    ]
    .into_iter()
    .flatten()
    {
        match focus.lane {
            GridLane::Header(column)
            | GridLane::Cell(column)
            | GridLane::Editor(column)
            | GridLane::Drag(column) => {
                keep.insert(column);
            }
            GridLane::Selection => {}
        }
    }
    if let Some(session) = editing() {
        keep.insert(session.column);
    }
    if let Some(session) = resizing.read().as_ref() {
        keep.insert(session.column.id.clone());
    }
    if let Some(column) = column_dragging() {
        keep.insert(column);
    }
    if let Some(column) = column_focus() {
        keep.insert(column);
    }
    // Alpha.2 ToggleData exposes no native newState (desktop serializes no
    // state). Keep only the last activated auto-popover header: opening another
    // replaces it, so a closed native filter adds at most one conservative slot.
    // Never guess native state or discover it through JavaScript.
    if let Some(column) = column_filter() {
        if columns.iter().any(|item| item.id == column) {
            keep.insert(column);
        } else {
            column_filter.set(None);
        }
    }
    for (index, column) in columns.iter().enumerate() {
        if keep.contains(&column.id) {
            column_indices.insert(index);
        }
    }
    drop(mounts);
    let column_indices: Vec<_> = column_indices.into_iter().collect();
    let header_state = state.clone();
    let header_rows = rows.clone();
    let header_columns = columns.clone();
    let resize_state = state.clone();
    let hit_autosize = autosize.clone();
    let measured_autosize = autosize.clone();
    let validation_error = editing().and_then(|session| session.error);
    let page_state = state.clone();
    let size_state = state.clone();
    let pagination_label = format!("{label} pages");
    let capacity_label = pagination_labels.items.clone();
    rsx! {
        div { id: grid_id, class: "ix-data-grid {class}", "data-striped": striped.to_string(), "data-paginated":state.pagination.is_some().then_some("true"), "data-virtualized": (!suppress_row_virtualisation).to_string(), "data-column-virtualized": (!suppress_column_virtualisation).to_string(), style: "height:{height}px;--ix-grid-row-height:{row_height}px;--ix-grid-columns:{template};--ix-grid-min-width:{min_width}px",
            onpointerdown: move |_| focus_context.cancel(),
            onfocusout: move |_| focus_context.blurred(),
            onkeydown: move |event: KeyboardEvent| { focus_context.cancel(); if event.key() == Key::Escape && resizing.peek().is_some() { event.prevent_default(); event.stop_propagation(); resizing.set(None); } },
            // The fixed resize overlay can become the native click target. Delegate
            // its double-click by hit-testing separators at the actual click point.
            ondoubleclick: move |event: MouseEvent| {
                let point = event.client_coordinates(); resizing.set(None); hit_autosize.fit_at(point.x,point.y);
            },
            div { id: scroll_id, class: "ix-data-grid-scroll", role: "grid", "aria-label": label, "aria-rowcount": full_count.saturating_add(1), "aria-colcount": columns.len() + 1, "aria-multiselectable": "true",
                onmounted: move |event| {
                    let node = event.data();
                    focus_context.root.set(Some(node.clone()));
                    spawn(async move {
                        if let Ok(rect) = node.get_client_rect().await {
                            if let Ok(mut current) = viewport.try_write() { current.height = rect.size.height.max(1.0); }
                            if rect.size.width > 0.0 { focus_context.width.set(rect.size.width); }
                            report_capacity.call(Some(rect.size.height));
                        }
                    });
                },
                onscroll: move |event: ScrollEvent| {
                    let measured = GridViewport { top: event.scroll_top(), height: f64::from(event.client_height()) };
                    if measured.top.is_finite() && measured.height > 0.0 && *viewport.peek() != measured { viewport.set(measured); }
                    if event.scroll_left().is_finite() { scroll_left.set(event.scroll_left().max(0.0)); }
                    if event.client_width() > 0 { focus_context.width.set(f64::from(event.client_width())); }
                },
                onresize: move |event: ResizeEvent| {
                    if let Ok(size) = event.get_content_box_size() {
                        if size.height > 0.0 { let mut current = *viewport.peek(); current.height = size.height; viewport.set(current); }
                        if size.width.is_finite() && size.width > 0.0 { focus_context.width.set(size.width); }
                        report_capacity.call(Some(size.height));
                    }
                },
                div { class: "ix-data-grid-header", role: "row", "aria-rowindex": "1",
                    div { class: "ix-data-grid-selection", role: "columnheader", style:"grid-column:1", "aria-colindex":1,
                        GridCheckbox { label: "Select all filtered rows", checked, mixed, disabled: full_count == 0,
                            onmounted: header_mount, onfocus: move |_| focus_context.focused(None),
                            onchange: move |_| { let mut next = header_state.clone(); next.select_filtered_with_columns(&header_rows, &header_columns, !checked); onstatechange.call(next); },
                        }
                    }
                    for column_index in column_indices.iter().copied() {
                        GridHeader { key: "{columns[column_index].id}", column:columns[column_index].clone(), column_index, columns: columns.clone(), state: state.clone(), onstatechange, resizing, column_dragging, column_drop_target, focused }
                    }
                }
                div { class: "ix-data-grid-body", role: "rowgroup", style: "height:{visible_count as u64 * u64::from(row_height)}px",
                    for index in rendered {
                        GridRecord { key: "{visible[index].id}", row: visible[index].clone(), row_label_column: row_label_column.clone(), columns: columns.clone(), column_indices:column_indices.clone(), rows: callback_rows.clone(), state: record_state.clone(),
                            onstatechange, dragging, drop_target, tooltip_delay_ms, index, page_offset:page.range.start, row_height, focused, onnavigate: navigate.clone(), suppress_cell_focus, entry_row: index == window.start, total: visible_count, editing, edit_action, read_only, render_cell, render_editor,
                        }
                    }
                }
                if visible_count == 0 { div { class: "ix-data-grid-empty", role: "status", "No Rows To Show" } }
            }
            if state.pagination.is_some() && !hide_pagination_controls {
                fieldset {class:"ix-data-grid-pagination",disabled:editing().is_some(),
                    crate::Pagination {count:page.page_count,selected_page:page.page,advanced:true,item_count:page.page_size,item_count_options:page_size_options,hide_item_count:auto_page_size,label:pagination_label,labels:pagination_labels,
                        onpage:move|requested:usize| {
                            if editing.peek().is_some() {return;}
                            let mut next=page_state.clone();
                            next.pagination=Some(crate::DataGridPagination::new(requested,page.page_size));
                            onstatechange.call(next);
                        },
                        onitemcount:move|requested:usize| {
                            if editing.peek().is_some() {return;}
                            let mut next=size_state.clone();
                            next.pagination=Some(crate::DataGridPagination::new(0,requested.max(1)));
                            onstatechange.call(next);
                        }
                    }
                    if auto_page_size {span {class:"ix-data-grid-page-capacity","{page.page_size} {capacity_label}"}}
                }
                if editing().is_some() {p {class:"ix-data-grid-pagination-hint",role:"status","Save or cancel the cell edit before changing pages."}}
            }
            if let Some(request)=autosize.pending.read().clone() {
                GridAutosizeProbe {key:"{request.serial}",request:request.clone(),onmeasure:{let context=measured_autosize.clone();move |width|context.complete(request.serial,width,onstatechange)}}
            }
            if let Some(error) = validation_error {
                div { id: validation.id.clone(), class: "ix-data-grid-edit-error", role: "alert", "{error}" }
            }
            if resizing.read().is_some() {
                div { class: "ix-data-grid-resize-overlay", "aria-hidden": "true",
                    onpointermove: move |event: PointerEvent| {
                        event.prevent_default();
                        let mut next = resizing.peek().clone();
                        if let Some(session) = &mut next { session.current_x = event.client_coordinates().x; if session.measured { session.width = session.column.clamp_width(session.start_width + session.current_x - session.start_x); } }
                        resizing.set(next);
                    },
                    onpointerup: move |_| {
                        let current = resizing.peek().clone();
                        if let Some(session) = current.filter(|session| session.measured && (session.current_x - session.start_x).abs() > 0.5) { let mut next = resize_state.clone(); next.resize_column(&session.column, session.width as f64); onstatechange.call(next); }
                        resizing.set(None);
                    },
                    onpointercancel: move |_| resizing.set(None),
                }
            }
        }
    }
}

#[component]
fn GridCheckbox(
    label: String,
    checked: bool,
    #[props(default)] mixed: bool,
    #[props(default)] disabled: bool,
    onchange: EventHandler<MouseEvent>,
    #[props(default)] onfocus: EventHandler<()>,
    #[props(default)] onmounted: EventHandler<MountedEvent>,
) -> Element {
    rsx! { button { onmounted, class: "ix-data-grid-check", r#type: "button", role: "checkbox", disabled,
        "aria-label": label, "aria-checked": if mixed { "mixed" } else if checked { "true" } else { "false" },
        "data-checked": (checked || mixed).to_string(), onclick: move |event| onchange.call(event), onfocus: move |_| onfocus.call(()),
        svg { view_box: "0 0 18 18", width: "18", height: "18", "aria-hidden": "true",
            if mixed { path { d: "M4 9h10", stroke: "currentColor", stroke_width: "2", fill: "none" } }
            else if checked { path { d: "m3.5 9 3.5 3.5 7.5-8", stroke: "currentColor", stroke_width: "2", fill: "none" } }
        }
    } }
}

#[component]
fn GridHeader(
    column: DataGridColumn,
    column_index: usize,
    columns: Vec<DataGridColumn>,
    state: DataGridState,
    onstatechange: EventHandler<DataGridState>,
    mut resizing: Signal<Option<ResizeSession>>,
    mut column_dragging: Signal<Option<String>>,
    mut column_drop_target: Signal<Option<String>>,
    mut focused: Signal<Option<GridFocus>>,
) -> Element {
    let focus_context = use_context::<GridMountContext>();
    let header_focus = GridFocus {
        row: String::new(),
        lane: GridLane::Header(column.id.clone()),
    };
    let header_mount = use_grid_focus_lane(Some(header_focus.clone()), None);
    let navigation_columns = columns.clone();
    let navigation_column = column.id.clone();
    let mut column_context = use_context::<GridColumnContext>();
    let focus_column = column.id.clone();
    let filter_column = column.id.clone();
    let autosize = use_context::<GridAutosizeContext>();
    let separator_owner = use_hook(|| Rc::new(Cell::new(true)));
    let drop_owner = separator_owner.clone();
    let drop_autosize = autosize.clone();
    let drop_column = column.id.clone();
    use_drop(move || {
        drop_owner.set(false);
        let mut separators = drop_autosize.separators.borrow_mut();
        if separators
            .get(&drop_column)
            .is_some_and(|mount| Rc::ptr_eq(&mount.owner, &drop_owner))
        {
            separators.remove(&drop_column);
        }
    });
    if !column.resizable {
        autosize.separators.borrow_mut().remove(&column.id);
    }
    let mount_autosize = autosize.clone();
    let mount_column = column.id.clone();
    let pointer_autosize = autosize.clone();
    let pointer_fit_column = column.id.clone();
    let enter_autosize = autosize.clone();
    let enter_fit_column = column.id.clone();
    let resize_autosize = autosize.clone();
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut title_mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut restore_keyboard_focus = use_signal(|| false);
    use_effect(move || {
        if restore_keyboard_focus() {
            restore_keyboard_focus.set(false);
            let node = title_mounted.peek().clone();
            spawn(async move {
                if let Some(node) = node {
                    let _ = node.set_focus(true).await;
                }
            });
        }
    });
    let mut measured_width = use_signal(|| None::<u32>);
    let latest_state = use_hook(|| Rc::new(RefCell::new(state.clone())));
    *latest_state.borrow_mut() = state.clone();
    let filter_id =
        use_hook(|| format!("ix-grid-filter-{}", GRID_ID.fetch_add(1, Ordering::Relaxed)));
    let header_id = use_hook(|| {
        format!(
            "ix-grid-heading-{}",
            GRID_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let title_drag_id = column.id.clone();
    let drop_id = column.id.clone();
    let drag_over_id = column.id.clone();
    let drop_columns = columns.clone();
    let drop_state = latest_state;
    let order_id = column.id.clone();
    let order_state = state.clone();
    let reorderable = column.reorderable;
    let sort = state
        .sort
        .as_ref()
        .filter(|(id, _)| *id == column.id)
        .map(|(_, direction)| *direction);
    let sort_state = state.clone();
    let sort_column = column.id.clone();
    let pointer_column = column.clone();
    let key_column = column.clone();
    let key_state = state.clone();
    let reset_state = state.clone();
    let reset_column = column.id.clone();
    let active = state
        .filters
        .get(&column.id)
        .is_some_and(DataGridTextFilter::is_active);
    let displayed_width = resizing
        .read()
        .as_ref()
        .filter(|session| session.measured && session.column.id == column.id)
        .map(|session| session.width)
        .or_else(|| state.column_widths.get(&column.id).copied())
        .or_else(&*measured_width)
        .unwrap_or(column.min_width.max(24));
    rsx! { div { id: header_id, class: "ix-data-grid-heading", role: "columnheader", "data-column-id": column.id.clone(), style:"grid-column:{column_index+2}", "aria-colindex":column_index+2, tabindex:"-1",
        onfocusin:move |_|{
            column_context.focused.set(Some(focus_column.clone()));
            focus_context.focused(Some(header_focus.clone()));
            focused.set(None);
        },
        "data-column-drop-target": (column_drop_target.read().as_deref() == Some(column.id.as_str())).to_string(),
        ondragover: move |event: DragEvent| { if column_dragging.peek().is_some() { event.prevent_default(); event.data_transfer().set_drop_effect("move"); column_drop_target.set(Some(drag_over_id.clone())); } },
        ondrop: move |event: DragEvent| {
            if let Some(moving) = column_dragging.peek().clone() {
                event.prevent_default(); event.stop_propagation();
                let node = mounted.peek().clone(); let x = event.client_coordinates().x;
                let target = drop_id.clone(); let columns = drop_columns.clone(); let latest = drop_state.clone();
                spawn(async move {
                    if let Some(node) = node { if let Ok(rect) = node.get_client_rect().await {
                        let mut next = latest.borrow().clone();
                        if next.move_column(&columns, &moving, &target, x > rect.origin.x + rect.size.width / 2.0) { onstatechange.call(next); }
                    } }
                    column_dragging.set(None); column_drop_target.set(None);
                });
            }
        },
        onkeydown: move |event: KeyboardEvent| {
            if !event.modifiers().alt() && matches!(event.key(),Key::ArrowLeft|Key::ArrowRight) {
                event.prevent_default();event.stop_propagation();
                if let Some(index)=navigation_columns.iter().position(|column|column.id==navigation_column) {
                    let right=event.key()==Key::ArrowRight;
                    let jump=event.modifiers().ctrl()||event.modifiers().meta();
                    let next=if right {if jump {navigation_columns.len().saturating_sub(1)} else {(index+1).min(navigation_columns.len().saturating_sub(1))}}
                        else if jump {0} else {index.saturating_sub(1)};
                    if let Some(column)=navigation_columns.get(next) {
                        focus_context.request(Some(GridFocus {row:String::new(),lane:GridLane::Header(column.id.clone())}),None);
                    }
                }
                return;
            }
            if reorderable && event.modifiers().alt() && matches!(event.key(), Key::ArrowLeft | Key::ArrowRight) {
                event.prevent_default(); event.stop_propagation();
                let ids: Vec<_> = order_state.ordered_columns(&columns).into_iter().map(|column| column.id.clone()).collect();
                if let Some(index) = ids.iter().position(|id| *id == order_id) {
                    let right = event.key() == Key::ArrowRight;
                    let adjacent = if right { index.checked_add(1).and_then(|next| ids.get(next)) } else { index.checked_sub(1).and_then(|next| ids.get(next)) };
                    if let Some(target) = adjacent { let mut next = order_state.clone(); if next.move_column(&columns, &order_id, target, right) { restore_keyboard_focus.set(true); onstatechange.call(next); } }
                }
            }
        },
        "aria-sort": if sort == Some(DataGridSortDirection::Ascending) { "ascending" } else if sort == Some(DataGridSortDirection::Descending) { "descending" } else { "none" },
        onmounted: move |event:MountedEvent| { header_mount.call(event.clone()); let node = event.data(); mounted.set(Some(node.clone())); spawn(async move { if let Ok(rect) = node.get_client_rect().await { measured_width.set(Some(rect.size.width.round() as u32)); } }); },
        div { class: "ix-data-grid-title", draggable: reorderable.to_string(), title: if reorderable { "Drag to move column. Keyboard: Alt+Left or Alt+Right." } else { "" },
            ondragstart: move |event: DragEvent| { if reorderable { let transfer = event.data_transfer(); let _ = transfer.set_data("text/plain", &title_drag_id); transfer.set_effect_allowed("move"); column_dragging.set(Some(title_drag_id.clone())); } },
            ondragend: move |_| { column_dragging.set(None); column_drop_target.set(None); },
        if column.sortable { button { class: "ix-data-grid-sort", r#type: "button", "aria-label": "Sort {column.title}", "aria-description": if reorderable { "Move column with Alt+Left or Alt+Right." } else { "" },
            onmounted: move |event| title_mounted.set(Some(event.data())),
            onclick: move |_| { let mut next = sort_state.clone(); next.cycle_sort(&sort_column); onstatechange.call(next); },
            {grid_header_title(&column.title,sort)}
        } } else if reorderable { button { class: "ix-data-grid-header-label ix-data-grid-title-button", r#type: "button", "aria-label": "Move column {column.title}", "aria-description": "Move column with Alt+Left or Alt+Right.", onmounted: move |event| title_mounted.set(Some(event.data())), "{column.title}" } }
        else { span { class: "ix-data-grid-header-label", "{column.title}" } }
        }
        if column.filterable {
            button { class: "ix-data-grid-filter-trigger", r#type: "button", "aria-label": "Filter {column.title}", "aria-haspopup": "dialog",
                "popovertarget": filter_id.clone(), onclick:move |_|column_context.filter.set(Some(filter_column.clone())), "data-active": active.to_string(), style: "anchor-name:--{filter_id}",
                Icon { name: if active { "filter-filled" } else { "filter" }, size: 16 }
            }
            div { id: filter_id.clone(), class: "ix-data-grid-filter", role: "dialog", "aria-label": "Filter {column.title}", "popover": "auto", style: "position-anchor:--{filter_id}",
                GridFilterEditor { column: column.clone(), state: state.clone(), onstatechange }
                button { r#type: "button", class: "ix-data-grid-filter-clear", onclick: move |_| { let mut next = reset_state.clone(); next.filters.remove(&reset_column); next.reset_pagination_page(); onstatechange.call(next); }, "Reset" }
            }
        }
        if column.resizable {
            div { onmounted:move |event|{mount_autosize.separators.borrow_mut().insert(mount_column.clone(),GridSeparatorMount {node:event.data(),owner:separator_owner.clone()});},class: "ix-data-grid-resizer", role: "separator", tabindex: "0", "aria-orientation": "vertical", "aria-label": "Resize {column.title}",
                title: "Drag or Left/Right to resize. Double-click or Enter to fit content.",
                "aria-description": "Left and Right resize the column. Enter fits its header and filtered row content.",
                "aria-valuemin": column.min_width.max(24), "aria-valuemax": column.max_width.max(column.min_width.max(24)),
                "aria-valuenow": displayed_width,
                ondoubleclick: move |event: MouseEvent| {
                    event.prevent_default(); event.stop_propagation(); resizing.set(None);
                    pointer_autosize.fit(&pointer_fit_column);
                },
                onfocus: move |_| { let node = mounted.peek().clone(); spawn(async move { if let Some(node) = node { if let Ok(rect) = node.get_client_rect().await { measured_width.set(Some(rect.size.width.round() as u32)); } } }); },
                onpointerdown: move |event: PointerEvent| {
                    event.prevent_default(); event.stop_propagation();
                    resize_autosize.cancel();
                    let start_x = event.client_coordinates().x; let column = pointer_column.clone(); let node = mounted.peek().clone();
                    let id = GRID_ID.fetch_add(1, Ordering::Relaxed);
                    // Capture releases immediately, even when the desktop renderer
                    // needs an IPC roundtrip to measure this column.
                    resizing.set(Some(ResizeSession { id, width: displayed_width, column, start_x, current_x: start_x, start_width: 0.0, measured: false }));
                    spawn(async move {
                        let rect = if let Some(node) = node { node.get_client_rect().await.ok() } else { None };
                        let current = resizing.peek().clone();
                        if let Some(mut session) = current.filter(|session| session.id == id) {
                            if let Some(rect) = rect { session.start_width = rect.size.width; session.measured = true; session.width = session.column.clamp_width(session.start_width + session.current_x - session.start_x); resizing.set(Some(session)); }
                            else { resizing.set(None); }
                        }
                    });
                },
                onkeydown: move |event: KeyboardEvent| {
                    let key = event.key();
                    if event.modifiers().alt() { return; }
                    if matches!(key, Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End | Key::Enter) {
                        event.prevent_default(); event.stop_propagation();
                        let step = if event.modifiers().shift() { 25.0 } else { 10.0 };
                        let node = mounted.peek().clone(); let mut next = key_state.clone(); let column = key_column.clone();
                        if key == Key::Enter {
                            enter_autosize.fit(&enter_fit_column);
                            return;
                        }
                        enter_autosize.cancel();
                        spawn(async move { if let Some(node) = node { if let Ok(rect) = node.get_client_rect().await {
                            let width = match key { Key::Home => column.min_width as f64, Key::End => column.max_width as f64,
                                Key::ArrowLeft => rect.size.width - step, _ => rect.size.width + step };
                            next.resize_column(&column, width); measured_width.set(Some(column.clamp_width(width))); onstatechange.call(next);
                        } } });
                    }
                },
            }
        }
    } }
}

#[component]
fn GridFilterEditor(
    column: DataGridColumn,
    state: DataGridState,
    onstatechange: EventHandler<DataGridState>,
) -> Element {
    let filter = state.filters.get(&column.id).cloned().unwrap_or_default();
    let join_state = state.clone();
    let join_column = column.id.clone();
    let join_filter = filter.clone();
    rsx! { div { class: "ix-data-grid-filter-fields", onkeydown:move |event:KeyboardEvent|event.stop_propagation(),
        GridFilterInput { column: column.clone(), state: state.clone(), second: false, onstatechange }
        if filter.first.is_active() || filter.second.is_active() {
            label { class: "ix-data-grid-filter-join", span { class: "ix-sr-only", "Combine filter conditions" }
                select { value: if filter.match_any { "or" } else { "and" }, "aria-label": "Combine filter conditions",
                    onchange: move |event: FormEvent| { let mut next = join_state.clone(); let mut filter = join_filter.clone(); filter.match_any = event.value() == "or"; next.filters.insert(join_column.clone(), filter); next.reset_pagination_page(); onstatechange.call(next); },
                    option { value: "and", selected: !filter.match_any, "AND" } option { value: "or", selected: filter.match_any, "OR" }
                }
            }
            GridFilterInput { column, state, second: true, onstatechange }
        }
    } }
}

#[component]
fn GridFilterInput(
    column: DataGridColumn,
    state: DataGridState,
    second: bool,
    onstatechange: EventHandler<DataGridState>,
) -> Element {
    let mut filter = state.filters.get(&column.id).cloned().unwrap_or_default();
    if column.value_type == DataGridValueType::Number {
        for condition in [&mut filter.first, &mut filter.second] {
            if !condition.is_active() && condition.operator == DataGridFilterOperator::Contains {
                condition.operator = DataGridFilterOperator::Equals;
            }
        }
    }
    let operators = DataGridFilterOperator::choices(column.value_type);
    let input_type = if column.value_type == DataGridValueType::Number {
        "number"
    } else {
        "text"
    };
    let condition = if second {
        filter.second.clone()
    } else {
        filter.first.clone()
    };
    let operator_state = state.clone();
    let operator_column = column.id.clone();
    let operator_filter = filter.clone();
    let text_state = state.clone();
    let text_column = column.id.clone();
    let text_filter = filter.clone();
    let number = if second { 2 } else { 1 };
    rsx! { div { class: "ix-data-grid-filter-condition",
        select { "aria-label": "{column.title} filter condition {number}", value: condition.operator.token(),
            onchange: move |event: FormEvent| {
                let mut filter = operator_filter.clone(); let condition = if second { &mut filter.second } else { &mut filter.first };
                condition.operator = DataGridFilterOperator::from_token(&event.value());
                let mut next = operator_state.clone(); next.filters.insert(operator_column.clone(), filter); next.reset_pagination_page(); onstatechange.call(next);
            },
            for operator in operators.iter().copied() { option { value: operator.token(), selected: operator == condition.operator, "{operator.label()}" } }
        }
        if condition.operator.needs_value() { input { r#type: input_type, step: if input_type == "number" { Some("any") } else { None }, value: condition.value, placeholder: "Filter...", autofocus: !second, "aria-label": "{column.title} filter value {number}",
            oninput: move |event: FormEvent| {
                let mut filter = text_filter.clone(); let condition = if second { &mut filter.second } else { &mut filter.first }; condition.value = event.value();
                let mut next = text_state.clone(); next.filters.insert(text_column.clone(), filter); next.reset_pagination_page(); onstatechange.call(next);
            },
        } }
    } }
}

#[component]
fn GridRecord(
    row: DataGridRow,
    row_label_column: Option<String>,
    columns: Vec<DataGridColumn>,
    column_indices: Vec<usize>,
    rows: Rc<Vec<DataGridRow>>,
    state: Rc<DataGridState>,
    onstatechange: EventHandler<DataGridState>,
    mut dragging: Signal<Option<String>>,
    mut drop_target: Signal<Option<String>>,
    tooltip_delay_ms: u32,
    index: usize,
    #[props(default)] page_offset: usize,
    row_height: u32,
    mut focused: Signal<Option<GridFocus>>,
    onnavigate: EventHandler<(usize, Key, bool, bool)>,
    suppress_cell_focus: bool,
    entry_row: bool,
    total: usize,
    editing: Signal<Option<GridEditSession>>,
    edit_action: EventHandler<GridEditAction>,
    read_only: bool,
    #[props(default)] render_cell: Option<
        Callback<DataGridCellContext, Option<DataGridCellRender>>,
    >,
    #[props(default)] render_editor: Option<Callback<DataGridEditorContext, Option<Element>>>,
) -> Element {
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let selected = state.selected.contains(&row.id);
    let row_label = row_label_column
        .as_deref()
        .map(|column| row.value(column))
        .unwrap_or(&row.id)
        .to_string();
    let cell_columns = Rc::new(columns.clone());
    let select_state = state.clone();
    let select_rows = rows.clone();
    let select_columns = columns.clone();
    let select_id = row.id.clone();
    let drop_state = state.clone();
    let drop_rows = rows.clone();
    let drop_id = row.id.clone();
    let over_id = row.id.clone();
    let can_reorder = state.can_reorder();
    let is_target = drop_target.read().as_deref() == Some(row.id.as_str());
    let focus_context = use_context::<GridMountContext>();
    let selection_mount = use_grid_focus_lane(
        Some(GridFocus {
            row: row.id.clone(),
            lane: GridLane::Selection,
        }),
        None,
    );
    let focus_id = row.id.clone();
    let first_column = columns.first().map(|column| column.id.clone());
    let last_column = columns.last().map(|column| column.id.clone());
    rsx! { div { class: "ix-data-grid-row", role: "row", "data-row-id": row.id.clone(), "data-odd": (index.saturating_add(page_offset) % 2 == 1).to_string(),
        style: "top:{index as u64 * u64::from(row_height)}px",
        "data-selected": selected.to_string(), "data-drop-target": is_target.to_string(), "aria-rowindex": index.saturating_add(page_offset).saturating_add(2), "aria-selected": selected.to_string(),
        onmounted: move |event| mounted.set(Some(event.data())),
        onkeydown: move |event: KeyboardEvent| {
            if event.modifiers().alt() { return; }
            let key = event.key();
            let command = event.modifiers().ctrl() || event.modifiers().meta();
            let horizontal = matches!(key, Key::ArrowLeft | Key::ArrowRight) && focused.peek().as_ref().is_some_and(|focus| matches!(focus.lane, GridLane::Cell(_)));
            let tab = key == Key::Tab && focused.peek().as_ref().is_some_and(|focus| if let GridLane::Cell(id)=&focus.lane {
                if event.modifiers().shift() {index>0 || first_column.as_ref()!=Some(id)} else {index+1<total || last_column.as_ref()!=Some(id)}
            } else {false});
            if horizontal || tab || matches!(key, Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End | Key::PageUp | Key::PageDown) {
                event.prevent_default(); event.stop_propagation(); onnavigate.call((index,key,command,event.modifiers().shift()));
            }
        },
        ondragover: move |event: DragEvent| { if can_reorder && dragging.peek().is_some() { event.prevent_default(); event.data_transfer().set_drop_effect("move"); drop_target.set(Some(over_id.clone())); } },
        ondrop: move |event: DragEvent| {
            if can_reorder { event.prevent_default(); let moving = dragging.peek().clone(); let node = mounted.peek().clone(); let y = event.client_coordinates().y;
                let mut next = drop_state.as_ref().clone(); let rows = drop_rows.clone(); let target = drop_id.clone();
                spawn(async move { if let (Some(moving), Some(node)) = (moving, node) { if let Ok(rect) = node.get_client_rect().await {
                    if next.move_row(&rows, &moving, &target, y > rect.origin.y + rect.size.height / 2.0) { onstatechange.call(next); }
                } } dragging.set(None); drop_target.set(None); });
            }
        },
        div { class: "ix-data-grid-selection", role: "gridcell", style:"grid-column:1", "aria-colindex":1, GridCheckbox { label: format!("Select {row_label}"), checked: selected,
            onmounted: selection_mount,
            onfocus: move |_| { let focus = GridFocus {row:focus_id.clone(),lane:GridLane::Selection}; focus_context.focused(Some(focus.clone())); focused.set(Some(focus)); if editing.peek().is_some() { edit_action.call(GridEditAction::Commit { tab:None, restore_focus:false }); } },
            onchange: move |event: MouseEvent| { let mut next = select_state.as_ref().clone(); next.select_row_with_columns(&select_rows, &select_columns, &select_id, !selected, event.modifiers().shift()); onstatechange.call(next); },
        } }
        for (column_index, column) in column_indices.into_iter().map(|index|(index,columns[index].clone())) {
            GridCell { key: "{column.id}", text: row.value(&column.id).to_string(), tooltip: row.tooltips.get(&column.id).cloned().unwrap_or_else(|| row.value(&column.id).to_string()),
                column: column.clone(), column_index, columns: cell_columns.clone(), row_id: row.id.clone(), row_label: row_label.clone(), rows: rows.clone(), state: state.clone(),
                onstatechange, dragging, drop_target, tooltip_delay_ms, focused, suppress_cell_focus, entry_cell: entry_row && column_index == 0, editing, edit_action, editable: editable_cell(&row,&column,read_only),
                render_cell, render_editor,
                render_context: (render_cell.is_some() || render_editor.is_some()).then(|| DataGridCellContext {
                    row: row.clone(), column: column.clone(), value: row.value(&column.id).to_string(),
                    row_index: index.saturating_add(page_offset), selected, editable: editable_cell(&row,&column,read_only) && !suppress_cell_focus,
                    grid_read_only: read_only,
                }),
            }
        }
    } }
}

#[component]
fn GridCell(
    text: String,
    tooltip: String,
    column: DataGridColumn,
    #[props(default)] column_index: usize,
    #[props(default)] columns: Rc<Vec<DataGridColumn>>,
    row_id: String,
    row_label: String,
    rows: Rc<Vec<DataGridRow>>,
    state: Rc<DataGridState>,
    onstatechange: EventHandler<DataGridState>,
    mut dragging: Signal<Option<String>>,
    mut drop_target: Signal<Option<String>>,
    tooltip_delay_ms: u32,
    mut focused: Signal<Option<GridFocus>>,
    suppress_cell_focus: bool,
    entry_cell: bool,
    editing: Signal<Option<GridEditSession>>,
    edit_action: EventHandler<GridEditAction>,
    editable: bool,
    #[props(default)] render_context: Option<DataGridCellContext>,
    #[props(default)] render_cell: Option<
        Callback<DataGridCellContext, Option<DataGridCellRender>>,
    >,
    #[props(default)] render_editor: Option<Callback<DataGridEditorContext, Option<Element>>>,
) -> Element {
    let column_context = try_use_context::<GridColumnContext>();
    let native_focus_column = column.id.clone();
    let focus_context = use_context::<GridMountContext>();
    let cell_mount = use_grid_focus_lane(
        Some(GridFocus {
            row: row_id.clone(),
            lane: GridLane::Cell(column.id.clone()),
        }),
        None,
    );
    let drag_mount = use_grid_focus_lane(
        Some(GridFocus {
            row: row_id.clone(),
            lane: GridLane::Drag(column.id.clone()),
        }),
        None,
    );
    let mut generation = use_signal(|| 0_u64);
    let mut show_tooltip = use_signal(|| false);
    let mut point = use_signal(|| (0.0_f64, 0.0_f64));
    let tooltip_id = use_hook(|| {
        format!(
            "ix-grid-tooltip-{}",
            GRID_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let drag_id = row_id.clone();
    let key_id = row_id.clone();
    let can_reorder = state.can_reorder();
    let focus_row = row_id.clone();
    let focus_column = column.id.clone();
    let drag_focus_row = row_id.clone();
    let drag_focus_column = column.id.clone();
    let is_active = focused.read().as_ref().is_some_and(|focus| {
        focus.row == row_id
            && matches!(&focus.lane,GridLane::Cell(id)|GridLane::Editor(id) if *id==column.id)
    });
    let edit_row = row_id.clone();
    let edit_column = column.id.clone();
    let double_row = row_id.clone();
    let double_column = column.id.clone();
    let current_session = editing
        .read()
        .as_ref()
        .filter(|session| session.row == row_id && session.column == column.id)
        .cloned();
    let last_editor = use_hook(|| Rc::new(RefCell::new(None::<GridEditSession>)));
    if let Some(session) = &current_session {
        *last_editor.borrow_mut() = Some(session.clone());
    }
    // Retain the one outgoing, actually focused editor while its next mounted
    // target is pending. Removing it first emits native focusout and looks like
    // a real outside departure. Target focus/cancellation releases this input;
    // outside focusout still cancels normally, without inspecting relatedTarget.
    let retiring = current_session.is_none()
        && editable
        && !suppress_cell_focus
        && last_editor.borrow().as_ref().is_some_and(|session| {
            retain_outgoing_grid_editor(session, &focus_context.state.read())
        });
    let session = if retiring {
        last_editor.borrow().clone()
    } else {
        current_session
    };
    if session.is_none() {
        *last_editor.borrow_mut() = None;
    }
    let select_state = state.clone();
    let select_rows = rows.clone();
    let select_columns = columns.clone();
    let select_id = row_id.clone();
    let custom = if session.is_none() {
        render_cell.and_then(|render| {
            render_context
                .clone()
                .and_then(|context| render.call(context))
        })
    } else {
        None
    };
    let interactive = custom.as_ref().is_some_and(|render| render.interactive);
    let content_focus_row = row_id.clone();
    let content_focus_column = column.id.clone();
    let text_entry = matches!(&column.cell_editor, DataGridCellEditor::Text);
    let type_to_edit = text_entry || matches!(&column.cell_editor, DataGridCellEditor::Number(_));
    rsx! { div { onmounted: cell_mount, class: "ix-data-grid-cell", role: "gridcell", "data-column-id": column.id.clone(), style:"grid-column:{column_index+2}", "aria-colindex":column_index+2,
        "aria-readonly": (!editable).to_string(), "data-editing": session.is_some().to_string(), "aria-describedby": if show_tooltip() { Some(tooltip_id.clone()) } else { None },
        tabindex: if suppress_cell_focus { None } else { Some(if session.is_none() && (is_active || (entry_cell && !focused.read().as_ref().is_some_and(|focus| matches!(focus.lane, GridLane::Cell(_) | GridLane::Editor(_))))) { "0" } else { "-1" }) },
        onfocus: move |_| if !suppress_cell_focus {
            let current=editing.peek().clone();
            let own=current.as_ref().is_some_and(|session|session.row==focus_row && session.column==focus_column);
            let focus = GridFocus {row:focus_row.clone(),lane:if own {GridLane::Editor(focus_column.clone())} else {GridLane::Cell(focus_column.clone())}};
            focus_context.focused(Some(focus.clone())); focused.set(Some(focus));
            if current.is_some() && !own { edit_action.call(GridEditAction::Commit {tab:None,restore_focus:false}); }
        },
        onmouseenter: move |event: MouseEvent| {
            let current = generation.peek().wrapping_add(1); generation.set(current);
            let coordinates = event.client_coordinates(); point.set((coordinates.x + 12.0, coordinates.y + 18.0));
            spawn(async move { futures_timer::Delay::new(Duration::from_millis(tooltip_delay_ms as u64)).await; if *generation.peek() == current { show_tooltip.set(true); } });
        },
        onmouseleave: move |_| { let next = generation.peek().wrapping_add(1); generation.set(next); show_tooltip.set(false); },
        ondoubleclick: move |event: MouseEvent| if editable && !suppress_cell_focus { event.stop_propagation(); edit_action.call(GridEditAction::Start {row:double_row.clone(),column:double_column.clone(),initial:None}); },
        onkeydown: move |event: KeyboardEvent| {
            if interactive && event.key() == Key::Tab {
                // Native Tab enters the cell's composed controls rather than
                // being translated into virtual row/cell navigation.
                event.stop_propagation();
                return;
            }
            if editable && !suppress_cell_focus && !event.is_composing() && !event.modifiers().ctrl() && !event.modifiers().meta() && !event.modifiers().alt() && focused.peek().as_ref().is_some_and(|focus|matches!(focus.lane,GridLane::Cell(_))) {
                let initial=match event.key() {Key::Enter|Key::F2=>Some(None),Key::Backspace if type_to_edit=>Some(Some(String::new())),Key::Character(ref value) if type_to_edit && (text_entry || value != " ") && value.chars().count()==1=>Some(Some(value.clone())),_=>None};
                if let Some(initial)=initial {event.prevent_default();event.stop_propagation();edit_action.call(GridEditAction::Start {row:edit_row.clone(),column:edit_column.clone(),initial});return;}
            }
            if event.key() == Key::Escape { let next = generation.peek().wrapping_add(1); generation.set(next); show_tooltip.set(false); }
            if !suppress_cell_focus && event.key() == Key::Character(" ".into()) && focused.peek().as_ref().is_some_and(|focus| matches!(focus.lane,GridLane::Cell(_))) {
                event.prevent_default(); event.stop_propagation(); let mut next=select_state.as_ref().clone(); let checked=!next.selected.contains(&select_id); next.select_row_with_columns(&select_rows,&select_columns,&select_id,checked,event.modifiers().shift()); onstatechange.call(next);
            }
        },
        if column.row_drag { button { onmounted: drag_mount, class: "ix-data-grid-drag", r#type: "button", draggable: can_reorder.to_string(), disabled: !can_reorder,
            onfocus: move |event: FocusEvent| { event.stop_propagation(); let focus = GridFocus {row:drag_focus_row.clone(),lane:GridLane::Drag(drag_focus_column.clone())}; focus_context.focused(Some(focus.clone())); focused.set(Some(focus)); },
            "aria-label": "Move {row_label}", title: if can_reorder { "Drag to move. Keyboard: Alt+Up or Alt+Down." } else { "Row movement is unavailable while sorting or filtering." },
            ondragstart: move |event: DragEvent| { if can_reorder { let transfer = event.data_transfer(); let _ = transfer.set_data("text/plain", &drag_id); transfer.set_effect_allowed("move"); dragging.set(Some(drag_id.clone())); } },
            ondragend: move |_| { dragging.set(None); drop_target.set(None); },
            onkeydown: move |event: KeyboardEvent| {
                if can_reorder && event.modifiers().alt() && matches!(event.key(), Key::ArrowUp | Key::ArrowDown) {
                    event.prevent_default(); event.stop_propagation();
                    let ids: Vec<_> = state.paged_rows_with_columns(&rows, &columns).into_iter().map(|row| row.id.clone()).collect();
                    if let Some(index) = ids.iter().position(|id| *id == key_id) {
                        let down = event.key() == Key::ArrowDown;
                        let adjacent = if down { index.checked_add(1).and_then(|next| ids.get(next)) } else { index.checked_sub(1).and_then(|next| ids.get(next)) };
                        if let Some(target) = adjacent { let mut next = state.as_ref().clone(); if next.move_row(&rows, &key_id, target, down) { onstatechange.call(next); } }
                    }
                }
            },
            Icon { name: "drag-gripper", size: 16 }
        } }
        if let Some(session)=session.clone() { GridCellEditor {session,title:column.title.clone(),kind:column.cell_editor.clone(),editing,edit_action,retiring,render_editor,cell_context:render_context} }
        else if let Some(render) = custom {
            div { class: "ix-data-grid-cell-content",
                onfocus: move |event: FocusEvent| { if interactive { event.stop_propagation(); } },
                onfocusin: move |event: FocusEvent| {
                    if interactive {
                        event.stop_propagation();
                        // Own this transition once, on focusin. The separate
                        // focus boundary only contains the parent cell handler.
                        focus_context.focused(None);
                        if let Some(mut context)=column_context { context.focused.set(Some(native_focus_column.clone())); }
                        focused.set(None);
                        if editing.peek().is_some() {
                            edit_action.call(GridEditAction::Commit { tab: None, restore_focus: false });
                        }
                    }
                },
                onkeydown: move |event: KeyboardEvent| {
                    if interactive {
                        event.stop_propagation();
                        if !suppress_cell_focus && event.default_action_enabled()
                            && !event.is_composing() && event.key() == Key::Escape
                        {
                            event.prevent_default();
                            let target = GridFocus { row: content_focus_row.clone(), lane: GridLane::Cell(content_focus_column.clone()) };
                            focus_context.request(Some(target.clone()), None);
                            focused.set(Some(target));
                        }
                    }
                },
                ondoubleclick: move |event: MouseEvent| { if interactive { event.stop_propagation(); } },
                {render.content}
            }
        }
        else { {grid_cell_text(&text,false)} }
        if editing.read().is_none() && show_tooltip() && !tooltip.is_empty() { span { id: tooltip_id.clone(), class: "ix-data-grid-tooltip", role: "tooltip", style: "--ix-grid-tip-x:{point().0}px;--ix-grid-tip-y:{point().1}px", "{tooltip}" } }
    } }
}

fn grid_editor_session_active(
    editing: Signal<Option<GridEditSession>>,
    id: usize,
    retiring: bool,
    alive: &Cell<bool>,
) -> bool {
    alive.get()
        && !retiring
        && editing
            .try_peek()
            .is_ok_and(|session| session.as_ref().is_some_and(|session| session.id == id))
}

#[component]
fn GridCellEditor(
    session: GridEditSession,
    title: String,
    kind: DataGridCellEditor,
    mut editing: Signal<Option<GridEditSession>>,
    edit_action: EventHandler<GridEditAction>,
    #[props(default)] retiring: bool,
    #[props(default)] render_editor: Option<Callback<DataGridEditorContext, Option<Element>>>,
    #[props(default)] cell_context: Option<DataGridCellContext>,
) -> Element {
    let focus_context = use_context::<GridMountContext>();
    let editor_id =
        use_hook(|| format!("ix-grid-editor-{}", GRID_ID.fetch_add(1, Ordering::Relaxed)));
    let error_id = use_context::<GridValidationContext>().id;
    let session_id = session.id;
    let callback_owner = crate::data_grid_custom_editor::EditorCallbackOwner::capture();
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let drop_alive = alive.clone();
    use_drop(move || drop_alive.set(false));
    let update_alive = alive.clone();
    let update = EventHandler::new(move |value: String| {
        if !grid_editor_session_active(editing, session_id, retiring, &update_alive) {
            return;
        }
        focus_context.cancel();
        let current = editing.peek().clone();
        if let Some(mut session) = current.filter(|session| session.id == session_id) {
            session.draft = value;
            session.error = None;
            session.select_all = false;
            editing.set(Some(session));
        }
    });
    let action_alive = alive.clone();
    let action = EventHandler::new(move |action: GridEditAction| {
        if grid_editor_session_active(editing, session_id, retiring, &action_alive) {
            edit_action.call(action);
        }
    });
    let focus = {
        let row = session.row.clone();
        let column = session.column.clone();
        let alive = alive.clone();
        EventHandler::new(move |_: FocusEvent| {
            if grid_editor_session_active(editing, session_id, retiring, &alive) {
                focus_context.focused(Some(GridFocus {
                    row: row.clone(),
                    lane: GridLane::Editor(column.clone()),
                }));
            }
        })
    };
    let key_alive = alive.clone();
    let keydown = EventHandler::new(move |event: KeyboardEvent| {
        event.stop_propagation();
        if !grid_editor_session_active(editing, session_id, retiring, &key_alive) {
            event.prevent_default();
            return;
        }
        if !event.default_action_enabled() || event.is_composing() {
            return;
        }
        match event.key() {
            Key::Escape => {
                event.prevent_default();
                action.call(GridEditAction::Cancel);
            }
            Key::Enter => {
                event.prevent_default();
                action.call(GridEditAction::Commit {
                    tab: None,
                    restore_focus: true,
                });
            }
            Key::Tab => {
                event.prevent_default();
                action.call(GridEditAction::Commit {
                    tab: Some(event.modifiers().shift()),
                    restore_focus: true,
                });
            }
            _ => {}
        }
    });
    let pointer_guard = EventHandler::new(move |event: MouseEvent| {
        if retiring {
            event.prevent_default();
            event.stop_propagation();
        }
    });
    let label = format!("Edit {title} for {}", session.row);
    let invalid = session.error.is_some().to_string();
    let error_message = session.error.as_ref().map(|_| error_id);
    // The renderer needs its primary onmounted handler before it chooses Some
    // or None. This scoped forwarder supplies that handler to the single
    // existing focus lane after the choice is known, without a second runtime.
    let mount_forward = use_hook(|| Rc::new(RefCell::new(None::<EventHandler<MountedEvent>>)));
    let forwarded = mount_forward.clone();
    let mount_alive = alive.clone();
    let custom_mount = EventHandler::new(move |event: MountedEvent| {
        if grid_editor_session_active(editing, session_id, retiring, &mount_alive) {
            let handler = *forwarded.borrow();
            if let Some(handler) = handler {
                handler.call(event);
            }
        }
    });
    let custom = render_editor
        .and_then(|render| {
            cell_context.map(|cell| {
                render.call(DataGridEditorContext {
                    cell,
                    draft: session.draft.clone(),
                    error: session.error.clone(),
                    label: label.clone(),
                    error_id: error_message.clone(),
                    active: grid_editor_session_active(editing, session_id, retiring, &alive),
                    onmounted: custom_mount,
                    callback_owner,
                    alive: alive.clone(),
                    set_value_handler: update,
                    commit_handler: EventHandler::new(move |_| {
                        action.call(GridEditAction::Commit {
                            tab: None,
                            restore_focus: true,
                        })
                    }),
                    cancel_handler: EventHandler::new(move |_| action.call(GridEditAction::Cancel)),
                    advance_handler: EventHandler::new(move |backwards| {
                        action.call(GridEditAction::Commit {
                            tab: Some(backwards),
                            restore_focus: true,
                        })
                    }),
                })
            })
        })
        .flatten();
    let editor_mount = use_grid_focus_lane(
        Some(GridFocus {
            row: session.row.clone(),
            lane: GridLane::Editor(session.column.clone()),
        }),
        (custom.is_none() && matches!(&kind, DataGridCellEditor::Text))
            .then(|| (editor_id.clone(), session.select_all)),
    );
    *mount_forward.borrow_mut() = custom.is_some().then_some(editor_mount);
    if let Some(content) = custom {
        return rsx! { div {
            class: "ix-data-grid-editor-wrap ix-data-grid-custom-editor", "data-editor-kind": "custom", "data-retiring": retiring.to_string(),
            onfocusin: move |event: FocusEvent| { event.stop_propagation(); focus.call(event); },
            onkeydown: keydown,
            onclick: move |event: MouseEvent| { event.stop_propagation(); pointer_guard.call(event); },
            onmousedown: pointer_guard, ondoubleclick: move |event: MouseEvent| event.stop_propagation(),
            {content}
        } };
    }
    let control = match kind {
        DataGridCellEditor::Text => rsx! {
            input { id: editor_id, onmounted: editor_mount, class: "ix-data-grid-editor", r#type: "text",
                readonly: retiring, tabindex: retiring.then_some("-1"), value: session.draft,
                "data-select-all": session.select_all.to_string(), "data-editor-kind": "text", "data-retiring": retiring.to_string(),
                "aria-label": label, "aria-invalid": invalid, "aria-errormessage": error_message,
                onfocus: focus, onkeydown: keydown, onclick: pointer_guard, onmousedown: pointer_guard,
                oninput: move |event: FormEvent| update.call(event.value()),
            }
        },
        DataGridCellEditor::Number(config) => rsx! {
            input { id: editor_id, onmounted: editor_mount, class: "ix-data-grid-editor", r#type: "number",
                readonly: retiring, tabindex: retiring.then_some("-1"), value: session.draft,
                min: config.min.map(|value| value.to_string()), max: config.max.map(|value| value.to_string()),
                step: config.step.map_or_else(|| "any".to_string(), |value| value.to_string()),
                "data-editor-kind": "number", "data-retiring": retiring.to_string(),
                "aria-label": label, "aria-invalid": invalid, "aria-errormessage": error_message,
                onfocus: focus, onkeydown: keydown, onclick: pointer_guard, onmousedown: pointer_guard,
                oninput: move |event: FormEvent| update.call(event.value()),
            }
        },
        DataGridCellEditor::Select(options) => {
            let known = options.iter().any(|option| option.value == session.draft);
            rsx! {
                select { id: editor_id, onmounted: editor_mount, class: "ix-data-grid-editor",
                    tabindex: retiring.then_some("-1"), value: session.draft.clone(), "aria-disabled": retiring.to_string(),
                    "data-editor-kind": "select", "data-retiring": retiring.to_string(),
                    "aria-label": label, "aria-invalid": invalid, "aria-errormessage": error_message,
                    onfocus: focus, onkeydown: keydown, onclick: pointer_guard, onmousedown: pointer_guard,
                    onchange: move |event: FormEvent| { if retiring { event.prevent_default(); } else { update.call(event.value()); } },
                    if !known {
                        option { value: session.draft.clone(), disabled: true, selected: true,
                            if session.draft.is_empty() { "Choose an option" } else { "Unavailable value: {session.draft}" }
                        }
                    }
                    for option in options {
                        option { key: "{option.value}", value: option.value.clone(), disabled: option.disabled,
                            selected: option.value == session.draft, "{option.label}"
                        }
                    }
                }
            }
        }
        DataGridCellEditor::Checkbox => rsx! {
            input { id: editor_id, onmounted: editor_mount, class: "ix-data-grid-editor", r#type: "checkbox",
                tabindex: retiring.then_some("-1"), checked: session.draft == "true", "aria-disabled": retiring.to_string(),
                "data-editor-kind": "checkbox", "data-retiring": retiring.to_string(),
                "aria-label": label, "aria-invalid": invalid, "aria-errormessage": error_message,
                onfocus: focus, onkeydown: keydown, onclick: pointer_guard, onmousedown: pointer_guard,
                onchange: move |event: FormEvent| { if retiring { event.prevent_default(); } else { update.call(event.checked().to_string()); } },
            }
        },
    };
    rsx! { div { class: "ix-data-grid-editor-wrap", {control} } }
}

pub(crate) const STYLES: &str = include_str!("styles/data_grid.css");

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "data_grid/pagination_tests.rs"]
mod pagination_tests;
