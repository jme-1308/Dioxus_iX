// The existing forms event converter supplies ordinary mouse, keyboard and form
// events; native range defaults and FormData are covered by the gallery workflow.
fn render_slider(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn slider_endpoint_content_preserves_legacy_absence_and_explicit_empty_precedence() {
    let absent = render_slider(|| rsx! { Slider {} });
    assert!(!absent.contains("ix-slider-endpoints"));
    let legacy = render_slider(|| rsx! { Slider {label_start:"Minimum",label_end:"Maximum"} });
    assert!(legacy.contains("<span>Minimum</span>"));
    assert!(legacy.contains("<span>Maximum</span>"));
    let composed = render_slider(|| rsx! {
        Slider {label_start:"Hidden start",label_end:"Hidden end",
            label_start_content:rsx!{strong{"Composed minimum"}},
            label_end_content:rsx!{button{r#type:"button","Set maximum"}}
        }
    });
    assert!(composed.contains("ix-slider-endpoints"));
    assert!(composed.contains("<strong>Composed minimum</strong>"));
    assert!(composed.contains("Set maximum"));
    assert!(!composed.contains("Hidden start"));
    assert!(!composed.contains("Hidden end"));
    let empty = render_slider(|| rsx! {
        Slider {label_start:"No fallback",label_start_content:VNode::empty()}
    });
    assert!(empty.contains("ix-slider-endpoints"));
    assert!(!empty.contains("No fallback"));
    let end_only = render_slider(|| rsx! {
        Slider {label_end_content:rsx!{span{"End only"}}}
    });
    assert!(end_only.contains("ix-slider-endpoints"));
    assert!(end_only.contains("End only"));
}

#[derive(Default)]
struct SliderComposition {
    value: std::cell::Cell<f64>,
    disabled: std::cell::Cell<bool>,
    mounts: std::cell::Cell<usize>,
    requests: std::cell::RefCell<Vec<f64>>,
    actions: std::cell::Cell<usize>,
}
impl PartialEq for SliderComposition {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn EndpointPreparation(data: Rc<SliderComposition>, disabled: bool) -> Element {
    use_hook(|| data.mounts.set(data.mounts.get() + 1));
    let mut note = use_signal(|| String::from("Prepare tools"));
    rsx! {
        input {id:String::from("slider-endpoint-note"),name:"note",value:note(),disabled,
            oninput:move|event|if !disabled {note.set(event.value());}
        }
    }
}
fn slider_composition_fixture(data: Rc<SliderComposition>) -> Element {
    let minimum = data.clone();
    let maximum = data.clone();
    let requests = data.clone();
    let disabled = data.disabled.get();
    rsx! {
        form {id:"slider-work-order",
            Slider {label:"Inspection rate",name:"rate",value:data.value.get(),disabled,
                label_start:"Hidden minimum",label_end:"Hidden maximum",
                onvaluechange:move|value|{
                    requests.value.set(value);
                    requests.requests.borrow_mut().push(value);
                },
                label_start_content:rsx!{
                    button {id:String::from("slider-set-minimum"),r#type:"button",disabled,
                        onclick:move|_|if !disabled {
                            minimum.value.set(0.);
                            minimum.actions.set(minimum.actions.get()+1);
                        },
                        "Set minimum"
                    }
                },
                label_end_content:rsx!{
                    EndpointPreparation {data:data.clone(),disabled}
                    button {id:String::from("slider-set-maximum"),r#type:"button",disabled,
                        onclick:move|_|if !disabled {
                            maximum.value.set(100.);
                            maximum.actions.set(maximum.actions.get()+1);
                        },
                        "Set maximum"
                    }
                }
            }
        }
    }
}
fn composition_dom() -> (
    VirtualDom,
    Rc<SliderComposition>,
    std::collections::BTreeMap<String, dioxus::core::ElementId>,
) {
    let data = Rc::new(SliderComposition::default());
    data.value.set(30.);
    let mut dom = VirtualDom::new_with_props(slider_composition_fixture, data.clone());
    let changes = dom.rebuild_to_vec();
    let mut ids = std::collections::BTreeMap::new();
    let mut current = dioxus::core::ElementId::ROOT;
    for edit in changes.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => current = id,
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" => {ids.insert(value, current);}
            _ => {}
        }
    }
    (dom, data, ids)
}
fn composition_range(html: &str) -> &str {
    html.split('<')
        .filter_map(|part|part.split_once('>').map(|(tag,_)|tag))
        .find(|tag|tag.starts_with("input ") && feedback_attribute(tag,"type")==Some("range"))
        .unwrap()
}
fn assert_composed_note(dom: &VirtualDom, data: &SliderComposition) {
    let html = dioxus_ssr::render(dom);
    assert_eq!(feedback_attribute(feedback_tag(&html,"slider-endpoint-note"),"value"),Some("Tools ready"));
    assert_eq!(data.mounts.get(),1,"ordinary controlled updates retain the endpoint component");
}

#[test]
fn slider_composed_controls_keep_local_input_and_controlled_range_requests_independent() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(PaneFormsConverter));
    let (mut dom,data,ids) = composition_dom();
    let initial = dioxus_ssr::render(&dom);
    let range_id = feedback_attribute(composition_range(&initial),"id").unwrap().to_string();
    assert_eq!(feedback_attribute(composition_range(&initial),"name"),Some("rate"));
    for id in ["slider-set-minimum","slider-set-maximum"] {
        assert_eq!(feedback_attribute(feedback_tag(&initial,id),"type"),Some("button"));
    }
    feedback_event(&dom,"input",ids["slider-endpoint-note"],String::from("Tools ready"));
    feedback_refresh(&mut dom);
    assert_composed_note(&dom,&data);
    assert!(feedback_event(&dom,"click",ids["slider-set-minimum"],()));
    feedback_refresh(&mut dom);
    assert_eq!(data.value.get(),0.);
    assert_eq!(feedback_attribute(composition_range(&dioxus_ssr::render(&dom)),"value"),Some("0"));
    assert_composed_note(&dom,&data);
    assert!(feedback_event(&dom,"input",ids[&range_id],String::from("42")));
    feedback_refresh(&mut dom);
    assert_eq!(*data.requests.borrow(),[42.]);
    assert_eq!(data.value.get(),42.);
    assert!(feedback_event(&dom,"keydown",ids[&range_id],Key::ArrowRight),"native range keyboard default remains available");
    assert!(feedback_event(&dom,"click",ids["slider-set-maximum"],()));
    feedback_refresh(&mut dom);
    assert_eq!(data.value.get(),100.);
    assert_eq!(data.actions.get(),2);
    assert_eq!(*data.requests.borrow(),[42.],"endpoint actions are consumer events, not synthetic range requests");
    assert_composed_note(&dom,&data);
}

