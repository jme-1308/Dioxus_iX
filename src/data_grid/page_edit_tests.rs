use super::*;
use std::{future::Future, pin::Pin};

#[derive(Clone, PartialEq)]
struct Settings {
    across: bool,
    host: bool,
    auto: bool,
    loading: bool,
    read_only: bool,
    mounted: bool,
}
struct Observation {
    initial: DataGridState,
    settings: Settings,
    rows: Cell<Option<Signal<Vec<DataGridRow>>>>,
    state: Cell<Option<Signal<DataGridState>>>,
    options: Cell<Option<Signal<Settings>>>,
    requests: RefCell<Vec<DataGridState>>,
    changes: RefCell<Vec<DataGridCellValueChanged>>,
    focuses: Rc<RefCell<Vec<String>>>,
    echo: Cell<bool>,
    loading_page: Cell<Option<usize>>,
}
impl Observation {
    fn new(across: bool, page: usize) -> Rc<Self> {
        Rc::new(Self {
            initial: DataGridState {
                pagination: Some(crate::DataGridPagination::new(page, 2)),
                ..Default::default()
            },
            settings: Settings {
                across,
                host: false,
                auto: false,
                loading: false,
                read_only: false,
                mounted: true,
            },
            rows: Cell::new(None),
            state: Cell::new(None),
            options: Cell::new(None),
            requests: RefCell::new(Vec::new()),
            changes: RefCell::new(Vec::new()),
            focuses: Rc::new(RefCell::new(Vec::new())),
            echo: Cell::new(false),
            loading_page: Cell::new(None),
        })
    }
}
fn records() -> Vec<DataGridRow> {
    (0..6)
        .map(|index| {
            DataGridRow::new(format!("row{index}"))
                .cell("value", format!("Value{index}"))
                .cell("note", format!("Note{index}"))
                .cell("group", ["A", "B", "C"][index / 2])
        })
        .collect()
}
fn columns() -> Vec<DataGridColumn> {
    ["value", "note"]
        .into_iter()
        .map(|id| {
            let mut column = DataGridColumn::new(id, if id == "value" { "Value" } else { "Note" });
            column.editable = true;
            column.editor.required = true;
            column
        })
        .chain(std::iter::once(DataGridColumn::new("group", "Group")))
        .collect()
}
fn fixture(data: Rc<Observation>) -> Element {
    let mut rows = use_signal(records);
    let mut state = use_signal(|| data.initial.clone());
    let options = use_signal(|| data.settings.clone());
    data.rows.set(Some(rows));
    data.state.set(Some(state));
    data.options.set(Some(options));
    let current = options();
    let source = rows();
    let accepted = state();
    // The host accepts query, logical total, grouped page and global summaries
    // together. The Grid never receives an invented or constant first page.
    let projected = accepted.visible_rows_with_columns(&source, &columns());
    let total = projected.len();
    let groups = accepted.page_group_metadata(&source, &columns());
    let supplied = if current.host {
        let page = accepted.pagination_info(total);
        projected[page.range]
            .iter()
            .map(|row| (*row).clone())
            .collect()
    } else {
        source
    };
    let requests = data.clone();
    let changes = data.clone();
    rsx! {
      input { id: String::from("outside-page-grid"), onfocus: move |_| {} }
      if current.mounted { DataGrid {
        columns: columns(), rows: supplied, state: accepted, label: "Cross-page worklist",
        row_model: if current.host { DataGridRowModel::Page { total_rows: total } } else { DataGridRowModel::Client },
        page_groups: current.host.then_some(groups),
        tab_across_pages: current.across, auto_page_size: current.auto, page_loading: current.loading,
        read_only: current.read_only,
        onstatechange: move |next: DataGridState| {
            requests.requests.borrow_mut().push(next.clone());
            if requests.loading_page.get().is_some_and(|page| next.pagination.is_some_and(|pagination| pagination.page == page)) {
                requests.options.get().unwrap().with_mut(|settings| settings.loading = true);
            } else if requests.echo.get() { state.set(next); }
        },
        oncellvaluechanged: move |change: DataGridCellValueChanged| {
            changes.changes.borrow_mut().push(change.clone());
            assert!(change.apply_to(&mut rows.write()));
        },
        render_cell: move |context: DataGridCellContext| Some(DataGridCellRender::display(rsx! {
            span { "data-page-cell": format!("{}:{}", context.row.id, context.column.id), "{context.value}" }
        })),
      } }
    }
}

