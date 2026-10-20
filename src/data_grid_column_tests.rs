// Actual DataGrid VDOM regressions. Pure geometry is covered in data_grid_columns.
#[derive(Clone, Default)]
struct ColumnConfig { full: bool, reverse: bool, empty: bool }
#[derive(Default)]
struct ColumnObservation {
    config: Cell<Option<Signal<ColumnConfig>>>,
    state: Cell<Option<Signal<DataGridState>>>,
    context: Cell<Option<GridMountContext>>,
    changes: RefCell<Vec<DataGridCellValueChanged>>,
}
#[derive(Clone, Default)]
struct ColumnFixture(Rc<ColumnObservation>);
impl PartialEq for ColumnFixture {
    fn eq(&self,other:&Self)->bool {Rc::ptr_eq(&self.0,&other.0)}
}
#[component]
fn ColumnContextProbe(fixture:ColumnFixture,value:String)->Element {
    fixture.0.context.set(Some(use_context::<GridMountContext>()));
    rsx! {span {"{value}"}}
}
fn column_fixture(fixture:ColumnFixture)->Element {
    let config=use_signal(ColumnConfig::default);
    let mut state=use_signal(DataGridState::default);
    let mut records=use_signal(||(0..2).map(|index| {
        let mut row=DataGridRow::new(format!("row{index}"));
        for column in 0..64 {row=row.cell(format!("c{column}"),format!("{index}-{column}"));}
        row
    }).collect::<Vec<_>>());
    fixture.0.config.set(Some(config));
    fixture.0.state.set(Some(state));
    let current=config();
    let mut columns:Vec<_>=(0..64).map(|index| {
        let mut column=DataGridColumn::new(format!("c{index}"),format!("Field {index}"));
        column.min_width=100;
        column.sortable=true;
        column.filterable=true;
        column.editable=index==0||index==63;
        column
    }).collect();
    if current.reverse {columns.reverse();}
    let changed=fixture.clone();
    rsx! {DataGrid {
        columns,rows:if current.empty {Vec::new()} else {records()},state:state(),onstatechange:move|next|state.set(next),
        suppress_column_virtualisation:current.full,
        oncellvaluechanged:move|event:DataGridCellValueChanged| {
            event.apply_to(&mut records.write());
            changed.0.changes.borrow_mut().push(event);
        },
        render_cell:move|context:DataGridCellContext| {
            if context.column.id=="c0" {Some(DataGridCellRender::display(rsx! {ColumnContextProbe {fixture:fixture.clone(),value:context.value}}))}
            else {None}
        },
    }}
}
#[derive(Clone)]
struct ColumnScroll {left:f64,width:i32}
impl dioxus::html::HasScrollData for ColumnScroll {
    fn as_any(&self)->&dyn std::any::Any {self}
    fn scroll_top(&self)->f64 {0.0}
    fn scroll_left(&self)->f64 {self.left}
    fn scroll_width(&self)->i32 {6448}
    fn scroll_height(&self)->i32 {132}
    fn client_width(&self)->i32 {self.width}
    fn client_height(&self)->i32 {192}
}
#[derive(Clone)]
struct ColumnResize(f64);
impl dioxus::html::HasResizeData for ColumnResize {
    fn as_any(&self)->&dyn std::any::Any {self}
    fn get_content_box_size(&self)->dioxus::html::ResizeResult<dioxus::html::geometry::PixelsSize> {
        Ok(dioxus::html::geometry::PixelsSize::new(self.0,192.0))
    }
}
#[derive(Clone)]
struct ColumnKey {key:Key,modifiers:Modifiers}
impl dioxus::html::ModifiersInteraction for ColumnKey {
    fn modifiers(&self)->Modifiers {self.modifiers}
}
impl dioxus::html::HasKeyboardData for ColumnKey {
    fn key(&self)->Key {self.key.clone()}
    fn code(&self)->dioxus::html::Code {dioxus::html::Code::Unidentified}
    fn location(&self)->dioxus::html::Location {dioxus::html::Location::Standard}
    fn is_auto_repeating(&self)->bool {false}
    fn is_composing(&self)->bool {false}
    fn as_any(&self)->&dyn std::any::Any {self}
}
struct ColumnConverter;
macro_rules! column_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for ColumnConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {
                let _=event;column_converter!(@data $group,event)
            })*
        }
    };
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(RenderMouse)};
    (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(RenderFocus)};
    (@data Keyboard,$event:ident)=>{if let Some(key)=$event.downcast::<ColumnKey>() {dioxus::html::KeyboardData::new(key.clone())}else{dioxus::html::KeyboardData::new($event.downcast::<RenderKey>().unwrap().clone())}};
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(EditorForm($event.downcast::<String>().unwrap().clone()))};
    (@data Scroll,$event:ident)=>{dioxus::html::ScrollData::new($event.downcast::<ColumnScroll>().unwrap().clone())};
    (@data Resize,$event:ident)=>{dioxus::html::ResizeData::new($event.downcast::<ColumnResize>().unwrap().clone())};
    (@data Mounted,$event:ident)=>{MountedData::new(())};
    (@data $other:ident,$event:ident)=>{panic!("Unexpected column test event")};
}
dioxus::html::with_html_event_groups!(column_converter);
fn column_setup(fixture:ColumnFixture)->(VirtualDom,BTreeMap<usize,BTreeMap<String,String>>) {
    let mut dom=VirtualDom::new_with_props(column_fixture,fixture);
    let mut attrs=BTreeMap::new();
    collect_render_ids(dom.rebuild_to_vec(),&mut attrs);
    settle_render(&mut dom,&mut attrs);
    (dom,attrs)
}
fn scrollport(attrs:&BTreeMap<usize,BTreeMap<String,String>>)->dioxus::core::ElementId {
    dioxus::core::ElementId::from_raw(attrs.iter().find_map(|(id,map)|map.get("id").is_some_and(|id|id.ends_with("-scroll")).then_some(*id)).expect("scrollport"))
}
fn scroll_columns(dom:&mut VirtualDom,attrs:&mut BTreeMap<usize,BTreeMap<String,String>>,left:f64,width:i32) {
    render_dispatch(dom,"scroll",scrollport(attrs),Box::new(ColumnScroll {left,width}));
    settle_render(dom,attrs);
}
fn column_tags(html:&str,role:&str)->Vec<String> {
    html.split('<').filter_map(|tag| {
        let tag=tag.split('>').next().unwrap();
        (tag.contains(&format!("role=\"{role}\""))&&tag.contains("data-column-id=")).then(||tag.to_string())
    }).collect()
}
fn has_column(html:&str,id:&str)->bool {html.contains(&format!("data-column-id=\"{id}\""))}
fn logical_attribute(tag:&str,name:&str,value:usize)->bool {
    tag.split_ascii_whitespace().any(|attribute|attribute==format!("{name}={value}")||attribute==format!("{name}=\"{value}\""))
}