#[test]
fn slider_endpoint_consumers_guard_disabled_actions_and_resume_without_remounting() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(PaneFormsConverter));
    let (mut dom,data,ids) = composition_dom();
    let initial = dioxus_ssr::render(&dom);
    let range_id = feedback_attribute(composition_range(&initial),"id").unwrap().to_string();
    feedback_event(&dom,"input",ids["slider-endpoint-note"],String::from("Tools ready"));
    feedback_refresh(&mut dom);
    data.disabled.set(true);
    feedback_refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(composition_range(&html).contains(" disabled"));
    assert!(feedback_tag(&html,"slider-set-minimum").contains(" disabled"));
    assert!(feedback_tag(&html,"slider-endpoint-note").contains(" disabled"));
    // Synthetic dispatch deliberately reaches disabled handlers: both the native
    // slider and the consumer's composed controls must keep their own guard.
    feedback_event(&dom,"click",ids["slider-set-minimum"],());
    feedback_event(&dom,"input",ids[&range_id],String::from("80"));
    feedback_event(&dom,"input",ids["slider-endpoint-note"],String::from("Blocked"));
    feedback_refresh(&mut dom);
    assert_eq!(data.value.get(),30.);
    assert_eq!(data.actions.get(),0);
    assert!(data.requests.borrow().is_empty());
    assert_composed_note(&dom,&data);
    data.disabled.set(false);
    feedback_refresh(&mut dom);
    feedback_event(&dom,"click",ids["slider-set-maximum"],());
    feedback_refresh(&mut dom);
    assert_eq!(data.value.get(),100.);
    assert_eq!(data.actions.get(),1);
    assert_composed_note(&dom,&data);
}
