use crate::{DataGridPageGroup, DataGridPageGroupSelection, DataGridRowModel};
use crate::data_grid_group::validate_page_groups;

fn summary(pairs:&[(&str,&str)],count:usize,selected_count:usize)->DataGridPageGroup{DataGridPageGroup{path:path(pairs),count,selected_count}}
fn host_rows()->Vec<DataGridRow>{vec![
    DataGridRow::new("b").cell("status","Normal").cell("owner","Team 1").cell("asset","Accepted B"),
    DataGridRow::new("c").cell("status","Warning").cell("owner","").cell("asset","Accepted C"),
]}
fn host_columns()->Vec<DataGridColumn>{let mut columns=grouping_columns();let mut asset=DataGridColumn::new("asset","Asset");asset.editable=true;columns.push(asset);columns}
fn summaries()->Vec<DataGridPageGroup>{vec![
    summary(&[("status","Normal")],8,4),summary(&[("status","Normal"),("owner","Team 1")],8,4),
    summary(&[("status","Warning")],10,0),summary(&[("status","Warning"),("owner","")],10,0),
    summary(&[("status","Maintenance")],5,1),
]}
fn host_state()->DataGridState{DataGridState{
    grouping:DataGridGrouping::new(["status","owner"]),pagination:Some(DataGridPagination::new(3,2)),selected:BTreeSet::from(["outside".into(),"b".into()]),
    // These represent the accepted host query; they deliberately contradict
    // local rows so a second client projection would visibly lose/reorder data.
    filters:BTreeMap::from([("status".into(),DataGridTextFilter{first:DataGridFilterCondition{operator:DataGridFilterOperator::Equals,value:"No local match".into()},..Default::default()})]),
    sort:Some(("asset".into(),DataGridSortDirection::Descending)),..Default::default()
}}
#[test]
fn full_model_summaries_include_hidden_offpage_members_and_host_projection_keeps_accepted_rows(){
    let rows=grouping_rows();let columns=grouping_columns();let mut state=DataGridState{grouping:DataGridGrouping::new(["status","owner","status"]),pagination:Some(DataGridPagination::new(1,1)),selected:BTreeSet::from(["a".into(),"outside".into()]),..Default::default()};
    state.collapse_all_groups(&rows,&columns);
    assert_eq!(state.page_group_metadata(&rows,&columns),vec![summary(&[("status","Normal")],2,1),summary(&[("status","Normal"),("owner","Team 1")],2,1),summary(&[("status","Warning")],1,0),summary(&[("status","Warning"),("owner","")],1,0)]);
    let request=DataGridPageGroupSelection{path:path(&[("status","Normal")]),checked:true};state.select_group(&rows,&columns,&request.path,request.checked);
    assert_eq!(state.page_group_metadata(&rows,&columns)[0].selected_count,2);assert!(state.selected.contains("outside"));
    let loaded=host_rows();let mut accepted=host_state();let model=DataGridRowModel::Page{total_rows:23};
    assert_eq!(model.selection_rows(&accepted,&loaded,&host_columns()).iter().map(|row|row.id.as_str()).collect::<Vec<_>>(),["b","c"]);
    accepted.set_group_expanded(&path(&[("status","Normal")]),false);
    assert_eq!(model.projected_rows(&accepted,&loaded,&host_columns()).iter().map(|row|row.id.as_str()).collect::<Vec<_>>(),["c"]);
    assert_eq!(model.selection_rows(&accepted,&loaded,&host_columns()).len(),2,"header selection includes collapsed loaded records");
}
#[test]
fn metadata_validation_rejects_missing_invalid_counts_and_noncontiguous_ancestry(){
    let rows=host_rows();let grouping=DataGridGrouping::new(["status","owner"]);let valid=summaries();assert!(validate_page_groups(&rows,&grouping,&valid).is_ok(),"off-page summaries are allowed");
    let mut duplicate=valid.clone();duplicate.push(valid[0].clone());assert!(validate_page_groups(&rows,&grouping,&duplicate).unwrap_err().contains("duplicate"));
    let mut wrong_prefix=valid.clone();wrong_prefix[0].path=path(&[("owner","Normal")]);assert!(validate_page_groups(&rows,&grouping,&wrong_prefix).unwrap_err().contains("invalid path"));
    let mut overselected=valid.clone();overselected[0].selected_count=9;assert!(validate_page_groups(&rows,&grouping,&overselected).unwrap_err().contains("selection exceeds"));
    let mut missing=valid.clone();missing.remove(1);assert!(validate_page_groups(&rows,&grouping,&missing).unwrap_err().contains("missing a loaded ancestor"));
    let mut too_small=valid.clone();too_small[2].count=0;assert!(validate_page_groups(&rows,&grouping,&too_small).unwrap_err().contains("smaller than"));
    let noncontiguous=vec![rows[0].clone(),rows[1].clone(),rows[0].clone()];assert!(validate_page_groups(&noncontiguous,&grouping,&valid).unwrap_err().contains("contiguous"));
    let nested=vec![rows[0].clone(),DataGridRow::new("d").cell("status","Normal").cell("owner","Team 2"),rows[0].clone()];assert!(validate_page_groups(&nested,&grouping,&valid).unwrap_err().contains("contiguous"));
}

