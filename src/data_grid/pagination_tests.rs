use super::*;
use crate::{DataGridCsvOptions, DataGridPagination};

#[test]
fn page_projection_clamps_empty_zero_and_large_requests_without_overflow() {
    for (request, total, expected) in [
        (DataGridPagination::new(9, 0), 5, (4, 1, 5, 4..5)),
        (DataGridPagination::new(usize::MAX, 2), 5, (2, 2, 3, 4..5)),
        (
            DataGridPagination::new(usize::MAX, usize::MAX),
            usize::MAX,
            (0, usize::MAX, 1, 0..usize::MAX),
        ),
        (
            DataGridPagination::new(usize::MAX, 1),
            usize::MAX,
            (usize::MAX - 1, 1, usize::MAX, usize::MAX - 1..usize::MAX),
        ),
        (DataGridPagination::new(12, 10), 0, (0, 10, 0, 0..0)),
    ] {
        let info = request.info(total);
        assert_eq!(
            (info.page, info.page_size, info.page_count, info.range),
            expected
        );
        assert_eq!(info.total_rows, total);
    }
    assert_eq!(DataGridState::default().pagination_info(7).range, 0..7);
    assert_eq!(DataGridState::default().pagination_info(0).page_count, 0);
}
#[test]
fn typed_page_navigation_keeps_full_filtered_selection_and_csv_intact() {
    let mut column = DataGridColumn::new("value", "Value");
    column.value_type = DataGridValueType::Number;
    column.editable = true;
    let columns = [column];
    let rows: Vec<_> = [
        ("a", "10"),
        ("b", "2"),
        ("c", "2.0"),
        ("d", "30"),
        ("e", "4"),
    ]
    .into_iter()
    .map(|(id, value)| DataGridRow::new(id).cell("value", value))
    .collect();
    let mut state = DataGridState {
        pagination: Some(DataGridPagination::new(1, 2)),
        sort: Some(("value".into(), DataGridSortDirection::Ascending)),
        row_order: vec!["c".into(), "b".into()],
        ..Default::default()
    };
    assert_eq!(
        state
            .paged_rows_with_columns(&rows, &columns)
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        ["e", "a"]
    );
    assert_eq!(
        next_editable_cell(&rows, &columns, &state, "e", "value", false),
        Some(("a".into(), "value".into()))
    );
    assert_eq!(
        next_editable_cell(&rows, &columns, &state, "a", "value", false),
        None,
        "Tab stops at the current page boundary"
    );
    state.select_filtered_with_columns(&rows, &columns, true);
    assert_eq!(state.selected.len(), 5);
    assert_eq!(
        state
            .export_csv(&rows, &columns, &DataGridCsvOptions::default())
            .row_ids,
        ["c", "b", "e", "a", "d"]
    );
    state.cycle_sort("value");
    assert_eq!(state.pagination.unwrap().page, 0);
    assert_eq!(state.selected.len(), 5);
}
#[derive(Default)]
struct Observation {
    initial: DataGridState,
    state: Cell<Option<Signal<DataGridState>>>,
    hidden: Cell<Option<Signal<bool>>>,
    requests: RefCell<Vec<DataGridState>>,
    autosize_rows: Cell<usize>,
}
impl PartialEq for Observation {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn AutosizeProbe(data: Rc<Observation>, value: String) -> Element {
    let context = use_context::<GridAutosizeContext>();
    data.autosize_rows.set(context.inputs.borrow().rows.len());
    rsx! {span {"{value}"}}
}
fn fixture(data: Rc<Observation>) -> Element {
    let state = use_signal(|| data.initial.clone());
    let hidden = use_signal(|| false);
    data.state.set(Some(state));
    data.hidden.set(Some(hidden));
    let mut column = DataGridColumn::new("asset", "Asset");
    column.editable = true;
    let rows = (0..5)
        .map(|index| DataGridRow::new(format!("row{index}")).cell("asset", format!("Asset{index}")))
        .collect::<Vec<_>>();
    let requests = data.clone();
    let render_data = data.clone();
    rsx! {DataGrid {columns:vec![column],rows,state:state(),onstatechange:move|next|requests.requests.borrow_mut().push(next),hide_pagination_controls:hidden(),label:"Paged equipment",page_size_options:vec![2,3],
        render_cell:Callback::new(move|context:DataGridCellContext|Some(DataGridCellRender::display(rsx!{AutosizeProbe {data:render_data.clone(),value:context.value}})))
    }}
}
fn collect(edits: dioxus::core::Mutations, attrs: &mut BTreeMap<usize, BTreeMap<String, String>>) {
    let mut current = dioxus::core::ElementId::ROOT;
    for edit in edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } => {
                attrs.entry(current.raw()).or_default().insert(name, value);
            }
            _ => {}
        }
    }
}
fn settle(dom: &mut VirtualDom, attrs: &mut BTreeMap<usize, BTreeMap<String, String>>) {
    for _ in 0..4 {
        collect(dom.render_immediate_to_vec(), attrs);
        dom.process_events();
    }
}
fn setup(data: Rc<Observation>) -> (VirtualDom, BTreeMap<usize, BTreeMap<String, String>>) {
    let mut dom = VirtualDom::new_with_props(fixture, data);
    let mut attrs = BTreeMap::new();
    collect(dom.rebuild_to_vec(), &mut attrs);
    settle(&mut dom, &mut attrs);
    (dom, attrs)
}
fn row_tag<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .find(|tag| tag.starts_with("div ") && tag.contains(&format!("data-row-id=\"{id}\"")))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
#[test]
fn actual_grid_default_has_no_pager_and_paged_rows_keep_global_aria_and_full_autosizing() {
    let data = Rc::new(Observation::default());
    let (mut dom, mut attrs) = setup(data.clone());
    assert!(!dioxus_ssr::render(&dom).contains("ix-data-grid-pagination"));
    dom.in_runtime(|| {
        data.state
            .get()
            .unwrap()
            .with_mut(|state| state.pagination = Some(DataGridPagination::new(1, 3)))
    });
    settle(&mut dom, &mut attrs);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("aria-label=\"Paged equipment pages\""));
    assert!(html.contains("aria-rowcount=6"));
    assert!(!html.contains("data-row-id=\"row0\""));
    let first = row_tag(&html, "row3");
    assert!(first.contains("aria-rowindex=5"));
    assert!(first.contains("data-odd=\"true\""));
    assert!(
        first.contains("top:0px"),
        "physical row offsets stay page-local"
    );
    assert!(row_tag(&html, "row4").contains("aria-rowindex=6"));
    assert_eq!(
        data.autosize_rows.get(),
        5,
        "autosizing sees all logical filtered rows, not two page rows"
    );
    dom.in_runtime(|| data.hidden.get().unwrap().set(true));
    settle(&mut dom, &mut attrs);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("ix-data-grid-pagination"));
    assert!(html.contains("data-row-id=\"row3\""));
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
struct Form(String);
impl dioxus::html::HasFileData for Form {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        vec![]
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
        vec![]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*}
    };
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(Mouse)};
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(Form($event.downcast::<String>().unwrap().clone()))};
    (@data $other:ident,$event:ident)=>{panic!("unexpected pagination fixture event")};
}
dioxus::html::with_html_event_groups!(converter);
fn dispatch(dom: &VirtualDom, name: &str, target: usize, payload: impl std::any::Any) {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    dom.runtime().handle_event(
        name,
        Event::new(data, true),
        dioxus::core::ElementId::from_raw(target),
    );
}
#[test]
fn actual_footer_emits_controlled_requests_and_native_fieldset_protects_active_edits() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Observation {
        initial: DataGridState {
            pagination: Some(DataGridPagination::new(1, 2)),
            ..Default::default()
        },
        ..Default::default()
    });
    let (mut dom, mut attrs) = setup(data.clone());
    let target = |label: &str| {
        *attrs
            .iter()
            .find(|(_, map)| map.get("aria-label").is_some_and(|value| value == label))
            .unwrap()
            .0
    };
    let next = target("Next page");
    let size = target("Items per page");
    dispatch(&dom, "click", next, ());
    dispatch(&dom, "change", size, "3".to_string());
    assert_eq!(
        data.requests
            .borrow()
            .iter()
            .map(|state| state.pagination.unwrap())
            .collect::<Vec<_>>(),
        [DataGridPagination::new(2, 2), DataGridPagination::new(0, 3)]
    );
    assert_eq!(
        data.state.get().unwrap().peek().pagination,
        Some(DataGridPagination::new(1, 2)),
        "host remains authoritative until accepting a request"
    );
    let cell = *attrs
        .iter()
        .find(|(_, map)| {
            map.get("data-column-id")
                .is_some_and(|value| value == "asset")
                && map.contains_key("aria-readonly")
        })
        .unwrap()
        .0;
    dispatch(&dom, "dblclick", cell, ());
    settle(&mut dom, &mut attrs);
    let html = dioxus_ssr::render(&dom);
    let fieldset = html
        .split('<')
        .find(|tag| tag.starts_with("fieldset "))
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(
        fieldset
            .split_whitespace()
            .any(|part| part == "disabled" || part.starts_with("disabled="))
    );
    assert!(html.contains("Save or cancel the cell edit before changing pages."));
    dispatch(&dom, "click", next, ());
    dispatch(&dom, "change", size, "3".to_string());
    assert_eq!(
        data.requests.borrow().len(),
        2,
        "even synthetic events cannot discard an active edit through pager callbacks"
    );
}
