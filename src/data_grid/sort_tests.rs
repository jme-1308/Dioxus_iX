use crate::{DataGridCsvOptions, DataGridPagination};

#[test]
fn priority_normalization_and_additive_cycles_preserve_other_controlled_state() {
    use DataGridSortDirection::{Ascending as Asc, Descending as Desc};
    let mut state = DataGridState {
        sort: Some(("group".into(), Asc)),
        secondary_sorts: vec![("group".into(), Desc), ("count".into(), Asc), ("count".into(), Desc), ("unknown".into(), Desc)],
        pagination: Some(DataGridPagination::new(3, 2)),
        selected: BTreeSet::from(["outside".into()]),
        filters: BTreeMap::from([("group".into(), DataGridTextFilter {first:DataGridFilterCondition {operator:DataGridFilterOperator::Contains,value:"Line".into()},..Default::default()})]),
        column_order: vec!["count".into(),"group".into()],
        row_order: vec!["b".into(), "a".into()],
        column_widths: BTreeMap::from([("group".into(), 220)]),
        ..Default::default()
    };
    assert_eq!(state.sort_chain(), [("group", Asc), ("count", Asc), ("unknown", Desc)]);
    assert_eq!(state.sort_for_column("count"), Some((2, Asc)));
    state.cycle_sort_with_append("count", true);
    assert_eq!(state.sort_chain(), [("group", Asc), ("count", Desc), ("unknown", Desc)]);
    state.cycle_sort_with_append("count", true);
    assert_eq!(state.sort_for_column("unknown"), Some((2, Desc)));
    state.cycle_sort_with_append("group", true);
    state.cycle_sort_with_append("group", true);
    assert_eq!(state.sort_chain(), [("unknown", Desc)], "primary removal promotes its next priority");
    state.cycle_sort_with_append("count", true);
    state.cycle_sort("count");
    assert_eq!(state.sort_chain(), [("count", Asc)], "ordinary activation makes a secondary the sole ascending primary");
    state.cycle_sort("count");
    assert_eq!(state.sort, Some(("count".into(), Desc)));
    state.cycle_sort("count");
    assert!(state.sort_chain().is_empty() && state.secondary_sorts.is_empty());
    state.secondary_sorts.push(("stale".into(), Asc));
    assert!(state.sort_chain().is_empty(), "an absent primary ignores the tail");
    state.clear_sorts();
    assert!(state.secondary_sorts.is_empty());
    assert_eq!(state.pagination, Some(DataGridPagination::new(0, 2)));
    assert_eq!(state.selected, BTreeSet::from(["outside".into()]));
    assert_eq!(state.row_order, ["b", "a"]);
    assert_eq!(state.column_widths["group"], 220);
    assert_eq!(state.filters["group"].first.value,"Line");
    assert_eq!(state.column_order,["count","group"]);
}
fn sort_columns() -> Vec<DataGridColumn> {
    let mut group = DataGridColumn::new("group", "Group"); group.sortable = true;
    let mut count = DataGridColumn::new("count", "Count"); count.sortable = true; count.value_type = DataGridValueType::Number; count.editable = true;
    vec![group, count]
}
fn sort_rows() -> Vec<DataGridRow> {
    [("a","Line","10","x"),("b","line","2","10"),("c","Line","2.0","2"),("d","Alpha","9","x"),("e","Line","NaN","x"),("f","Line","","x"),("g","Line","2","10")]
        .into_iter().map(|(id,group,count,tag)|DataGridRow::new(id).cell("group",group).cell("count",count).cell("unknown",tag)).collect()
}
#[test]
fn typed_multi_key_projection_drives_pages_csv_edit_navigation_and_stable_complete_ties() {
    use DataGridSortDirection::Ascending as Asc;
    let rows = sort_rows(); let columns = sort_columns();
    let state = DataGridState {
        sort: Some(("group".into(), Asc)), secondary_sorts: vec![("count".into(), Asc),("unknown".into(), Asc)],
        row_order: vec!["g".into(),"b".into(),"c".into()], pagination: Some(DataGridPagination::new(1,2)), ..Default::default()
    };
    let expected = ["d","g","b","c","a","e","f"];
    assert_eq!(state.visible_rows_with_columns(&rows,&columns).iter().map(|row|row.id.as_str()).collect::<Vec<_>>(),expected);
    assert_eq!(state.export_csv(&rows,&columns,&DataGridCsvOptions::default()).row_ids,expected);
    assert_eq!(state.paged_rows_with_columns(&rows,&columns).iter().map(|row|row.id.as_str()).collect::<Vec<_>>(),["b","c"]);
    assert_eq!(next_editable_cell(&rows,&columns,&state,"b","count",false),Some(("c".into(),"count".into())));
    assert_eq!(state.visible_rows(&rows).iter().map(|row|row.id.as_str()).collect::<Vec<_>>(),["d","f","a","g","b","c","e"],"legacy untyped projection retains text ordering");
    assert_eq!(crate::DataGridRowModel::Page{total_rows:20}.projected_rows(&state,&rows,&columns).iter().map(|row|row.id.as_str()).collect::<Vec<_>>(),["a","b"],"host pages remain authoritative supplied order");
}
#[test]
fn header_fit_request_includes_visible_sort_priority_and_rejects_changed_priority() {
    use DataGridSortDirection::Ascending as Asc;
    let mut inputs = GridAutosizeInputs {columns:sort_columns(),rows:Rc::new(sort_rows()),state:DataGridState {sort:Some(("group".into(),Asc)),..Default::default()}};
    let single = inputs.request("group",1).unwrap(); assert_eq!(single.sort_priority,None);
    inputs.state.cycle_sort_with_append("count",true);
    assert!(!inputs.current(&single),"a newly visible priority changes header fit content");
    let secondary = inputs.request("count",2).unwrap(); assert_eq!(secondary.sort_priority,Some(2)); assert_eq!(secondary.sort,Some(Asc));
    let mut dom = VirtualDom::new_with_props(|request|rsx!{GridAutosizeProbe {request,onmeasure:move |_|{}}},secondary.clone());
    dom.rebuild_in_place(); let html=dioxus_ssr::render(&dom);
    assert!(html.contains("ix-data-grid-sort-priority"));
    assert!(html.contains(">2</span>"),"fit probe renders the same numeric priority as its real header");
    inputs.state.cycle_sort_with_append("group",true);inputs.state.cycle_sort_with_append("group",true);
    assert!(!inputs.current(&secondary),"promotion removes the now-sole priority badge");
}