#[derive(Default)]
struct HostCapture{
    state:Cell<Option<Signal<DataGridState>>>,metadata:Cell<Option<Signal<Option<Vec<DataGridPageGroup>>>>>,handler:Cell<Option<Signal<bool>>>,auto:Cell<Option<Signal<bool>>>,rows:Cell<Option<Signal<Vec<DataGridRow>>>>,
    selections:RefCell<Vec<DataGridPageGroupSelection>>,requests:RefCell<Vec<DataGridState>>,editor:RefCell<Option<DataGridEditorContext>>,decline:Cell<bool>,
}
fn host_fixture(data:Rc<HostCapture>)->Element{
    let mut state=use_signal(host_state);let mut rows=use_signal(host_rows);let metadata=use_signal(||Some(summaries()));let handler=use_signal(||true);let auto=use_signal(||false);
    data.state.set(Some(state));data.rows.set(Some(rows));data.metadata.set(Some(metadata));data.handler.set(Some(handler));data.auto.set(Some(auto));
    let requests=data.clone();let selected=data.clone();let editor=data.clone();
    let ongroup=handler().then(||EventHandler::new(move|request|selected.selections.borrow_mut().push(request)));
    rsx!{DataGrid {columns:host_columns(),rows:rows(),state:state(),row_model:DataGridRowModel::Page{total_rows:23},page_groups:metadata(),onpagegroupselect:ongroup,auto_page_size:auto(),height:400,row_height:32,label:"Host groups",tooltip_delay_ms:0,
        onstatechange:move|next:DataGridState|{requests.requests.borrow_mut().push(next.clone());if !requests.decline.get(){state.set(next);}},
        oncellvaluechanged:move|change:DataGridCellValueChanged|rows.with_mut(|rows|{change.apply_to(rows);}),
        render_editor:Callback::new(move|context:DataGridEditorContext|{*editor.editor.borrow_mut()=Some(context);None})
    }}
}
fn host_setup()->(VirtualDom,Rc<HostCapture>,BTreeMap<usize,BTreeMap<String,String>>){let data=Rc::new(HostCapture::default());let mut dom=VirtualDom::new_with_props(host_fixture,data.clone());let mut attrs=BTreeMap::new();collect_render_ids(dom.rebuild_to_vec(),&mut attrs);settle_render(&mut dom,&mut attrs);(dom,data,attrs)}
struct Resize(f64);
impl dioxus::html::HasResizeData for Resize{fn as_any(&self)->&dyn std::any::Any{self}fn get_content_box_size(&self)->dioxus::html::ResizeResult<dioxus::html::geometry::PixelsSize>{Ok(dioxus::html::geometry::PixelsSize::new(640.0,self.0))}}
struct HostConverter;
macro_rules! host_converter{
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{impl dioxus::html::HtmlEventConverter for HostConverter{$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data{host_converter!(@data $group,$convert,event)})*}};
    (@data Resize,$convert:ident,$event:ident)=>{ResizeData::new(Resize(*$event.downcast::<f64>().unwrap()))};
    (@data $other:ident,$convert:ident,$event:ident)=>{RenderConverter.$convert($event)};
}
dioxus::html::with_html_event_groups!(host_converter);
#[test]
fn actual_host_group_counts_selection_collapse_and_local_aria_follow_only_accepted_metadata(){
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(HostConverter));
    let(mut dom,data,mut attrs)=host_setup();let html=dioxus_ssr::render(&dom);let grid=group_tag(&html,"aria-label=\"Host groups\"");assert_eq!(group_attr(grid,"role"),Some("treegrid"));assert_eq!(group_attr(grid,"aria-rowcount"),Some("7"));
    assert_eq!(group_attr(group_tag(&html,"data-row-id=\"b\""),"aria-rowindex"),Some("4"));assert_eq!(group_attr(group_tag(&html,"data-row-id=\"c\""),"aria-rowindex"),Some("7"));assert!(html.contains("7–8 of 23 Items"));assert!(html.contains("8 matching rows"));assert_eq!(group_attr(group_tag(&html,"aria-label=\"Select group Status: Normal\""),"aria-checked"),Some("mixed"));
    let group=render_target(&attrs,"aria-label","Select group Status: Normal");render_dispatch(&dom,"click",group,Box::new(()));settle_render(&mut dom,&mut attrs);
    assert_eq!(&*data.selections.borrow(),&[DataGridPageGroupSelection{path:path(&[("status","Normal")]),checked:true}]);assert_eq!(data.state.get().unwrap().peek().selected,BTreeSet::from(["outside".into(),"b".into()]));assert!(data.requests.borrow().is_empty(),"host group selection cannot silently select loaded members");
    dom.in_runtime(||data.metadata.get().unwrap().with_mut(|metadata|{for group in metadata.as_mut().unwrap(){if group.path.0[0].1=="Normal"{group.selected_count=group.count;}}}));settle_render(&mut dom,&mut attrs);assert_eq!(group_attr(group_tag(&dioxus_ssr::render(&dom),"aria-label=\"Select group Status: Normal\""),"aria-checked"),Some("true"));
    render_dispatch(&dom,"click",render_target(&attrs,"aria-label","Status: Normal"),Box::new(()));settle_render(&mut dom,&mut attrs);let html=dioxus_ssr::render(&dom);assert!(!html.contains("data-row-id=\"b\""));assert!(html.contains("data-row-id=\"c\""));assert_eq!(group_attr(group_tag(&html,"aria-label=\"Host groups\""),"aria-rowcount"),Some("5"));assert_eq!(data.state.get().unwrap().peek().pagination,Some(DataGridPagination::new(3,2)));
    render_dispatch(&dom,"click",render_target(&attrs,"aria-label","Select all loaded rows"),Box::new(()));settle_render(&mut dom,&mut attrs);assert_eq!(data.state.get().unwrap().peek().selected,BTreeSet::from(["outside".into(),"b".into(),"c".into()]));
    dom.in_runtime(||data.handler.get().unwrap().set(false));settle_render(&mut dom,&mut attrs);let html=dioxus_ssr::render(&dom);assert!(group_tag(&html,"aria-label=\"Select group Status: Normal\"").contains("disabled"));let previous=data.selections.borrow().len();render_dispatch(&dom,"click",render_target(&attrs,"aria-label","Select group Status: Normal"),Box::new(()));assert_eq!(data.selections.borrow().len(),previous);
    dom.in_runtime(||data.metadata.get().unwrap().set(None));settle_render(&mut dom,&mut attrs);let html=dioxus_ssr::render(&dom);assert!(html.contains("Grouped host pages require group metadata."));assert!(!html.contains("data-row-id=")&&!html.contains("data-group-path="));
}
#[test]
fn actual_host_group_auto_is_conservative_preserves_position_and_defers_for_loaded_cell_edits(){
    use crate::data_grid_pagination::host_group_page_size;
    assert_eq!(host_group_page_size(8,2),2);assert_eq!(host_group_page_size(1,2),1);assert_eq!(host_group_page_size(usize::MAX,usize::MAX),1);
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(HostConverter));
    let(mut dom,data,mut attrs)=host_setup();data.decline.set(true);dom.in_runtime(||data.auto.get().unwrap().set(true));settle_render(&mut dom,&mut attrs);assert!(data.requests.borrow().is_empty());let grid=render_target(&attrs,"aria-label","Host groups");
    render_dispatch(&dom,"resize",grid,Box::new(48.0+32.0*4.0));settle_render(&mut dom,&mut attrs);assert_eq!(data.requests.borrow().last().unwrap().pagination,Some(DataGridPagination::new(6,1)));assert_eq!(data.state.get().unwrap().peek().pagination,Some(DataGridPagination::new(3,2)));
    data.requests.borrow_mut().clear();
    let cell=render_target(&attrs,"data-column-id","asset");render_dispatch(&dom,"dblclick",cell,Box::new(()));settle_render(&mut dom,&mut attrs);assert!(data.editor.borrow().as_ref().unwrap().active);
    render_dispatch(&dom,"resize",grid,Box::new(48.0+32.0*12.0));settle_render(&mut dom,&mut attrs);assert!(data.requests.borrow().is_empty(),"active editor keeps its accepted page");
    dom.in_runtime(||data.editor.borrow().as_ref().unwrap().cancel());settle_render(&mut dom,&mut attrs);assert_eq!(data.requests.borrow().last().unwrap().pagination,Some(DataGridPagination::new(1,4)));assert!(data.requests.borrow().last().unwrap().selected.contains("outside"));
    dom.in_runtime(||data.auto.get().unwrap().set(false));settle_render(&mut dom,&mut attrs);let cell=render_target(&attrs,"data-column-id","asset");render_dispatch(&dom,"dblclick",cell,Box::new(()));settle_render(&mut dom,&mut attrs);let row=data.editor.borrow().as_ref().unwrap().cell.row.id.clone();dom.in_runtime(||data.editor.borrow().as_ref().unwrap().set_value("Committed host value".into()));settle_render(&mut dom,&mut attrs);dom.in_runtime(||data.editor.borrow().as_ref().unwrap().commit());settle_render(&mut dom,&mut attrs);assert_eq!(data.rows.get().unwrap().peek().iter().find(|item|item.id==row).unwrap().value("asset"),"Committed host value");
    dom.in_runtime(||data.auto.get().unwrap().set(true));settle_render(&mut dom,&mut attrs);data.requests.borrow_mut().clear();
    dom.in_runtime(||data.state.get().unwrap().with_mut(|state|state.set_group_expanded(&path(&[("status","Normal")]),false)));settle_render(&mut dom,&mut attrs);assert!(data.requests.borrow().is_empty(),"collapse does not change the stable host depth budget");
}
