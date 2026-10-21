use super::*;
use crate::DataGridPagination;

fn column() -> DataGridColumn {
    let mut column = DataGridColumn::new("value", "Value");
    column.sortable = true;
    column.filterable = true;
    column.editable = true;
    column.row_drag = true;
    column.value_type = DataGridValueType::Number;
    column
}
fn rows() -> Vec<DataGridRow> {
    vec![
        DataGridRow::new("zeta").cell("value", "9"),
        DataGridRow::new("alpha").cell("value", "2"),
        DataGridRow::new("overflow").cell("value", "100"),
    ]
}
fn query() -> DataGridState {
    DataGridState {
        pagination: Some(DataGridPagination::new(1, 2)),
        sort: Some(("value".into(), DataGridSortDirection::Ascending)),
        filters: BTreeMap::from([(
            "value".into(),
            DataGridTextFilter {
                first: DataGridFilterCondition {
                    operator: DataGridFilterOperator::Equals,
                    value: "999".into(),
                },
                ..Default::default()
            },
        )]),
        row_order: vec!["alpha".into(), "zeta".into()],
        selected: BTreeSet::from(["remote-selected".into()]),
        ..Default::default()
    }
}
#[test]
fn host_projection_preserves_supplied_query_result_and_range_selection_without_client_reprocessing()
{
    let rows = rows();
    let columns = [column()];
    let mut state = query();
    let model = DataGridRowModel::Page { total_rows: 9 };
    assert_eq!(
        model
            .projected_rows(&state, &rows, &columns)
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        ["zeta", "alpha"]
    );
    assert!(
        state.paged_rows_with_columns(&rows, &columns).is_empty(),
        "existing client helper retains typed filtering"
    );
    select_grid_row(&mut state, model, &rows, &columns, "zeta", true, false);
    select_grid_row(&mut state, model, &rows, &columns, "alpha", true, true);
    assert_eq!(
        state.selected,
        BTreeSet::from(["remote-selected".into(), "zeta".into(), "alpha".into()])
    );
    assert_eq!(state.sort, query().sort);
    assert_eq!(state.filters, query().filters);
    assert_eq!(state.row_order, query().row_order);
    assert_eq!(
        next_editable_in_rows(
            model.projected_rows(&state, &rows, &columns),
            &columns,
            &state,
            "zeta",
            "value",
            false
        ),
        Some(("alpha".into(), "value".into()))
    );
    state.pagination = Some(DataGridPagination::new(usize::MAX, 2));
    assert_eq!(
        model.projected_rows(&state, &rows, &columns).len(),
        1,
        "last effective page has one row"
    );
    assert!(
        DataGridRowModel::Page { total_rows: 0 }
            .projected_rows(&state, &rows, &columns)
            .is_empty()
    );
    state.pagination = None;
    assert!(
        model.projected_rows(&state, &rows, &columns).is_empty(),
        "missing pagination never falls back to client projection"
    );
}
#[derive(Default)]
struct Capture {
    state: Cell<Option<Signal<DataGridState>>>,
    rows: Cell<Option<Signal<Vec<DataGridRow>>>>,
    model: Cell<Option<Signal<DataGridRowModel>>>,
    editor: RefCell<Option<DataGridEditorContext>>,
    requests: RefCell<Vec<DataGridState>>,
    autosize_ids: RefCell<Vec<String>>,
}
impl PartialEq for Capture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn Probe(data: Rc<Capture>, value: String) -> Element {
    let autosize = use_context::<GridAutosizeContext>();
    *data.autosize_ids.borrow_mut() = autosize
        .inputs
        .borrow()
        .rows
        .iter()
        .map(|row| row.id.clone())
        .collect();
    rsx! {span {"{value}"}}
}
fn fixture(data: Rc<Capture>) -> Element {
    let mut state = use_signal(query);
    let mut rows = use_signal(rows);
    let model = use_signal(|| DataGridRowModel::Page { total_rows: 9 });
    data.state.set(Some(state));
    data.rows.set(Some(rows));
    data.model.set(Some(model));
    let requests = data.clone();
    let render = data.clone();
    let editor = data.clone();
    rsx! {DataGrid {columns:vec![column()],rows:rows(),state:state(),row_model:model(),label:"Host records",page_size_options:vec![1,2,3],tooltip_delay_ms:0,
        onstatechange:move|next:DataGridState|{requests.requests.borrow_mut().push(next.clone());state.set(next);},
        oncellvaluechanged:move|change:DataGridCellValueChanged|rows.with_mut(|rows|{change.apply_to(rows);}),
        render_cell:Callback::new(move|context:DataGridCellContext|Some(DataGridCellRender::display(rsx!{Probe {data:render.clone(),value:context.value}}))),
        render_editor:Callback::new(move|context:DataGridEditorContext|{*editor.editor.borrow_mut()=Some(context);None})
    }}
}
fn collect(edits: dioxus::core::Mutations, attrs: &mut BTreeMap<usize, BTreeMap<String, String>>) {
    // Only record attributes for an explicitly registered/selected node. Stack
    // mutations can select unregistered template nodes, so they invalidate the
    // current ID instead of attributing their static attributes to its owner.
    let mut current = None;
    for edit in edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } => {
                attrs.insert(id.raw(), BTreeMap::new());
                current = Some(id);
            }
            dioxus::core::Mutation::PushId { id } => current = Some(id),
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } => {
                if let Some(id) = current {
                    attrs.entry(id.raw()).or_default().insert(name, value);
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

fn settle(dom: &mut VirtualDom, attrs: &mut BTreeMap<usize, BTreeMap<String, String>>) {
    for _ in 0..6 {
        collect(dom.render_immediate_to_vec(), attrs);
        dom.process_events();
    }
}
fn setup() -> (
    VirtualDom,
    Rc<Capture>,
    BTreeMap<usize, BTreeMap<String, String>>,
) {
    let data = Rc::new(Capture::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut attrs = BTreeMap::new();
    collect(dom.rebuild_to_vec(), &mut attrs);
    settle(&mut dom, &mut attrs);
    (dom, data, attrs)
}
fn target(
    attrs: &BTreeMap<usize, BTreeMap<String, String>>,
    name: &str,
    value: &str,
) -> dioxus::core::ElementId {
    dioxus::core::ElementId::from_raw(
        *attrs
            .iter()
            .find(|(_, map)| {
                map.get(name).map(String::as_str) == Some(value)
                    && (name != "data-column-id" || map.contains_key("aria-readonly"))
            })
            .unwrap_or_else(|| {
                panic!(
                    "missing {name}={value}; available labels: {:?}",
                    attrs
                        .values()
                        .filter_map(|map| map.get("aria-label"))
                        .collect::<Vec<_>>()
                )
            })
            .0,
    )
}
fn opening<'a>(html: &'a str, marker: &str) -> &'a str {
    html.split('<')
        .find(|tag| tag.contains(marker))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else {
        value.split_whitespace().next()
    }
}

#[test]
fn actual_host_grid_uses_global_metadata_loaded_autosizing_and_explicit_missing_pagination_status()
{
    let (mut dom, data, mut attrs) = setup();
    let html = dioxus_ssr::render(&dom);
    assert!(opening(&html, "data-row-model=").contains("data-row-model=\"page\""));
    assert_eq!(
        attr(
            opening(&html, "aria-label=\"Host records\""),
            "aria-rowcount"
        ),
        Some("10")
    );
    assert_eq!(
        attr(opening(&html, "data-row-id=\"zeta\""), "aria-rowindex"),
        Some("4")
    );
    assert_eq!(
        attr(opening(&html, "data-row-id=\"alpha\""), "aria-rowindex"),
        Some("5")
    );
    assert!(!html.contains("data-row-id=\"overflow\""));
    assert!(
        html.find("data-row-id=\"zeta\"").unwrap() < html.find("data-row-id=\"alpha\"").unwrap()
    );
    assert_eq!(*data.autosize_ids.borrow(), ["zeta", "alpha"]);
    assert!(opening(&html, "aria-label=\"Move 9\"").contains("disabled"));
    dom.in_runtime(|| {
        data.state
            .get()
            .unwrap()
            .with_mut(|state| state.pagination = None)
    });
    settle(&mut dom, &mut attrs);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Host-paged rows require pagination."));
    assert!(!html.contains("data-row-id="));
    dom.in_runtime(|| {
        data.state
            .get()
            .unwrap()
            .with_mut(|state| state.pagination = Some(DataGridPagination::new(0, 2)));
        data.rows.get().unwrap().set(Vec::new());
    });
    settle(&mut dom, &mut attrs);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("No Rows To Show"));
    assert_eq!(
        attr(
            opening(&html, "aria-label=\"Host records\""),
            "aria-rowcount"
        ),
        Some("10")
    );
    dom.in_runtime(|| data.model.get().unwrap().set(DataGridRowModel::Client));
    settle(&mut dom, &mut attrs);
    assert!(!dioxus_ssr::render(&dom).contains("data-row-model="));
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
struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Keyboard(Key);
impl dioxus::html::ModifiersInteraction for Keyboard {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for Keyboard {
    fn key(&self) -> Key {
        self.0.clone()
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
struct CellMount;
impl dioxus::html::RenderedElementBacking for CellMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        _: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        Box::pin(async { Ok(()) })
    }
    fn get_client_rect(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>,
                >,
        >,
    > {
        Box::pin(async {
            Ok(dioxus::html::geometry::PixelsRect::new(
                dioxus::html::geometry::euclid::Point2D::new(100.0, 100.0),
                dioxus::html::geometry::euclid::Size2D::new(80.0, 24.0),
            ))
        })
    }
}
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*}
    };
    (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(Focus)};
    (@data Keyboard,$event:ident)=>{dioxus::html::KeyboardData::new(Keyboard($event.downcast::<Key>().unwrap().clone()))};
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(Mouse)};
    (@data Mounted,$event:ident)=>{MountedData::new(CellMount)};
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(Form($event.downcast::<String>().unwrap().clone()))};
    (@data $other:ident,$event:ident)=>{panic!("unexpected grid help test event")};
}
dioxus::html::with_html_event_groups!(converter);
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    target: dioxus::core::ElementId,
    payload: impl std::any::Any,
) -> bool {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    let event = Event::new(
        data,
        !matches!(
            name,
            "mounted" | "focus" | "blur" | "mouseenter" | "mouseleave"
        ),
    );
    dom.runtime().handle_event(name, event.clone(), target);
    event.propagates()
}