#[derive(Default)]
struct SortObservation {state:Cell<Option<Signal<DataGridState>>>,requests:RefCell<Vec<DataGridState>>,accept:Cell<bool>}
fn sort_fixture(data:Rc<SortObservation>)->Element {
    let mut state=use_signal(||DataGridState {pagination:Some(DataGridPagination::new(1,2)),selected:BTreeSet::from(["outside".into()]),..Default::default()});
    data.state.set(Some(state));
    rsx!{DataGrid {columns:sort_columns(),rows:sort_rows(),state:state(),onstatechange:move|next:DataGridState|{if data.accept.get(){state.set(next.clone());}data.requests.borrow_mut().push(next);}}}
}
struct ShiftMouse(bool);
impl dioxus::html::InteractionLocation for ShiftMouse {
    fn client_coordinates(&self)->dioxus::html::geometry::ClientPoint{dioxus::html::InteractionLocation::client_coordinates(&RenderMouse)}
    fn page_coordinates(&self)->dioxus::html::geometry::PagePoint{dioxus::html::InteractionLocation::page_coordinates(&RenderMouse)}
    fn screen_coordinates(&self)->dioxus::html::geometry::ScreenPoint{dioxus::html::InteractionLocation::screen_coordinates(&RenderMouse)}
}
impl dioxus::html::InteractionElementOffset for ShiftMouse {fn element_coordinates(&self)->dioxus::html::geometry::ElementPoint{dioxus::html::InteractionElementOffset::element_coordinates(&RenderMouse)}}
impl dioxus::html::ModifiersInteraction for ShiftMouse {fn modifiers(&self)->Modifiers{if self.0 {Modifiers::SHIFT}else{Modifiers::empty()}}}
impl dioxus::html::PointerInteraction for ShiftMouse {
    fn trigger_button(&self)->Option<dioxus::html::input_data::MouseButton>{dioxus::html::PointerInteraction::trigger_button(&RenderMouse)}
    fn held_buttons(&self)->dioxus::html::input_data::MouseButtonSet{dioxus::html::PointerInteraction::held_buttons(&RenderMouse)}
}
impl dioxus::html::HasMouseData for ShiftMouse {fn as_any(&self)->&dyn std::any::Any{self}}
struct SortConverter;
macro_rules! sort_converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for SortConverter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data{sort_converter!(@data $group,$convert,event)})*}
    };
    (@data Mouse,$convert:ident,$event:ident)=>{dioxus::html::MouseData::new(ShiftMouse($event.downcast::<bool>().copied().unwrap_or(false)))};
    (@data $other:ident,$convert:ident,$event:ident)=>{<RenderConverter as dioxus::html::HtmlEventConverter>::$convert(&RenderConverter,$event)};
}
dioxus::html::with_html_event_groups!(sort_converter);
fn heading<'a>(html:&'a str,column:&str)->&'a str{html.split('<').find(|tag|tag.starts_with("div ")&&tag.contains("class=\"ix-data-grid-heading\"")&&tag.contains(&format!("data-column-id=\"{column}\""))).unwrap().split('>').next().unwrap()}
fn sort_attr<'a>(tag:&'a str,name:&str)->Option<&'a str>{let value=tag.split_once(&format!(" {name}="))?.1;if let Some(value)=value.strip_prefix('"'){Some(value.split('"').next().unwrap())}else{value.split_whitespace().next()}}
#[test]
fn actual_header_shift_requests_wait_for_owner_and_expose_current_priority_metadata() {
    use DataGridSortDirection::Ascending as Asc;
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(SortConverter));
    let data=Rc::new(SortObservation::default());let mut dom=VirtualDom::new_with_props(sort_fixture,data.clone());let mut attrs=BTreeMap::new();
    collect_render_ids(dom.rebuild_to_vec(),&mut attrs);settle_render(&mut dom,&mut attrs);
    // Header owners and their keys remain stable throughout these sort requests.
    let group=render_target(&attrs,"aria-label","Sort Group");let count=render_target(&attrs,"aria-label","Sort Count");
    render_dispatch(&dom,"click",group,Box::new(false));settle_render(&mut dom,&mut attrs);
    assert!(data.state.get().unwrap().peek().sort.is_none(),"sorting is controlled until the owner accepts");
    assert_eq!(data.requests.borrow().last().unwrap().sort,Some(("group".into(),Asc)));
    data.accept.set(true);dom.in_runtime(||data.state.get().unwrap().set(data.requests.borrow().last().unwrap().clone()));settle_render(&mut dom,&mut attrs);
    render_dispatch(&dom,"click",count,Box::new(true));settle_render(&mut dom,&mut attrs);
    let html=dioxus_ssr::render(&dom);
    assert_eq!(sort_attr(heading(&html,"group"),"data-sort-priority"),Some("1"));
    assert_eq!(sort_attr(heading(&html,"count"),"data-sort-priority"),Some("2"));
    assert_eq!(sort_attr(heading(&html,"group"),"aria-sort"),Some("ascending"));
    assert_eq!(sort_attr(heading(&html,"count"),"aria-sort"),Some("none"));
    assert_eq!(html.matches("class=\"ix-data-grid-sort-priority\"").count(),2);
    assert!(html.contains("Sort priority 2, ascending.")&&html.contains("Shift+click"));
    assert_eq!(data.state.get().unwrap().peek().pagination,Some(DataGridPagination::new(0,2)));
    assert!(data.state.get().unwrap().peek().selected.contains("outside"));
    render_dispatch(&dom,"click",count,Box::new(true));settle_render(&mut dom,&mut attrs);
    assert_eq!(data.state.get().unwrap().peek().sort_for_column("count"),Some((2,DataGridSortDirection::Descending)));
    render_dispatch(&dom,"click",count,Box::new(true));settle_render(&mut dom,&mut attrs);
    assert!(!dioxus_ssr::render(&dom).contains("class=\"ix-data-grid-sort-priority\""));
    render_dispatch(&dom,"click",count,Box::new(true));settle_render(&mut dom,&mut attrs);
    for _ in 0..2{render_dispatch(&dom,"click",group,Box::new(true));settle_render(&mut dom,&mut attrs);}
    assert_eq!(data.state.get().unwrap().peek().sort_chain(),[("count",Asc)]);
    render_dispatch(&dom,"click",group,Box::new(false));settle_render(&mut dom,&mut attrs);
    assert_eq!(data.state.get().unwrap().peek().sort_chain(),[("group",Asc)]);
    assert!(!dioxus_ssr::render(&dom).contains("class=\"ix-data-grid-sort-priority\""));
}
