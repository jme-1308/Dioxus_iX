// Reuses the scheduling converter/targets. Actual FormData and native validity
// are verified in the external temporal form browser/native workflow.
#[derive(Default)]
struct TemporalFormData {owner:RefCell<Option<String>>,mounts:Cell<usize>}
impl PartialEq for TemporalFormData {fn eq(&self,other:&Self)->bool{std::ptr::eq(self,other)}}
#[component]
fn TemporalPreparation(data:Rc<TemporalFormData>)->Element {
    use_hook(||data.mounts.set(data.mounts.get()+1));
    let mut ready=use_signal(||false);
    rsx!{button{id:String::from("temporal-preparation"),r#type:"button",aria_pressed:ready(),onclick:move|_|ready.toggle(),if ready(){"Tools prepared"}else{"Prepare tools"}}}
}
fn temporal_form_fixture(data:Rc<TemporalFormData>)->Element {
    let form=data.owner.borrow().clone();
    rsx!{
        form{id:"temporal-ancestor",DateInput{today:date(2026,10,20),name:"ancestor-date",value:"2026/10/20",onvaluechange:|_|{}}}
        form{id:"temporal-primary"}form{id:"temporal-alternate"}
        DateInput{today:date(2026,10,20),id:"temporal-form-date",name:"service-date",form:form.clone(),value:"2026/10/20",required:true,onvaluechange:|_|{}}
        TimeInput{id:"temporal-form-time",name:"service-time",form:form.clone(),value:"12:30:00",required:true,onvaluechange:|_|{}}
        DateTimeInput{today:date(2026,10,20),id:"temporal-form-combined",name:"appointment",form,value:"2026/10/20 12:30:00",required:true,onvaluechange:|_|{}}
        TemporalPreparation{data}
    }
}
fn temporal_form_attribute<'a>(tag:&'a str,name:&str)->Option<&'a str>{
    let value=tag.split_once(&format!(" {name}="))?.1;
    if let Some(value)=value.strip_prefix('"'){Some(value.split('"').next().unwrap())}
    else{Some(value.split(|ch:char|ch.is_whitespace()||ch=='>').next().unwrap())}
}
fn temporal_form_controls(html:&str)->Vec<&str>{
    html.split('<').filter_map(|part|part.split_once('>').map(|(tag,_)|tag)).filter(|tag|tag.starts_with("input ")&&temporal_form_attribute(tag,"name").is_some_and(|name|name!="ancestor-date")).collect()
}
fn temporal_form_setup(owner:Option<String>)->(VirtualDom,ScheduleTargets,Rc<TemporalFormData>){
    let data=Rc::new(TemporalFormData::default());*data.owner.borrow_mut()=owner;
    let mut dom=VirtualDom::new_with_props(temporal_form_fixture,data.clone());let mut targets=ScheduleTargets::default();targets.collect(dom.rebuild_to_vec());schedule_render(&mut dom,&mut targets);(dom,targets,data)
}
#[test]
fn temporal_form_omission_preserves_native_ancestor_and_default_input_markup(){
    let(dom,_,_)=temporal_form_setup(None);let html=dioxus_ssr::render(&dom);let fields=temporal_form_controls(&html);assert_eq!(fields.len(),3);
    for field in fields{assert!(temporal_form_attribute(field,"form").is_none());assert_eq!(temporal_form_attribute(field,"type"),Some("text"));}
    let ancestor=html.split('<').find(|part|part.starts_with("input ")&&temporal_form_attribute(part,"name")==Some("ancestor-date")).unwrap();
    assert!(temporal_form_attribute(ancestor,"form").is_none());assert!(html.contains("id=\"temporal-ancestor\""));
}
#[test]
fn temporal_form_three_public_paths_associate_only_the_named_successful_input(){
    let(dom,_,_)=temporal_form_setup(Some("temporal-primary".into()));let html=dioxus_ssr::render(&dom);let fields=temporal_form_controls(&html);assert_eq!(fields.len(),3);
    let expected=[("service-date","2026/10/20"),("service-time","12:30:00"),("appointment","2026/10/20 12:30:00")];
    for(name,value)in expected{
        let field=fields.iter().find(|field|temporal_form_attribute(field,"name")==Some(name)).unwrap();
        assert_eq!(temporal_form_attribute(field,"form"),Some("temporal-primary"));assert_eq!(temporal_form_attribute(field,"value"),Some(value));assert!(field.contains("required"));assert!(temporal_form_attribute(field,"pattern").is_none(),"valid temporal value retains native validation");
    }
    assert_eq!(html.matches("form=\"temporal-primary\"").count(),3,"no duplicate mirrors or associated picker buttons");
}
#[test]
fn temporal_form_live_owner_changes_keep_values_open_picker_and_preparation_scope(){
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let(mut dom,mut targets,data)=temporal_form_setup(Some("temporal-primary".into()));
    let prep=dioxus::core::ElementId::from_raw(*targets.attrs.iter().find(|(_,attrs)|attrs.get("id").is_some_and(|id|id=="temporal-preparation")).unwrap().0);
    schedule_dispatch(&dom,"click",prep,Box::new(()));schedule_render(&mut dom,&mut targets);
    // The first calendar toggle belongs to the ordinary ancestor DateInput;
    // select the external field's actual native button within its field wrapper.
    let html=dioxus_ssr::render(&dom);let from=html.find("id=\"temporal-form-date\"").unwrap();let button=html[from..].split("</button>").next().unwrap();
    let toggle=targets.attrs.iter().find_map(|(id,attrs)|{
        let label=attrs.get("aria-label")?;
        (label=="Open calendar"&&attrs.get("aria-controls").is_some_and(|value|value=="temporal-form-date-picker")).then_some(dioxus::core::ElementId::from_raw(*id))
    }).unwrap_or_else(||panic!("external calendar toggle missing: {button}"));
    schedule_dispatch(&dom,"click",toggle,Box::new(()));schedule_render(&mut dom,&mut targets);
    assert!(dioxus_ssr::render(&dom).contains("id=\"temporal-form-date-picker\""));
    for owner in[Some("temporal-alternate".to_string()),None]{
        *data.owner.borrow_mut()=owner.clone();dom.mark_dirty(ScopeId::APP);schedule_render(&mut dom,&mut targets);let html=dioxus_ssr::render(&dom);let fields=temporal_form_controls(&html);
        assert_eq!(fields.len(),3);for field in fields{assert_eq!(temporal_form_attribute(field,"form"),owner.as_deref());}
        for value in["2026/10/20","12:30:00","2026/10/20 12:30:00"]{assert!(html.contains(&format!("value=\"{value}\"")));}
        for id in["temporal-form-date","temporal-form-time","temporal-form-combined"]{assert!(html.contains(&format!("id=\"{id}\"")));}
        assert!(html.contains("id=\"temporal-form-date-picker\""),"owner updates retain the open picker");assert!(html.contains("Tools prepared"));assert_eq!(data.mounts.get(),1);
    }
}