#[derive(Clone)]
struct Mount {
    label: String,
    focuses: Rc<RefCell<Vec<String>>>,
}
impl dioxus::html::RenderedElementBacking for Mount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        if focus {
            self.focuses.borrow_mut().push(self.label.clone());
        }
        Box::pin(async { Ok(()) })
    }
    fn get_client_rect(
        &self,
    ) -> Pin<
        Box<dyn Future<Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>>>,
    > {
        Box::pin(async {
            Ok(dioxus::html::geometry::PixelsRect::from_size(
                dioxus::html::geometry::PixelsSize::new(640.0, 192.0),
            ))
        })
    }
    fn get_scroll_offset(
        &self,
    ) -> Pin<
        Box<
            dyn Future<
                Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsVector2D>,
            >,
        >,
    > {
        Box::pin(async { Ok(dioxus::html::geometry::PixelsVector2D::zero()) })
    }
    fn scroll(
        &self,
        _: dioxus::html::geometry::PixelsVector2D,
        _: dioxus::html::ScrollBehavior,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        Box::pin(async { Ok(()) })
    }
}
struct Form(String);
impl dioxus::html::HasFileData for Form {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for Form {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        Vec::new()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
struct Keyboard {
    key: Key,
    backwards: bool,
}
impl dioxus::html::ModifiersInteraction for Keyboard {
    fn modifiers(&self) -> Modifiers {
        if self.backwards {
            Modifiers::SHIFT
        } else {
            Modifiers::empty()
        }
    }
}
impl dioxus::html::HasKeyboardData for Keyboard {
    fn key(&self) -> Key {
        self.key.clone()
    }
    fn code(&self) -> dioxus::html::Code {
        dioxus::html::Code::Unidentified
    }
    fn location(&self) -> dioxus::html::Location {
        dioxus::html::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Mouse;
impl dioxus::html::InteractionLocation for Mouse {
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
impl dioxus::html::InteractionElementOffset for Mouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for Mouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for Mouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for Mouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl dioxus::html::HasPointerData for Mouse {
    fn pointer_id(&self) -> i32 {
        1
    }
    fn width(&self) -> f64 {
        1.0
    }
    fn height(&self) -> f64 {
        1.0
    }
    fn pressure(&self) -> f32 {
        0.5
    }
    fn tangential_pressure(&self) -> f32 {
        0.0
    }
    fn tilt_x(&self) -> i32 {
        0
    }
    fn tilt_y(&self) -> i32 {
        0
    }
    fn twist(&self) -> i32 {
        0
    }
    fn pointer_type(&self) -> String {
        "mouse".into()
    }
    fn is_primary(&self) -> bool {
        true
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Resize(f64);
impl dioxus::html::HasResizeData for Resize {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_content_box_size(
        &self,
    ) -> dioxus::html::ResizeResult<dioxus::html::geometry::PixelsSize> {
        Ok(dioxus::html::geometry::PixelsSize::new(640.0, self.0))
    }
}
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*}) => {
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data { let _=event; converter!(@data $group,event) })*}
    };
    (@data Mounted,$event:ident) => { MountedData::new($event.downcast::<Mount>().unwrap().clone()) };
    (@data Form,$event:ident) => { dioxus::html::FormData::new(Form($event.downcast::<String>().unwrap().clone())) };
    (@data Keyboard,$event:ident) => { dioxus::html::KeyboardData::new($event.downcast::<Keyboard>().unwrap().clone()) };
    (@data Focus,$event:ident) => { dioxus::html::FocusData::new(Focus) };
    (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
    (@data Pointer,$event:ident) => { dioxus::html::PointerData::new(Mouse) };
    (@data Resize,$event:ident) => { dioxus::html::ResizeData::new(Resize(*$event.downcast::<f64>().unwrap())) };
    (@data $other:ident,$event:ident) => { panic!("unexpected cross-page grid event") };
}
dioxus::html::with_html_event_groups!(converter);

#[derive(Default)]
struct Targets {
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    mounted: Vec<dioxus::core::ElementId>,
}
impl Targets {
    fn collect(&mut self, edits: dioxus::core::Mutations) {
        let mut current = None;
        for edit in edits.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } => {
                    self.attrs.insert(id.raw(), BTreeMap::new());
                    current = Some(id);
                }
                dioxus::core::Mutation::PushId { id } => current = Some(id),
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } => {
                    if let Some(id) = current {
                        self.attrs.entry(id.raw()).or_default().insert(name, value);
                    }
                }
                dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                    if let Some(id) = current {
                        self.mounted.push(id);
                    }
                }
                dioxus::core::Mutation::CreateElement { .. }
                | dioxus::core::Mutation::CreateText { .. }
                | dioxus::core::Mutation::Child { .. }
                | dioxus::core::Mutation::Pop
                | dioxus::core::Mutation::Clone
                | dioxus::core::Mutation::AppendChildren { .. }
                | dioxus::core::Mutation::ReplaceWith { .. }
                | dioxus::core::Mutation::InsertBefore { .. }
                | dioxus::core::Mutation::InsertAfter { .. }
                | dioxus::core::Mutation::Remove => current = None,
                _ => {}
            }
        }
    }
    fn target(&self, name: &str, value: &str) -> dioxus::core::ElementId {
        dioxus::core::ElementId::from_raw(
            *self
                .attrs
                .iter()
                .find(|(_, attrs)| attrs.get(name).is_some_and(|actual| actual == value))
                .unwrap_or_else(|| panic!("missing {name}={value}"))
                .0,
        )
    }
}
fn dispatch(dom: &VirtualDom, name: &str, id: dioxus::core::ElementId, value: impl std::any::Any) {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(value))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event(name, Event::new(data, name != "mounted"), id);
}
fn settle(dom: &mut VirtualDom, targets: &mut Targets, data: &Observation) {
    for _ in 0..6 {
        targets.collect(dom.render_immediate_to_vec());
        for id in std::mem::take(&mut targets.mounted) {
            let label = targets
                .attrs
                .get(&id.raw())
                .and_then(|attrs| attrs.get("aria-label").or_else(|| attrs.get("id")))
                .cloned()
                .unwrap_or_default();
            dispatch(
                dom,
                "mounted",
                id,
                Mount {
                    label,
                    focuses: data.focuses.clone(),
                },
            );
        }
        dom.process_events();
    }
}
fn setup(data: Rc<Observation>) -> (VirtualDom, Targets) {
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    settle(&mut dom, &mut targets, &data);
    (dom, targets)
}
fn label(row: &str, column: &str) -> String {
    format!(
        "Edit {} for {row}",
        if column == "value" { "Value" } else { "Note" }
    )
}
fn begin(dom: &mut VirtualDom, targets: &mut Targets, data: &Observation, row: &str, column: &str) {
    // A display span has no native event listener registration of its own.
    // Address the actual cell, then exercise its ordinary focus/F2 path.
    let cell = cell_dom_id(dom, row, column);
    let cell = targets.target("id", &cell);
    dispatch(dom, "focus", cell, ());
    dispatch(
        dom,
        "keydown",
        cell,
        Keyboard {
            key: Key::F2,
            backwards: false,
        },
    );
    settle(dom, targets, data);
    dispatch(
        dom,
        "focus",
        targets.target("aria-label", &label(row, column)),
        (),
    );
    settle(dom, targets, data);
}
fn type_value(
    dom: &mut VirtualDom,
    targets: &mut Targets,
    data: &Observation,
    row: &str,
    column: &str,
    value: &str,
) {
    dispatch(
        dom,
        "input",
        targets.target("aria-label", &label(row, column)),
        value.to_string(),
    );
    settle(dom, targets, data);
}
fn tab(dom: &VirtualDom, targets: &Targets, row: &str, column: &str, backwards: bool) {
    dispatch(
        dom,
        "keydown",
        targets.target("aria-label", &label(row, column)),
        Keyboard {
            key: Key::Tab,
            backwards,
        },
    );
}
fn accept(dom: &mut VirtualDom, targets: &mut Targets, data: &Observation) {
    let next = data.requests.borrow().last().unwrap().clone();
    dom.in_runtime(|| data.state.get().unwrap().set(next));
    settle(dom, targets, data);
}
fn editor_tag<'a>(html: &'a str, row: &str, column: &str) -> &'a str {
    html.split('<')
        .find(|tag| {
            tag.starts_with("input ")
                && tag.contains(&format!("aria-label=\"{}\"", label(row, column)))
        })
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn cell_dom_id(dom: &VirtualDom, row: &str, column: &str) -> String {
    let html = dioxus_ssr::render(dom);
    let before = html
        .split_once(&format!("data-page-cell=\"{row}:{column}\""))
        .unwrap()
        .0;
    let cell = before
        .rsplit("<div ")
        .find(|tag| {
            tag.split('>')
                .next()
                .unwrap()
                .contains("class=\"ix-data-grid-cell\"")
        })
        .unwrap();
    cell.split_once("id=\"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap()
        .to_string()
}
fn assert_idle(dom: &VirtualDom) {
    let html = dioxus_ssr::render(dom);
    assert!(!html.contains("class=\"ix-data-grid-editor\""), "{html}");
    let pager = html
        .split('<')
        .find(|tag| tag.starts_with("fieldset "))
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(
        !pager
            .split_whitespace()
            .any(|attr| attr == "disabled" || attr.starts_with("disabled=")),
        "{pager}"
    );
}

#[test]
fn actual_tab_commits_once_then_accepted_page_opens_and_focuses_the_next_editor() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Observation::new(true, 0);
    let (mut dom, mut targets) = setup(data.clone());
    begin(&mut dom, &mut targets, &data, "row1", "note");
    type_value(&mut dom, &mut targets, &data, "row1", "note", "Prepared");
    tab(&dom, &targets, "row1", "note", false);
    assert_eq!(data.changes.borrow().len(), 1);
    assert_eq!(data.requests.borrow().len(), 1);
    assert_eq!(data.rows.get().unwrap().peek()[1].value("note"), "Prepared");
    assert_eq!(data.requests.borrow()[0].pagination.unwrap().page, 1);
    accept(&mut dom, &mut targets, &data);
    assert!(editor_tag(&dioxus_ssr::render(&dom), "row2", "value").contains("value=\"Value2\""));
    assert!(
        data.focuses
            .borrow()
            .iter()
            .any(|label| label == "Edit Value for row2")
    );
    dispatch(
        &dom,
        "focus",
        targets.target("aria-label", "Edit Value for row2"),
        (),
    );
    type_value(&mut dom, &mut targets, &data, "row2", "value", "Inspected");
    tab(&dom, &targets, "row2", "value", true);
    assert_eq!(
        data.requests
            .borrow()
            .last()
            .unwrap()
            .pagination
            .unwrap()
            .page,
        0
    );
    accept(&mut dom, &mut targets, &data);
    assert!(editor_tag(&dioxus_ssr::render(&dom), "row1", "note").contains("value=\"Prepared\""));
    assert_eq!(data.changes.borrow().len(), 2);
    dom.in_runtime(|| {
        data.options
            .get()
            .unwrap()
            .with_mut(|settings| settings.auto = true)
    });
    settle(&mut dom, &mut targets, &data);
    dispatch(
        &dom,
        "resize",
        targets.target("aria-label", "Cross-page worklist"),
        90.0,
    );
    settle(&mut dom, &mut targets, &data);
    assert_eq!(
        data.requests.borrow().len(),
        2,
        "the new active editor defers automatic sizing"
    );
    assert!(editor_tag(&dioxus_ssr::render(&dom), "row1", "note").contains("value=\"Prepared\""));
}

