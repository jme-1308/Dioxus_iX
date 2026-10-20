// Actual FormData/constraint behavior is verified by the browser/native workflow;
// these focused VDOM tests verify successful-control ownership and retention.
#[derive(Default)]
struct ExternalFormData {
    owner: std::cell::RefCell<Option<String>>,
    mounts: std::cell::Cell<usize>,
}
impl PartialEq for ExternalFormData {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn ExternalPreparation(data: Rc<ExternalFormData>) -> Element {
    use_hook(|| data.mounts.set(data.mounts.get() + 1));
    let mut note = use_signal(|| String::from("Prepare tools"));
    rsx! {input{id:String::from("external-preparation"),value:note(),oninput:move|event|note.set(event.value())}}
}
fn external_form_fixture(data: Rc<ExternalFormData>) -> Element {
    let form = data.owner.borrow().clone();
    rsx! {
        form{id:"ancestor-form",Input{name:"ancestor",value:"Ancestor value"}}
        form{id:"external-a"}form{id:"external-b"}
        Input{id:"external-asset",name:"asset",form:form.clone(),value:"Motor 001",required:true}
        Textarea{id:"external-notes",name:"notes",form:form.clone(),value:"Inspect bearings"}
        NumberInput{id:"external-crew",name:"crew",form:form.clone(),value:"4",min:1.,max:6.}
        Checkbox{id:"external-check",name:"isolation",form:form.clone(),value:"isolated",label:"Isolate",checked:true}
        Radio{id:"external-mode",name:"mode",form:form.clone(),value:"repair",label:"Repair",checked:true}
        RadioGroup{name:"priority",form:form.clone(),value:"normal",options:vec![SelectOption::new("normal","Normal"),SelectOption::new("urgent","Urgent")]}
        Toggle{id:"external-remote",name:"remote",form:form.clone(),value:"enabled",checked:true}
        Slider{name:"progress",form:form.clone(),value:40.}
        Select{id:"external-zone",name:"zone",form:form.clone(),value:"line1",options:vec![SelectOption::new("line1","Line 1"),SelectOption::new("line2","Line 2")]}
        ExternalPreparation{data}
    }
}
fn external_form_openings(html: &str) -> Vec<&str> {
    html.split('<')
        .filter_map(|part| part.split_once('>').map(|(tag, _)| tag))
        .collect()
}
fn external_form_controls(html: &str) -> Vec<&str> {
    external_form_openings(html)
        .into_iter()
        .filter(|tag| {
            (tag.starts_with("input ")
                || tag.starts_with("textarea ")
                || tag.starts_with("select "))
                && feedback_attribute(tag, "name").is_some_and(|name| name != "ancestor")
        })
        .collect()
}
fn external_form_render(owner: Option<String>) -> String {
    let data = Rc::new(ExternalFormData::default());
    *data.owner.borrow_mut() = owner;
    let mut dom = VirtualDom::new_with_props(external_form_fixture, data);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    dioxus_ssr::render(&dom)
}
#[test]
fn external_form_omission_preserves_native_ancestor_markup_and_all_default_targets() {
    let html = external_form_render(None);
    let controls = external_form_controls(&html);
    assert_eq!(
        controls.len(),
        10,
        "nine public APIs include two generated group radios"
    );
    for control in controls {
        assert!(
            feedback_attribute(control, "form").is_none(),
            "omission must not override native ancestor ownership: {control}"
        );
    }
    let ancestor = external_form_openings(&html)
        .into_iter()
        .find(|tag| feedback_attribute(tag, "name") == Some("ancestor"))
        .unwrap();
    assert!(feedback_attribute(ancestor, "form").is_none());
    assert!(html.contains("id=\"ancestor-form\""));
}
#[test]
fn external_form_association_targets_successful_controls_and_forwards_composed_owners() {
    let html = external_form_render(Some("external-a".into()));
    let controls = external_form_controls(&html);
    assert_eq!(controls.len(), 10);
    let mut names = std::collections::BTreeMap::new();
    for control in controls {
        assert_eq!(
            feedback_attribute(control, "form"),
            Some("external-a"),
            "{control}"
        );
        *names
            .entry(feedback_attribute(control, "name").unwrap())
            .or_insert(0) += 1;
    }
    assert_eq!(
        names.keys().copied().collect::<Vec<_>>(),
        [
            "asset",
            "crew",
            "isolation",
            "mode",
            "notes",
            "priority",
            "progress",
            "remote",
            "zone"
        ]
    );
    assert_eq!(
        names["priority"], 2,
        "RadioGroup forwards form to both native radios"
    );
    let number = feedback_tag(&html, "external-crew");
    assert_eq!(feedback_attribute(number, "type"), Some("number"));
    assert_eq!(feedback_attribute(number, "form"), Some("external-a"));
    let mirror = external_form_openings(&html)
        .into_iter()
        .find(|tag| feedback_attribute(tag, "name") == Some("zone"))
        .unwrap();
    assert!(mirror.starts_with("select "));
    let visible = feedback_tag(&html, "external-zone");
    assert!(feedback_attribute(visible, "name").is_none());
    assert_eq!(
        feedback_attribute(visible, "form"),
        Some("external-a"),
        "nameless combobox shares ownership for SelectController without submitting a duplicate value"
    );
    for tag in external_form_openings(&html)
        .into_iter()
        .filter(|tag| tag.starts_with("button "))
    {
        assert!(
            feedback_attribute(tag, "form").is_none(),
            "presentation controls do not own submitted values"
        );
    }
}
#[test]
fn external_form_live_retarget_preserves_control_values_ids_and_independent_preparation() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(PaneFormsConverter));
    let data = Rc::new(ExternalFormData::default());
    *data.owner.borrow_mut() = Some("external-a".into());
    let mut dom = VirtualDom::new_with_props(external_form_fixture, data.clone());
    let changes = dom.rebuild_to_vec();
    let mut current = dioxus::core::ElementId::ROOT;
    let mut preparation = None;
    for edit in changes.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" && value == "external-preparation" => preparation = Some(current),
            _ => {}
        }
    }
    feedback_event(
        &dom,
        "input",
        preparation.unwrap(),
        String::from("Tools prepared"),
    );
    feedback_refresh(&mut dom);
    let initial = dioxus_ssr::render(&dom);
    let radio_ids = external_form_controls(&initial)
        .into_iter()
        .filter(|tag| feedback_attribute(tag, "name") == Some("priority"))
        .map(|tag| feedback_attribute(tag, "id").unwrap().to_string())
        .collect::<Vec<_>>();
    for owner in [Some("external-b".to_string()), None] {
        *data.owner.borrow_mut() = owner.clone();
        feedback_refresh(&mut dom);
        let html = dioxus_ssr::render(&dom);
        for control in external_form_controls(&html) {
            assert_eq!(
                feedback_attribute(control, "form"),
                owner.as_deref(),
                "{control}"
            );
        }
        assert_eq!(
            feedback_attribute(feedback_tag(&html, "external-zone"), "form"),
            owner.as_deref(),
            "SelectController input follows live form ownership"
        );
        assert_eq!(
            feedback_attribute(feedback_tag(&html, "external-asset"), "value"),
            Some("Motor 001")
        );
        assert_eq!(
            feedback_attribute(feedback_tag(&html, "external-crew"), "value"),
            Some("4")
        );
        assert!(feedback_tag(&html, "external-check").contains("aria-checked=\"true\""));
        assert!(feedback_tag(&html, "external-remote").contains("aria-checked=\"true\""));
        assert!(feedback_tag(&html, "external-preparation").contains("value=\"Tools prepared\""));
        let ids = external_form_controls(&html)
            .into_iter()
            .filter(|tag| feedback_attribute(tag, "name") == Some("priority"))
            .map(|tag| feedback_attribute(tag, "id").unwrap().to_string())
            .collect::<Vec<_>>();
        assert_eq!(ids, radio_ids);
        assert_eq!(data.mounts.get(), 1);
    }
}
