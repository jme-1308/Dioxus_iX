use crate::{DataGridCsvOptions,DataGridPagination};

fn grouping_columns()->Vec<DataGridColumn>{
    let mut status=DataGridColumn::new("status","Status");status.editable=true;
    let mut owner=DataGridColumn::new("owner","Owner");owner.editable=true;
    vec![status,owner]
}
fn grouping_rows()->Vec<DataGridRow>{vec![
    DataGridRow::new("a").cell("status","Normal").cell("owner","Team 1"),
    DataGridRow::new("b").cell("status","Normal").cell("owner","Team 1"),
    DataGridRow::new("c").cell("status","Warning").cell("owner",""),
]}
fn path(pairs:&[(&str,&str)])->DataGridGroupPath{DataGridGroupPath::new(pairs.iter().copied())}
#[test]
fn grouped_order_normalizes_columns_and_paths_without_colliding_with_record_ids(){
    let mut count=DataGridColumn::new("count","Count");count.value_type=DataGridValueType::Number;
    let rows=vec![
        DataGridRow::new("").cell("status","Normal").cell("owner","A/B").cell("count","10"),
        DataGridRow::new("group:Normal").cell("status","Normal/A").cell("owner","B").cell("count","2"),
        DataGridRow::new("leaf").cell("status","Normal").cell("count","3"),
        DataGridRow::new("last").cell("status","Normal").cell("owner","A/B").cell("count","4"),
    ];
    let mut state=DataGridState{grouping:DataGridGrouping::new(["status","owner","status"]),sort:Some(("count".into(),DataGridSortDirection::Ascending)),..Default::default()};
    assert_eq!(state.visible_rows_with_columns(&rows,&[count.clone()]).iter().map(|row|row.id.as_str()).collect::<Vec<_>>(),["group:Normal","leaf","last",""]);
    let left=path(&[("status","Normal"),("owner","A/B")]);let right=path(&[("status","Normal/A"),("owner","B")]);
    assert_ne!(left,right);assert_ne!(left.key(),right.key());
    assert!(state.group_paths(&rows,&[count.clone()]).contains(&path(&[("status","Normal"),("owner","")])));
    state.select_group(&rows,&[count.clone()],&DataGridGroupPath::default(),true);
    state.select_group(&rows,&[count.clone()],&path(&[("owner","A/B")]),true);
    state.select_group(&rows,&[count],&path(&[("status","Normal"),("stale","A/B")]),true);
    assert!(state.selected.is_empty(),"empty/non-prefix/stale column paths never select unrelated records");
    assert!(!state.can_reorder());
}
#[test]
fn collapsed_page_headers_keep_full_descendants_csv_and_correct_global_slot_positions(){
    let rows=grouping_rows();let columns=grouping_columns();let root=path(&[("status","Normal")]);
    let mut state=DataGridState{grouping:DataGridGrouping::new(["status","owner"]),pagination:Some(DataGridPagination::new(1,1)),selected:BTreeSet::from(["outside".into(),"a".into()]),..Default::default()};
    assert_eq!(state.group_selection(&rows,&columns,&root),(false,true));
    let full:Vec<_>=state.visible_rows_with_columns(&rows,&columns).into_iter().cloned().collect();
    let (slots,total)=group_slots(&full,&state.grouping,state.pagination_info(full.len()).range);
    assert_eq!(total,7);assert_eq!(slots.len(),3);
    assert!(matches!(&slots[0],GroupSlot::Group{count:2,aria_index:2,..}));
    assert!(matches!(&slots[2],GroupSlot::Record{row:1,aria_index:5}));
    state.set_group_expanded(&root,false);
    assert_eq!(state.expanded_page_rows_with_columns(&rows,&columns).len(),0);
    assert_eq!(next_editable_cell(&rows,&columns,&state,"b","status",false),None);
    let (slots,total)=group_slots(&full,&state.grouping,state.pagination_info(full.len()).range);
    assert_eq!(total,4);assert_eq!(slots.len(),1);
    state.select_group(&rows,&columns,&root,true);
    assert_eq!(state.selected,BTreeSet::from(["outside".into(),"a".into(),"b".into()]));
    assert_eq!(state.export_csv(&rows,&columns,&DataGridCsvOptions::default()).row_ids,["a","b","c"]);
    assert_eq!(state.pagination_info(full.len()).page_count,3);
    state.expand_all_groups();assert_eq!(state.expanded_page_rows_with_columns(&rows,&columns)[0].id,"b");
    state.collapse_all_groups(&rows,&columns);assert_eq!(state.grouping.collapsed.len(),4);
}
#[derive(Default)]
struct GroupObservation {
    state:Cell<Option<Signal<DataGridState>>>,rows:Cell<Option<Signal<Vec<DataGridRow>>>>,
    auto:Cell<Option<Signal<bool>>>,model:Cell<Option<Signal<crate::DataGridRowModel>>>,
    editor:RefCell<Option<DataGridEditorContext>>,context:Cell<Option<GridMountContext>>,
}
impl PartialEq for GroupObservation{fn eq(&self,other:&Self)->bool{std::ptr::eq(self,other)}}
#[component]
fn GroupProbe(data:Rc<GroupObservation>,value:String)->Element{data.context.set(Some(use_context::<GridMountContext>()));rsx!{span {"{value}"}}}
fn grouped_fixture(data:Rc<GroupObservation>)->Element{
    let mut state=use_signal(||DataGridState{grouping:DataGridGrouping::new(["status","owner"]),pagination:Some(DataGridPagination::new(1,1)),selected:BTreeSet::from(["outside".into(),"a".into()]),..Default::default()});
    let mut rows=use_signal(grouping_rows);let auto=use_signal(||false);let model=use_signal(||crate::DataGridRowModel::Client);
    data.state.set(Some(state));data.rows.set(Some(rows));data.auto.set(Some(auto));data.model.set(Some(model));
    let cell=data.clone();let editor=data.clone();
    rsx!{DataGrid {columns:grouping_columns(),rows:rows(),state:state(),row_model:model(),auto_page_size:auto(),label:"Grouped records",row_height:32,tooltip_delay_ms:0,
        onstatechange:move|next|state.set(next),oncellvaluechanged:move|change:DataGridCellValueChanged|rows.with_mut(|rows|{change.apply_to(rows);}),
        render_cell:Callback::new(move|context:DataGridCellContext|Some(DataGridCellRender::display(rsx!{GroupProbe {data:cell.clone(),value:context.value}}))),
        render_editor:Callback::new(move|context:DataGridEditorContext|{*editor.editor.borrow_mut()=Some(context);None})
    }}
}
fn group_setup()->(VirtualDom,Rc<GroupObservation>,BTreeMap<usize,BTreeMap<String,String>>){let data=Rc::new(GroupObservation::default());let mut dom=VirtualDom::new_with_props(grouped_fixture,data.clone());let mut attrs=BTreeMap::new();collect_render_ids(dom.rebuild_to_vec(),&mut attrs);settle_render(&mut dom,&mut attrs);(dom,data,attrs)}
fn group_tag<'a>(html:&'a str,marker:&str)->&'a str{html.split('<').find(|tag|tag.contains(marker)).unwrap().split('>').next().unwrap()}
fn group_attr<'a>(tag:&'a str,name:&str)->Option<&'a str>{let value=tag.split_once(&format!(" {name}="))?.1;if let Some(value)=value.strip_prefix('"'){Some(value.split('"').next().unwrap())}else{value.split_whitespace().next()}}
#[test]
fn actual_group_rows_select_collapsed_members_and_navigate_typed_group_lanes(){
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(RenderConverter));
    let(mut dom,data,mut attrs)=group_setup();let root=path(&[("status","Normal")]);let child=path(&[("status","Normal"),("owner","Team 1")]);
    let html=dioxus_ssr::render(&dom);
    assert_eq!(group_attr(group_tag(&html,"aria-label=\"Grouped records\""),"role"),Some("treegrid"));
    assert_eq!(group_attr(group_tag(&html,"aria-label=\"Grouped records\""),"aria-rowcount"),Some("8"));
    assert_eq!(group_attr(group_tag(&html,"data-row-id=\"b\""),"aria-rowindex"),Some("5"));
    assert!(group_tag(&html,"data-row-id=\"b\"").contains("top:64px"));
    assert_eq!(group_attr(group_tag(&html,"data-row-id=\"b\""),"aria-level"),Some("3"));
    assert_eq!(group_attr(group_tag(&html,"aria-label=\"Select group Status: Normal\""),"aria-checked"),Some("mixed"));
    let disclosure=render_target(&attrs,"aria-label","Status: Normal");let group_root=render_target(&attrs,"data-group-path",&root.key());let select=render_target(&attrs,"aria-label","Select group Status: Normal");
    render_dispatch(&dom,"focus",disclosure,Box::new(()));
    render_dispatch(&dom,"keydown",group_root,Box::new(RenderKey{key:Key::ArrowDown,composing:false}));settle_render(&mut dom,&mut attrs);
    let context=data.context.get().unwrap();
    assert_eq!(context.state.peek().pending.as_ref().unwrap().target.as_ref().unwrap().lane,GridLane::Group(child));
    render_dispatch(&dom,"click",disclosure,Box::new(()));settle_render(&mut dom,&mut attrs);
    assert!(!dioxus_ssr::render(&dom).contains("data-row-id="));
    assert!(context.state.peek().pending.is_none(),"collapse retires a now-hidden nested group target");
    render_dispatch(&dom,"click",select,Box::new(()));settle_render(&mut dom,&mut attrs);
    assert_eq!(data.state.get().unwrap().peek().selected,BTreeSet::from(["outside".into(),"a".into(),"b".into()]));
    render_dispatch(&dom,"keydown",group_root,Box::new(RenderKey{key:Key::ArrowRight,composing:false}));settle_render(&mut dom,&mut attrs);
    assert!(dioxus_ssr::render(&dom).contains("data-row-id=\"b\""));
    assert_eq!(data.state.get().unwrap().peek().pagination,Some(DataGridPagination::new(1,1)));
}
#[test]
fn actual_group_edits_regroup_by_identity_and_unsupported_modes_never_render_fallback_rows(){
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(RenderConverter));
    let(mut dom,data,mut attrs)=group_setup();
    dom.in_runtime(||data.state.get().unwrap().with_mut(|state|state.pagination=None));settle_render(&mut dom,&mut attrs);
    let cell=render_target(&attrs,"data-column-id","status");render_dispatch(&dom,"focus",cell,Box::new(()));render_dispatch(&dom,"keydown",cell,Box::new(RenderKey{key:Key::F2,composing:false}));settle_render(&mut dom,&mut attrs);
    let row=data.editor.borrow().as_ref().unwrap().cell.row.id.clone();
    dom.in_runtime(||data.editor.borrow().as_ref().unwrap().set_value("Warning".into()));settle_render(&mut dom,&mut attrs);
    dom.in_runtime(||data.editor.borrow().as_ref().unwrap().commit());settle_render(&mut dom,&mut attrs);
    assert_eq!(data.rows.get().unwrap().peek().iter().find(|item|item.id==row).unwrap().value("status"),"Warning");
    assert_eq!(data.state.get().unwrap().peek().group_rows(&data.rows.get().unwrap().peek(),&grouping_columns(),&path(&[("status","Warning")])).len(),2);
    dom.in_runtime(||data.model.get().unwrap().set(crate::DataGridRowModel::Page{total_rows:50}));settle_render(&mut dom,&mut attrs);
    let html=dioxus_ssr::render(&dom);assert!(html.contains("Grouping requires client rows."));assert!(!html.contains("data-row-id=")&&!html.contains("data-group-path="));
    dom.in_runtime(||{data.model.get().unwrap().set(crate::DataGridRowModel::Client);data.auto.get().unwrap().set(true);});settle_render(&mut dom,&mut attrs);
    let html=dioxus_ssr::render(&dom);assert!(html.contains("Automatic page sizing is unavailable while grouping."));assert!(!html.contains("data-row-id="));
    dom.in_runtime(||{data.auto.get().unwrap().set(false);data.state.get().unwrap().with_mut(|state|state.grouping=DataGridGrouping::default());});settle_render(&mut dom,&mut attrs);
    let html=dioxus_ssr::render(&dom);assert!(!html.contains("data-group-path="));assert_eq!(group_attr(group_tag(&html,"aria-label=\"Grouped records\""),"role"),Some("grid"));assert!(html.contains("data-row-id="));
}