#[test]
fn column_window_default_mount_is_bounded_and_optout_keeps_full_logical_tracks() {
    let fixture=ColumnFixture::default();
    let (mut dom,mut attrs)=column_setup(fixture.clone());
    let html=dioxus_ssr::render(&dom);
    let headers=column_tags(&html,"columnheader");
    assert!(headers.len()<16,"premeasurement must not mount all64 columns");
    assert!(!headers.is_empty());
    assert!(html.contains("aria-colcount=65")||html.contains("aria-colcount=\"65\""));
    assert!(html.contains("--ix-grid-min-width:6448px"));
    assert_eq!(column_tags(&html,"gridcell").len(),headers.len()*2);
    assert!(!has_column(&html,"c63"));
    let mut config=fixture.0.config.get().unwrap();
    dom.in_runtime(||config.write().full=true);
    settle_render(&mut dom,&mut attrs);
    let html=dioxus_ssr::render(&dom);
    assert_eq!(column_tags(&html,"columnheader").len(),64);
    assert_eq!(column_tags(&html,"gridcell").len(),128);
    assert!(html.contains("data-column-virtualized=\"false\""));
    assert!(html.contains("--ix-grid-min-width:6448px"));
}

#[test]
fn column_window_scroll_and_resize_keep_original_css_and_accessible_positions() {
    let _guard=crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ColumnConverter));
    let (mut dom,mut attrs)=column_setup(ColumnFixture::default());
    scroll_columns(&mut dom,&mut attrs,3000.0,400);
    let html=dioxus_ssr::render(&dom);
    assert!(has_column(&html,"c0"),"first logical cell remains a keyboard entry");
    assert!(has_column(&html,"c30"));
    assert!(!has_column(&html,"c5"));
    for tag in column_tags(&html,"columnheader").into_iter().chain(column_tags(&html,"gridcell")) {
        let id=tag.split("data-column-id=\"c").nth(1).unwrap().split('"').next().unwrap().parse::<usize>().unwrap();
        assert!(tag.contains(&format!("grid-column:{}",id+2)),"{tag}");
        assert!(logical_attribute(&tag,"aria-colindex",id+2),"{tag}");
    }
    let before=column_tags(&html,"columnheader").len();
    render_dispatch(&dom,"resize",scrollport(&attrs),Box::new(ColumnResize(900.0)));
    settle_render(&mut dom,&mut attrs);
    assert!(column_tags(&dioxus_ssr::render(&dom),"columnheader").len()>before);
}