#[test]
fn invalid_boundary_draft_does_not_publish_or_request_a_page() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Observation::new(true, 0);
    let (mut dom, mut targets) = setup(data.clone());
    begin(&mut dom, &mut targets, &data, "row1", "note");
    type_value(&mut dom, &mut targets, &data, "row1", "note", "");
    tab(&dom, &targets, "row1", "note", false);
    settle(&mut dom, &mut targets, &data);
    assert!(data.changes.borrow().is_empty() && data.requests.borrow().is_empty());
    assert!(
        editor_tag(&dioxus_ssr::render(&dom), "row1", "note").contains("aria-invalid=\"true\"")
    );
    type_value(&mut dom, &mut targets, &data, "row1", "note", "Corrected");
    tab(&dom, &targets, "row1", "note", false);
    accept(&mut dom, &mut targets, &data);
    assert_eq!(data.changes.borrow().len(), 1);
    assert_eq!(data.requests.borrow().len(), 1);
    assert!(editor_tag(&dioxus_ssr::render(&dom), "row2", "value").contains("value=\"Value2\""));
}

#[test]
fn unaccepted_or_changed_owner_echo_cannot_leave_hidden_editors_or_late_focus() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for refusal in ["held", "other-page", "removed", "readonly", "filtered"] {
        let data = Observation::new(true, 0);
        let (mut dom, mut targets) = setup(data.clone());
        let origin_id = cell_dom_id(&dom, "row1", "note");
        begin(&mut dom, &mut targets, &data, "row1", "note");
        type_value(&mut dom, &mut targets, &data, "row1", "note", "Saved once");
        tab(&dom, &targets, "row1", "note", false);
        let requested = data.requests.borrow().last().unwrap().clone();
        dom.in_runtime(|| {
            let mut accepted = requested.clone();
            match refusal {
                "held" => return,
                "other-page" => accepted.pagination.as_mut().unwrap().page = 2,
                "removed" => data
                    .rows
                    .get()
                    .unwrap()
                    .write()
                    .retain(|row| row.id != "row2"),
                "readonly" => {
                    data.rows.get().unwrap().write()[2]
                        .read_only_cells
                        .insert("value".into());
                }
                "filtered" => {
                    accepted.filters.insert(
                        "value".into(),
                        DataGridTextFilter {
                            first: DataGridFilterCondition {
                                operator: DataGridFilterOperator::NotEqual,
                                value: "Value2".into(),
                            },
                            ..Default::default()
                        },
                    );
                }
                _ => unreachable!(),
            }
            data.state.get().unwrap().set(accepted);
        });
        settle(&mut dom, &mut targets, &data);
        assert_idle(&dom);
        assert_eq!(data.requests.borrow().len(), 1);
        assert_eq!(data.changes.borrow().len(), 1);
        assert!(
            !data
                .focuses
                .borrow()
                .iter()
                .any(|label| label == "Edit Value for row2"),
            "{refusal}"
        );
        if refusal == "held" {
            assert!(
                data.focuses
                    .borrow()
                    .iter()
                    .any(|label| label == &origin_id),
                "a rejected hop restores the still-visible origin cell"
            );
            let focus_count = data.focuses.borrow().len();
            dom.in_runtime(|| data.state.get().unwrap().set(requested));
            settle(&mut dom, &mut targets, &data);
            assert_idle(&dom);
            assert!(
                !data
                    .focuses
                    .borrow()
                    .iter()
                    .any(|label| label == "Edit Value for row2")
            );
            assert_eq!(
                data.focuses.borrow().len(),
                focus_count,
                "late acceptance cannot revive a retired focus job"
            );
        }
    }
}

