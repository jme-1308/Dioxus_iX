// Reuses the scheduling event converter, mutation collector and native focus mock.
#[derive(Default)]
struct TemporalMountData {
    context:RefCell<Option<TemporalRangeContext>>,
    nodes:RefCell<BTreeMap<usize,Rc<MountedData>>>,
    deliveries:RefCell<Vec<usize>>,
    owner:RefCell<Option<String>>,
    revision:Cell<bool>,visible:Cell<bool>,
}
fn record_temporal_mount(data:&TemporalMountData,index:usize,event:MountedEvent){
    let context=data.context.borrow().clone().unwrap();
    let targets=context.targets.borrow();
    let internal=targets.values().nth(index).unwrap().input.borrow().clone();
    assert!(internal.as_ref().is_some_and(|node|Rc::ptr_eq(node,&event.data())),"internal range handle must be registered before consumer callback");
    data.deliveries.borrow_mut().push(index);data.nodes.borrow_mut().insert(index,event.data());
}
fn temporal_mount_fixture(data:Rc<TemporalMountData>)->Element {
    provide_range_navigation();
    *data.context.borrow_mut()=Some(consume_context::<TemporalRangeContext>());
    let date_owner=data.clone();let time_owner=data.clone();let combined_owner=data.clone();let form=data.owner.borrow().clone();
    rsx!{
        if data.visible.get(){
            DateInput{today:date(2026,10,20),id:"mounted-date",form:form.clone(),value:if data.revision.get(){"2026/10/21"}else{"2026/10/20"},onvaluechange:|_|{},onmounted:move|event|record_temporal_mount(&date_owner,0,event)}
            TimeInput{id:"mounted-time",form:form.clone(),value:"12:30:00",onvaluechange:|_|{},onmounted:move|event|record_temporal_mount(&time_owner,1,event)}
            DateTimeInput{today:date(2026,10,20),id:"mounted-combined",form,value:"2026/10/20 12:30:00",onvaluechange:|_|{},onmounted:move|event|record_temporal_mount(&combined_owner,2,event)}
        }
    }
}
struct TemporalMountHarness {
    dom:VirtualDom,targets:ScheduleTargets,data:Rc<TemporalMountData>,focuses:[Rc<Cell<usize>>;3],
}
impl TemporalMountHarness {
    fn new()->Self{
        let data=Rc::new(TemporalMountData::default());data.visible.set(true);*data.owner.borrow_mut()=Some("primary".into());
        let mut dom=VirtualDom::new_with_props(temporal_mount_fixture,data.clone());let changes=dom.rebuild_to_vec();
        let mut h=Self{dom,targets:ScheduleTargets::default(),data,focuses:std::array::from_fn(|_|Rc::new(Cell::new(0)))};h.mount_changes(changes);h
    }
    fn mount_changes(&mut self,changes:dioxus::core::Mutations){
        let mut current=dioxus::core::ElementId::ROOT;let mut pending=Vec::new();
        for edit in &changes.edits{match edit{
            dioxus::core::Mutation::SetId{id}|dioxus::core::Mutation::PushId{id}=>current = *id,
            dioxus::core::Mutation::NewEventListener{name} if name=="mounted"=>pending.push(current),_=>{}
        }}
        self.targets.collect(changes);
        for target in pending{
            let id=self.targets.attrs.get(&target.raw()).and_then(|attrs|attrs.get("id"));
            if let Some(index)=["mounted-date","mounted-time","mounted-combined"].iter().position(|expected|id.is_some_and(|id|id.as_str()==*expected)){
                schedule_dispatch(&self.dom,"mounted",target,Box::new(ScheduleMount(self.focuses[index].clone())));
            } else if id.is_some_and(|id| id.ends_with("-toggle")) {
                self.targets.mount_toggle(&self.dom,target);
            }
        }
        schedule_render(&mut self.dom,&mut self.targets);
    }
    fn refresh(&mut self){self.dom.mark_dirty(ScopeId::APP);let changes=self.dom.render_immediate_to_vec();self.mount_changes(changes);}
    fn focus_current(&mut self){
        let nodes=self.data.nodes.borrow().values().cloned().collect::<Vec<_>>();
        self.dom.in_scope(ScopeId::ROOT,||{for node in nodes{spawn(async move{node.set_focus(true).await.unwrap();});}});
        schedule_render(&mut self.dom,&mut self.targets);
    }
}
#[test]
fn temporal_mount_callbacks_receive_each_native_handle_after_internal_registration(){
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let mut h=TemporalMountHarness::new();let mut delivered=h.data.deliveries.borrow().clone();delivered.sort_unstable();assert_eq!(delivered,[0,1,2]);
    assert_eq!(h.data.nodes.borrow().len(),3);h.focus_current();
    assert_eq!(h.focuses.each_ref().map(|calls|calls.get()),[1,1,1],"consumer handles focus the three respective native inputs");
    let html=dioxus_ssr::render(&h.dom);
    for id in ["mounted-date","mounted-time","mounted-combined"]{assert!(!temporal_surface_open(&html,id),"input focus does not open a picker");}
}
#[test]
fn temporal_mount_owner_and_value_rerenders_keep_handles_until_actual_remount(){
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let mut h=TemporalMountHarness::new();let original=h.data.nodes.borrow().clone();
    *h.data.owner.borrow_mut()=Some("alternate".into());h.data.revision.set(true);h.refresh();
    assert_eq!(h.data.deliveries.borrow().len(),3,"ordinary prop updates do not redeliver mounted events");
    for(index,node)in &original{assert!(Rc::ptr_eq(node,&h.data.nodes.borrow()[index]));}
    h.focus_current();assert_eq!(h.focuses.each_ref().map(|calls|calls.get()),[1,1,1]);
    h.data.visible.set(false);h.refresh();assert!(h.data.context.borrow().as_ref().unwrap().targets.borrow().is_empty());
    h.data.visible.set(true);h.refresh();assert_eq!(h.data.deliveries.borrow().len(),6);
    for(index,node)in original{assert!(!Rc::ptr_eq(&node,&h.data.nodes.borrow()[&index]),"replacement inputs deliver replacement native handles");}
    h.focus_current();assert_eq!(h.focuses.each_ref().map(|calls|calls.get()),[2,2,2]);
}
#[test]
fn temporal_mount_consumer_callbacks_preserve_pointer_range_handoff(){
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let mut h=TemporalMountHarness::new();let calendar=h.targets.label("Open calendar");
    schedule_dispatch(&h.dom,"click",calendar,Box::new(()));schedule_render(&mut h.dom,&mut h.targets);
    let selection=h.targets.label("21 October 2026");schedule_dispatch(&h.dom,"click",selection,Box::new(()));schedule_render(&mut h.dom,&mut h.targets);
    let html=dioxus_ssr::render(&h.dom);
    assert!(!temporal_surface_open(&html,"mounted-date"));assert!(temporal_surface_open(&html,"mounted-time"),"range handoff still opens the next input picker");
    assert_eq!(h.focuses.each_ref().map(|calls|calls.get()),[0,1,0],"existing handoff uses the internally registered input handle");
    assert_eq!(h.data.deliveries.borrow().len(),3);
}