#[test]
fn column_window_pending_keyboard_target_retains_outgoing_native_focus() {
    let _guard=crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ColumnConverter));
    let fixture=ColumnFixture::default();
    let (mut dom,mut attrs)=column_setup(fixture.clone());
    let first=render_target(&attrs,"data-column-id","c0");
    render_dispatch(&dom,"focus",first,Box::new(()));
    render_dispatch(&dom,"keydown",first,Box::new(RenderKey {key:Key::End,composing:false}));
    settle_render(&mut dom,&mut attrs);
    let context=fixture.0.context.get().unwrap();
    assert_eq!(context.state.peek().pending.as_ref().unwrap().target.as_ref().unwrap().lane,GridLane::Cell("c63".into()));
    assert!(has_column(&dioxus_ssr::render(&dom),"c63"),"offscreen pending cell must mount before physical focus");
    scroll_columns(&mut dom,&mut attrs,5600.0,400);
    assert!(has_column(&dioxus_ssr::render(&dom),"c0"));
    let target=render_target(&attrs,"data-column-id","c63");
    render_dispatch(&dom,"focus",target,Box::new(()));
    settle_render(&mut dom,&mut attrs);
    assert_eq!(context.state.peek().active.as_ref().unwrap().lane,GridLane::Cell("c63".into()));
}

#[test]
fn column_window_editor_draft_and_full_next_editable_model_survive_horizontal_scroll() {
    let _guard=crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ColumnConverter));
    let fixture=ColumnFixture::default();
    let (mut dom,mut attrs)=column_setup(fixture.clone());
    let first=render_target(&attrs,"data-column-id","c0");
    render_dispatch(&dom,"dblclick",first,Box::new(()));
    settle_render(&mut dom,&mut attrs);
    let diagnostic=dioxus_ssr::render(&dom);
    assert!(diagnostic.contains("aria-label=\"Edit Field 0 for row0\""),
        "after doubleclick target={first:?} attrs={:?}; editor tags={:?}",
        attrs.get(&first.raw()),diagnostic.split('<').filter(|tag|tag.starts_with("input ")).map(|tag|tag.split('>').next().unwrap()).collect::<Vec<_>>());
    let editor=render_target(&attrs,"aria-label","Edit Field 0 for row0");
    render_dispatch(&dom,"focus",editor,Box::new(()));
    render_dispatch(&dom,"input",editor,Box::new(String::from("Inspection draft")));
    settle_render(&mut dom,&mut attrs);
    scroll_columns(&mut dom,&mut attrs,5600.0,400);
    assert!(dioxus_ssr::render(&dom).contains("value=\"Inspection draft\""));
    render_dispatch(&dom,"keydown",editor,Box::new(RenderKey {key:Key::Tab,composing:false}));
    settle_render(&mut dom,&mut attrs);
    assert_eq!(fixture.0.changes.borrow().len(),1);
    assert_eq!(fixture.0.changes.borrow()[0].new_value,"Inspection draft");
    assert!(dioxus_ssr::render(&dom).contains("Edit Field 63 for row0"));
    let context=fixture.0.context.get().unwrap();
    assert_eq!(context.state.peek().pending.as_ref().unwrap().target.as_ref().unwrap().lane,GridLane::Editor("c63".into()));
}