#[test]
fn cross_page_target_uses_expanded_projection_readonly_cells_and_current_column_order() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let mut data = Observation::new(true, 0);
    let owner = Rc::get_mut(&mut data).unwrap();
    owner.initial.grouping = crate::DataGridGrouping::new(["group"]);
    owner
        .initial
        .set_group_expanded(&crate::DataGridGroupPath::new([("group", "B")]), false);
    owner.initial.column_order = vec!["note".into(), "value".into(), "group".into()];
    let (mut dom, mut targets) = setup(data.clone());
    dom.in_runtime(|| {
        data.rows.get().unwrap().write()[4]
            .read_only_cells
            .extend(["note".into(), "value".into()])
    });
    settle(&mut dom, &mut targets, &data);
    begin(&mut dom, &mut targets, &data, "row1", "value");
    tab(&dom, &targets, "row1", "value", false);
    assert_eq!(
        data.requests
            .borrow()
            .last()
            .unwrap()
            .pagination
            .unwrap()
            .page,
        2
    );
    accept(&mut dom, &mut targets, &data);
    assert!(editor_tag(&dioxus_ssr::render(&dom), "row5", "note").contains("value=\"Note5\""));
    assert!(
        data.focuses
            .borrow()
            .iter()
            .any(|label| label == "Edit Note for row5")
    );
    assert!(
        data.changes.borrow().is_empty(),
        "unchanged source still advances without publishing an edit"
    );
}