#[test]
fn actual_host_requests_preserve_query_selection_and_page_local_editing() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, mut attrs) = setup();
    // Selection and sorting retain this column and its Equals condition. Keep
    // its registered input target rather than treating the attribute lookup
    // helper (which does not interpret the mutation stack) as a DOM inventory.
    let first_filter = target(&attrs, "aria-label", "Value filter value 1");

    dispatch(
        &dom,
        "click",
        target(&attrs, "aria-label", "Select all loaded rows"),
        (),
    );
    settle(&mut dom, &mut attrs);
    assert_eq!(
        data.state.get().unwrap().peek().selected,
        BTreeSet::from(["remote-selected".into(), "zeta".into(), "alpha".into()])
    );
    assert!(
        opening(
            &dioxus_ssr::render(&dom),
            "aria-label=\"Select all loaded rows\""
        )
        .contains("aria-checked=\"true\"")
    );
    dispatch(&dom, "click", target(&attrs, "aria-label", "Select 2"), ());
    settle(&mut dom, &mut attrs);
    assert!(
        opening(
            &dioxus_ssr::render(&dom),
            "aria-label=\"Select all loaded rows\""
        )
        .contains("aria-checked=\"mixed\"")
    );
    dispatch(
        &dom,
        "click",
        target(&attrs, "aria-label", "Select all loaded rows"),
        (),
    );
    settle(&mut dom, &mut attrs);
    dispatch(
        &dom,
        "click",
        target(&attrs, "aria-label", "Sort Value"),
        (),
    );
    settle(&mut dom, &mut attrs);
    assert_eq!(data.state.get().unwrap().peek().pagination.unwrap().page, 0);
    assert_eq!(
        data.state.get().unwrap().peek().sort,
        Some(("value".into(), DataGridSortDirection::Descending))
    );
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        attr(
            opening(&html, "aria-label=\"Value filter value 1\""),
            "value"
        ),
        Some("999")
    );
    dispatch(&dom, "input", first_filter, "1000".to_string());
    settle(&mut dom, &mut attrs);
    assert_eq!(
        data.state.get().unwrap().peek().filters["value"]
            .first
            .value,
        "1000"
    );
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("data-row-id=\"zeta\"") && html.contains("data-row-id=\"alpha\""));
    let cell = target(&attrs, "data-column-id", "value");
    dispatch(&dom, "focus", cell, ());
    dispatch(&dom, "keydown", cell, Key::F2);
    settle(&mut dom, &mut attrs);
    assert!(
        opening(
            &dioxus_ssr::render(&dom),
            "class=\"ix-data-grid-pagination\""
        )
        .contains("disabled")
    );
    let count = data.requests.borrow().len();
    dispatch(
        &dom,
        "change",
        target(&attrs, "aria-label", "Items per page"),
        "1".to_string(),
    );
    settle(&mut dom, &mut attrs);
    assert_eq!(
        data.requests.borrow().len(),
        count,
        "active editor rejects pager requests"
    );
    let editor = data.editor.borrow().clone().unwrap();
    assert_eq!(editor.cell.row.id, "zeta");
    dom.in_runtime(|| editor.set_value("15".into()));
    settle(&mut dom, &mut attrs);
    dom.in_runtime(|| data.editor.borrow().as_ref().unwrap().advance(false));
    settle(&mut dom, &mut attrs);
    assert_eq!(data.rows.get().unwrap().peek()[0].value("value"), "15");
    assert_eq!(
        data.editor.borrow().as_ref().unwrap().cell.row.id,
        "alpha",
        "Tab follows host order despite contradictory local query"
    );
    dom.in_runtime(|| data.editor.borrow().as_ref().unwrap().cancel());
    settle(&mut dom, &mut attrs);
    dispatch(
        &dom,
        "change",
        target(&attrs, "aria-label", "Items per page"),
        "1".to_string(),
    );
    settle(&mut dom, &mut attrs);
    assert_eq!(
        data.state.get().unwrap().peek().pagination,
        Some(DataGridPagination::new(0, 1))
    );
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("data-row-id=\"zeta\""));
    assert!(!html.contains("data-row-id=\"alpha\""));
    assert!(
        data.state
            .get()
            .unwrap()
            .peek()
            .selected
            .contains("remote-selected")
    );
    assert_eq!(
        data.state.get().unwrap().peek().filters["value"]
            .first
            .value,
        "1000"
    );
}