#[test]
fn column_window_native_headers_filters_and_reordered_model_remain_owned() {
    let _guard=crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ColumnConverter));
    let fixture=ColumnFixture::default();
    let (mut dom,mut attrs)=column_setup(fixture.clone());
    let heading=render_target(&attrs,"aria-label","Sort Field 3");
    render_dispatch(&dom,"focusin",heading,Box::new(()));
    let filter=render_target(&attrs,"aria-label","Filter Field 4");
    render_dispatch(&dom,"click",filter,Box::new(()));
    scroll_columns(&mut dom,&mut attrs,5600.0,400);
    let html=dioxus_ssr::render(&dom);
    assert!(has_column(&html,"c3"));
    assert!(has_column(&html,"c4"));
    assert!(!has_column(&html,"c5"));
    // A newly activated native filter replaces the one conservative filter slot.
    render_dispatch(&dom,"click",render_target(&attrs,"aria-label","Filter Field 60"),Box::new(()));
    settle_render(&mut dom,&mut attrs);
    assert!(!has_column(&dioxus_ssr::render(&dom),"c4"));
    let mut config=fixture.0.config.get().unwrap();
    dom.in_runtime(||config.write().reverse=true);
    settle_render(&mut dom,&mut attrs);
    let html=dioxus_ssr::render(&dom);
    let first=column_tags(&html,"columnheader").into_iter().find(|tag|tag.contains("data-column-id=\"c63\"")).unwrap();
    assert!(first.contains("grid-column:2"));
    assert!(logical_attribute(&first,"aria-colindex",2));
    let mut state=fixture.0.state.get().unwrap();
    dom.in_runtime(||state.write().selected.insert("row1".into()));
    settle_render(&mut dom,&mut attrs);
    assert!(dioxus_ssr::render(&dom).contains("aria-selected=\"true\""));
}

#[test]
fn column_window_header_keyboard_mounts_far_controls_with_empty_rows_and_preserves_filter_keys() {
    let _guard=crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ColumnConverter));
    let fixture=ColumnFixture::default();
    let (mut dom,mut attrs)=column_setup(fixture.clone());
    let context=fixture.0.context.get().unwrap();
    let mut config=fixture.0.config.get().unwrap();
    dom.in_runtime(||config.write().empty=true);
    settle_render(&mut dom,&mut attrs);
    let heading=render_target(&attrs,"aria-label","Sort Field 0");
    render_dispatch(&dom,"focusin",heading,Box::new(()));
    let result=render_dispatch(&dom,"keydown",heading,Box::new(ColumnKey {key:Key::ArrowRight,modifiers:Modifiers::CONTROL}));
    assert_eq!(result,(false,false));
    settle_render(&mut dom,&mut attrs);
    let html=dioxus_ssr::render(&dom);
    assert!(has_column(&html,"c63"),"offscreen header mounts despite there being no live rows");
    assert!(column_tags(&html,"gridcell").is_empty());
    let last=GridFocus {row:String::new(),lane:GridLane::Header("c63".into())};
    assert_eq!(context.state.peek().pending.as_ref().unwrap().target,Some(last));
    let root=attrs.iter().find_map(|(id,map)|(map.get("data-column-id").map(String::as_str)==Some("c63")&&!map.contains_key("aria-readonly")).then_some(dioxus::core::ElementId::from_raw(*id))).unwrap();
    render_dispatch(&dom,"focusin",root,Box::new(()));
    render_dispatch(&dom,"keydown",root,Box::new(RenderKey {key:Key::ArrowLeft,composing:false}));
    settle_render(&mut dom,&mut attrs);
    assert_eq!(context.state.peek().pending.as_ref().unwrap().target.as_ref().unwrap().lane,GridLane::Header("c62".into()));
    let filter=render_target(&attrs,"aria-label","Field 63 filter value 1");
    render_dispatch(&dom,"focusin",filter,Box::new(()));
    let before=context.state.peek().pending.as_ref().map(|request|request.serial);
    assert_eq!(render_dispatch(&dom,"keydown",filter,Box::new(ColumnKey {key:Key::ArrowLeft,modifiers:Modifiers::CONTROL})),(true,false));
    assert_eq!(context.state.peek().pending.as_ref().map(|request|request.serial),before,"native input word navigation must not request header focus");
}