#[test]
fn default_page_local_finite_ends_and_host_boundary_remain_explicit() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for (across, page, row, column, backwards) in [
        (false, 0, "row1", "note", false),
        (true, 0, "row0", "value", true),
        (true, 2, "row5", "note", false),
    ] {
        let data = Observation::new(across, page);
        let (mut dom, mut targets) = setup(data.clone());
        begin(&mut dom, &mut targets, &data, row, column);
        tab(&dom, &targets, row, column, backwards);
        settle(&mut dom, &mut targets, &data);
        assert_idle(&dom);
        assert!(data.requests.borrow().is_empty() && data.changes.borrow().is_empty());
    }
    let mut data = Observation::new(false, 0);
    Rc::get_mut(&mut data).unwrap().settings.host = true;
    let (mut dom, mut targets) = setup(data.clone());
    assert!(!dioxus_ssr::render(&dom).contains("Cross-page editing requires client rows."));
    begin(&mut dom, &mut targets, &data, "row1", "note");
    tab(&dom, &targets, "row1", "note", false);
    settle(&mut dom, &mut targets, &data);
    assert_idle(&dom);
    assert!(data.requests.borrow().is_empty());
    dispatch(&dom, "click", targets.target("aria-label", "Next page"), ());
    assert_eq!(
        data.requests
            .borrow()
            .last()
            .unwrap()
            .pagination
            .unwrap()
            .page,
        1,
        "ordinary host page requests remain available"
    );
}

fn host(page: usize) -> Rc<Observation> {
    let mut data = Observation::new(true, page);
    Rc::get_mut(&mut data).unwrap().settings.host = true;
    data
}
fn finish_loading(
    dom: &mut VirtualDom,
    targets: &mut Targets,
    data: &Observation,
    accepted: DataGridState,
) {
    dom.in_runtime(|| {
        data.state.get().unwrap().set(accepted);
        data.options
            .get()
            .unwrap()
            .with_mut(|settings| settings.loading = false);
    });
    settle(dom, targets, data);
}
fn start_loading(dom: &mut VirtualDom, targets: &mut Targets, data: &Observation) -> DataGridState {
    begin(dom, targets, data, "row1", "note");
    type_value(dom, targets, data, "row1", "note", "Prepared for host");
    tab(dom, targets, "row1", "note", false);
    let requested = data.requests.borrow().last().unwrap().clone();
    dom.in_runtime(|| {
        data.options
            .get()
            .unwrap()
            .with_mut(|settings| settings.loading = true)
    });
    settle(dom, targets, data);
    requested
}
fn assert_loading(dom: &VirtualDom) {
    let html = dioxus_ssr::render(dom);
    let surface = html
        .split('<')
        .find(|tag| tag.contains("aria-label=\"Cross-page worklist\""))
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(
        surface.contains("aria-busy=\"true\"") || surface.contains("aria-busy=true"),
        "{surface}"
    );
    let pager = html
        .split('<')
        .find(|tag| tag.starts_with("fieldset "))
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(
        pager
            .split_whitespace()
            .any(|attr| attr == "disabled" || attr.starts_with("disabled=")),
        "{pager}"
    );
    assert!(
        !html.contains("class=\"ix-data-grid-editor\""),
        "loading owns no editable ghost"
    );
}

#[test]
fn host_boundary_validation_and_forward_reverse_echoes_use_committed_current_pages() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = host(0);
    let (mut dom, mut targets) = setup(data.clone());
    begin(&mut dom, &mut targets, &data, "row1", "note");
    type_value(&mut dom, &mut targets, &data, "row1", "note", "");
    tab(&dom, &targets, "row1", "note", false);
    settle(&mut dom, &mut targets, &data);
    assert!(data.changes.borrow().is_empty() && data.requests.borrow().is_empty());
    assert!(
        editor_tag(&dioxus_ssr::render(&dom), "row1", "note").contains("aria-invalid=\"true\"")
    );
    type_value(
        &mut dom,
        &mut targets,
        &data,
        "row1",
        "note",
        "Confirmed host note",
    );
    tab(&dom, &targets, "row1", "note", false);
    assert_eq!(data.requests.borrow().len(), 1);
    accept(&mut dom, &mut targets, &data);
    assert!(editor_tag(&dioxus_ssr::render(&dom), "row2", "value").contains("value=\"Value2\""));
    assert!(
        data.focuses
            .borrow()
            .iter()
            .any(|label| label == "Edit Value for row2")
    );
    dispatch(
        &dom,
        "focus",
        targets.target("aria-label", "Edit Value for row2"),
        (),
    );
    type_value(
        &mut dom,
        &mut targets,
        &data,
        "row2",
        "value",
        "Inspected host asset",
    );
    tab(&dom, &targets, "row2", "value", true);
    accept(&mut dom, &mut targets, &data);
    assert_eq!(
        data.requests
            .borrow()
            .last()
            .unwrap()
            .pagination
            .unwrap()
            .page,
        0
    );
    assert!(
        editor_tag(&dioxus_ssr::render(&dom), "row1", "note")
            .contains("value=\"Confirmed host note\"")
    );
    assert_eq!(data.changes.borrow().len(), 2);
    assert_eq!(
        data.rows.get().unwrap().peek()[2].value("value"),
        "Inspected host asset"
    );
}

#[test]
fn host_skips_whole_readonly_and_collapsed_pages_directionally_with_finite_ends() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = host(0);
    let (mut dom, mut targets) = setup(data.clone());
    data.echo.set(true);
    dom.in_runtime(|| {
        for row in &mut data.rows.get().unwrap().write()[2..4] {
            row.read_only_cells.extend(["value".into(), "note".into()]);
        }
    });
    settle(&mut dom, &mut targets, &data);
    begin(&mut dom, &mut targets, &data, "row1", "note");
    tab(&dom, &targets, "row1", "note", false);
    settle(&mut dom, &mut targets, &data);
    assert_eq!(
        data.requests
            .borrow()
            .iter()
            .map(|request| request.pagination.unwrap().page)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert!(editor_tag(&dioxus_ssr::render(&dom), "row4", "value").contains("value=\"Value4\""));
    dispatch(
        &dom,
        "focus",
        targets.target("aria-label", "Edit Value for row4"),
        (),
    );
    tab(&dom, &targets, "row4", "value", true);
    settle(&mut dom, &mut targets, &data);
    assert_eq!(
        data.requests
            .borrow()
            .last()
            .unwrap()
            .pagination
            .unwrap()
            .page,
        0
    );
    assert!(editor_tag(&dioxus_ssr::render(&dom), "row1", "note").contains("value=\"Note1\""));
    dispatch(
        &dom,
        "keydown",
        targets.target("aria-label", "Edit Note for row1"),
        Keyboard {
            key: Key::Escape,
            backwards: false,
        },
    );
    settle(&mut dom, &mut targets, &data);
    dom.in_runtime(|| {
        data.rows.get().unwrap().with_mut(|rows| {
            for row in rows {
                row.read_only_cells.clear();
            }
        });
        data.state.get().unwrap().with_mut(|state| {
            state.grouping = crate::DataGridGrouping::new(["group"]);
            state.set_group_expanded(&crate::DataGridGroupPath::new([("group", "B")]), false);
        });
    });
    settle(&mut dom, &mut targets, &data);
    let previous = data.requests.borrow().len();
    begin(&mut dom, &mut targets, &data, "row1", "note");
    tab(&dom, &targets, "row1", "note", false);
    settle(&mut dom, &mut targets, &data);
    assert_eq!(data.requests.borrow().len(), previous + 2);
    assert!(editor_tag(&dioxus_ssr::render(&dom), "row4", "value").contains("value=\"Value4\""));
    // End on the last editable cell; no page request may wrap back to page zero.
    dispatch(
        &dom,
        "keydown",
        targets.target("aria-label", "Edit Value for row4"),
        Keyboard {
            key: Key::Escape,
            backwards: false,
        },
    );
    settle(&mut dom, &mut targets, &data);
    begin(&mut dom, &mut targets, &data, "row5", "note");
    let previous = data.requests.borrow().len();
    tab(&dom, &targets, "row5", "note", false);
    settle(&mut dom, &mut targets, &data);
    assert_idle(&dom);
    assert_eq!(data.requests.borrow().len(), previous);
    assert!(data.changes.borrow().is_empty());

    let data = host(0);
    data.echo.set(true);
    let (mut dom, mut targets) = setup(data.clone());
    dom.in_runtime(|| {
        for row in &mut data.rows.get().unwrap().write()[2..] {
            row.read_only_cells.extend(["value".into(), "note".into()]);
        }
    });
    settle(&mut dom, &mut targets, &data);
    begin(&mut dom, &mut targets, &data, "row1", "note");
    tab(&dom, &targets, "row1", "note", false);
    settle(&mut dom, &mut targets, &data);
    assert_idle(&dom);
    assert_eq!(
        data.requests
            .borrow()
            .iter()
            .map(|request| request.pagination.unwrap().page)
            .collect::<Vec<_>>(),
        [1, 2],
        "readonly pages stop at the finite original boundary"
    );
}

#[test]
fn host_held_altered_query_and_changed_count_retire_the_original_continuation() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for response in ["held", "page", "query", "count"] {
        let data = host(0);
        let (mut dom, mut targets) = setup(data.clone());
        let origin = cell_dom_id(&dom, "row1", "note");
        begin(&mut dom, &mut targets, &data, "row1", "note");
        type_value(
            &mut dom,
            &mut targets,
            &data,
            "row1",
            "note",
            "Saved before response",
        );
        tab(&dom, &targets, "row1", "note", false);
        let requested = data.requests.borrow().last().unwrap().clone();
        dom.in_runtime(|| {
            if response == "held" {
                return;
            }
            let mut accepted = requested.clone();
            if response == "page" {
                accepted.pagination.as_mut().unwrap().page = 2;
            }
            if response == "query" {
                accepted.sort = Some(("value".into(), DataGridSortDirection::Descending));
            }
            if response == "count" {
                data.rows.get().unwrap().write().pop();
            }
            data.state.get().unwrap().set(accepted);
        });
        settle(&mut dom, &mut targets, &data);
        assert_idle(&dom);
        assert_eq!(data.requests.borrow().len(), 1);
        assert_eq!(data.changes.borrow().len(), 1);
        if response == "held" {
            assert!(data.focuses.borrow().iter().any(|label| label == &origin));
        }
        let focus_count = data.focuses.borrow().len();
        dom.in_runtime(|| data.state.get().unwrap().set(requested));
        settle(&mut dom, &mut targets, &data);
        assert_idle(&dom);
        assert_eq!(
            data.focuses.borrow().len(),
            focus_count,
            "late {response} echo cannot focus a new editor"
        );
    }
}

#[test]
fn host_loading_spans_renders_then_accepts_current_source_and_defers_auto_capacity() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = host(0);
    let (mut dom, mut targets) = setup(data.clone());
    begin(&mut dom, &mut targets, &data, "row0", "value");
    type_value(
        &mut dom,
        &mut targets,
        &data,
        "row0",
        "value",
        "Unpublished local draft",
    );
    dom.in_runtime(|| {
        data.options
            .get()
            .unwrap()
            .with_mut(|settings| settings.loading = true)
    });
    settle(&mut dom, &mut targets, &data);
    type_value(
        &mut dom,
        &mut targets,
        &data,
        "row0",
        "value",
        "Rejected while loading",
    );
    dispatch(
        &dom,
        "keydown",
        targets.target("aria-label", "Edit Value for row0"),
        Keyboard {
            key: Key::Enter,
            backwards: false,
        },
    );
    dispatch(&dom, "click", targets.target("aria-label", "Next page"), ());
    settle(&mut dom, &mut targets, &data);
    assert!(
        editor_tag(&dioxus_ssr::render(&dom), "row0", "value")
            .contains("value=\"Unpublished local draft\"")
    );
    assert!(data.changes.borrow().is_empty() && data.requests.borrow().is_empty());
    dom.in_runtime(|| {
        data.options
            .get()
            .unwrap()
            .with_mut(|settings| settings.loading = false)
    });
    settle(&mut dom, &mut targets, &data);
    assert!(
        editor_tag(&dioxus_ssr::render(&dom), "row0", "value")
            .contains("value=\"Unpublished local draft\"")
    );
    assert!(data.changes.borrow().is_empty() && data.requests.borrow().is_empty());
    type_value(
        &mut dom,
        &mut targets,
        &data,
        "row0",
        "value",
        "Resumed local draft",
    );
    assert!(
        editor_tag(&dioxus_ssr::render(&dom), "row0", "value")
            .contains("value=\"Resumed local draft\"")
    );
    dispatch(
        &dom,
        "keydown",
        targets.target("aria-label", "Edit Value for row0"),
        Keyboard {
            key: Key::Escape,
            backwards: false,
        },
    );
    settle(&mut dom, &mut targets, &data);
    assert_idle(&dom);
    let requested = start_loading(&mut dom, &mut targets, &data);
    assert_loading(&dom);
    dom.in_runtime(|| {
        data.options
            .get()
            .unwrap()
            .with_mut(|settings| settings.auto = true)
    });
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom, &mut targets, &data);
    dispatch(
        &dom,
        "resize",
        targets.target("aria-label", "Cross-page worklist"),
        90.0,
    );
    dispatch(&dom, "click", targets.target("aria-label", "Next page"), ());
    settle(&mut dom, &mut targets, &data);
    assert_loading(&dom);
    assert_eq!(
        data.requests.borrow().len(),
        1,
        "loading suppresses pager and automatic requests"
    );
    dom.in_runtime(|| {
        data.rows.get().unwrap().write()[2]
            .cells
            .insert("value".into(), "Current host response".into());
    });
    finish_loading(&mut dom, &mut targets, &data, requested);
    assert!(
        editor_tag(&dioxus_ssr::render(&dom), "row2", "value")
            .contains("value=\"Current host response\"")
    );
    assert!(
        data.focuses
            .borrow()
            .iter()
            .any(|label| label == "Edit Value for row2")
    );
    assert_eq!(
        data.requests.borrow().len(),
        1,
        "accepted editor keeps the automatic resize deferred"
    );
    assert_eq!(data.changes.borrow().len(), 1);

    // A real host can accept an uneditable page before the next reply loads.
    // Its mounted header remains a focus owner from which outside blur cancels.
    let data = host(0);
    data.echo.set(true);
    data.loading_page.set(Some(2));
    let (mut dom, mut targets) = setup(data.clone());
    dom.in_runtime(|| {
        for row in &mut data.rows.get().unwrap().write()[2..4] {
            row.read_only_cells.extend(["value".into(), "note".into()]);
        }
    });
    settle(&mut dom, &mut targets, &data);
    begin(&mut dom, &mut targets, &data, "row1", "note");
    tab(&dom, &targets, "row1", "note", false);
    settle(&mut dom, &mut targets, &data);
    assert_loading(&dom);
    assert_eq!(
        data.requests
            .borrow()
            .iter()
            .map(|state| state.pagination.unwrap().page)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(data.state.get().unwrap().peek().pagination.unwrap().page, 1);
    assert!(
        data.focuses
            .borrow()
            .iter()
            .any(|label| label == "Select all loaded rows")
    );
    dispatch(
        &dom,
        "focus",
        targets.target("aria-label", "Select all loaded rows"),
        (),
    );
    dispatch(
        &dom,
        "focusout",
        targets.target("aria-label", "Cross-page worklist"),
        (),
    );
    dispatch(&dom, "focus", targets.target("id", "outside-page-grid"), ());
    settle(&mut dom, &mut targets, &data);
    let focus_count = data.focuses.borrow().len();
    let accepted = data.requests.borrow().last().unwrap().clone();
    finish_loading(&mut dom, &mut targets, &data, accepted);
    assert_idle(&dom);
    assert_eq!(
        data.focuses.borrow().len(),
        focus_count,
        "late chained response must preserve outside focus"
    );
    assert!(data.changes.borrow().is_empty());
}

#[test]
fn independent_native_intents_and_owner_teardown_cancel_loading_before_late_echoes() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for intent in [
        "escape", "blur", "key", "pointer", "readonly", "option", "teardown",
    ] {
        let data = host(0);
        let (mut dom, mut targets) = setup(data.clone());
        let requested = start_loading(&mut dom, &mut targets, &data);
        let grid = targets.target("aria-label", "Cross-page worklist");
        match intent {
            "escape" | "key" => dispatch(
                &dom,
                "keydown",
                grid,
                Keyboard {
                    key: if intent == "escape" {
                        Key::Escape
                    } else {
                        Key::ArrowDown
                    },
                    backwards: false,
                },
            ),
            "blur" => {
                dispatch(&dom, "focusout", grid, ());
                dispatch(&dom, "focus", targets.target("id", "outside-page-grid"), ());
            }
            "pointer" => dispatch(&dom, "pointerdown", grid, ()),
            _ => dom.in_runtime(|| {
                data.options
                    .get()
                    .unwrap()
                    .with_mut(|settings| match intent {
                        "readonly" => settings.read_only = true,
                        "option" => settings.across = false,
                        "teardown" => settings.mounted = false,
                        _ => unreachable!(),
                    })
            }),
        }
        settle(&mut dom, &mut targets, &data);
        let focus_count = data.focuses.borrow().len();
        finish_loading(&mut dom, &mut targets, &data, requested);
        dom.in_runtime(|| {
            data.options.get().unwrap().with_mut(|settings| {
                settings.mounted = true;
                settings.read_only = false;
                settings.across = true;
            })
        });
        settle(&mut dom, &mut targets, &data);
        assert_idle(&dom);
        assert_eq!(
            data.focuses.borrow().len(),
            focus_count,
            "late response after {intent} stole focus"
        );
        assert_eq!(data.requests.borrow().len(), 1);
        assert_eq!(data.changes.borrow().len(), 1);
    }
}
