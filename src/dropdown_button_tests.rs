// Reuses Dropdown's existing converter, mutation collection and local-state fixture.
fn dropdown_button_fixture(config: Rc<RefCell<Fixture>>) -> Element {
    let settings = config.borrow().clone();
    config.borrow_mut().update = Some(dioxus::core::schedule_update());
    let before = config.clone();
    let changed = config.clone();
    let button_label = settings
        .button_content
        .map(|text| rsx! {span{class:"task-action-label","{text}"}});
    rsx! {crate::DropdownButton{label:settings.button_text,button_label,icon:settings.button_icon,aria_label:"Task actions",
        placement:settings.placement,disabled:settings.blocked,loading:settings.loading,enable_top_layer:settings.layer,
        onbeforechange:move|event:DropdownChangeEvent|{
            let veto={let mut state=before.borrow_mut();state.requests.push((event.open,event.previous));state.veto_open&&event.open};
            if veto{event.prevent_default();}
        },onchange:move|open|changed.borrow_mut().changed.push(open),
        Stateful{config:config.clone()}DropdownItem{label:"Apply task"}
    }}
}
fn dropdown_button_harness() -> Harness {
    dioxus::html::set_event_converter(Box::new(Converter));
    let config = Rc::new(RefCell::new(Fixture::default()));
    let mut dom = VirtualDom::new_with_props(dropdown_button_fixture, config.clone());
    let changes = dom.rebuild_to_vec();
    let mut h = Harness {
        dom,
        config,
        attrs: BTreeMap::new(),
        listeners: BTreeMap::new(),
    };
    h.collect(changes);
    for id in h.listeners.get("mounted").cloned().unwrap_or_default() {
        dispatch(&h.dom, "mounted", id, ());
    }
    h.settle();
    h
}
fn dropdown_button_presentation(html: &str) -> &str {
    html.split("<button ")
        .skip(1)
        .find(|part| {
            part.split('>')
                .next()
                .unwrap()
                .contains("ix-dropdown-trigger ")
        })
        .unwrap()
        .split("</button>")
        .next()
        .unwrap()
}
fn dropdown_button_key(h: &mut Harness, key: Key) -> (bool, bool) {
    let target = h.target("Task actions");
    dispatch(&h.dom, "focus", target, ());
    h.key("Task actions", key)
}
#[test]
fn dropdown_button_preserves_legacy_trigger_and_names_omitted_icon_presentation() {
    let _guard = crate::test_event_converter_guard();
    fn legacy() -> Element {
        rsx! {crate::DropdownButton{label:"Legacy action",DropdownItem{label:"Apply"}}}
    }
    fn generic() -> Element {
        rsx! {Dropdown{label:"Legacy action",variant:crate::ButtonVariant::Primary,DropdownItem{label:"Apply"}}}
    }
    fn omitted() -> Element {
        rsx! {crate::DropdownButton{DropdownItem{label:"Apply"}}}
    }
    let mut facade = VirtualDom::new(legacy);
    facade.rebuild_in_place();
    let mut normal = VirtualDom::new(generic);
    normal.rebuild_in_place();
    let old_html = dioxus_ssr::render(&facade);
    let normal_html = dioxus_ssr::render(&normal);
    assert_eq!(
        dropdown_button_presentation(&old_html),
        dropdown_button_presentation(&normal_html)
    );
    assert!(!old_html.contains("ix-dropdown-trigger-icon-only"));
    let mut icon = VirtualDom::new(omitted);
    icon.rebuild_in_place();
    let html = dioxus_ssr::render(&icon);
    assert!(html.contains("aria-label=\"Open dropdown\""));
    assert!(html.contains("aria-label=\"Menu\""));
    let presentation = dropdown_button_presentation(&html);
    assert!(presentation.contains("ix-dropdown-trigger-icon-only"));
    let tag = presentation.split('>').next().unwrap();
    assert!(
        tag.split_ascii_whitespace()
            .any(|attr| attr == "inert" || attr.starts_with("inert=")),
        "{tag}"
    );
    assert!(
        tag.split_ascii_whitespace().any(|attr| attr
            .strip_prefix("tabindex=")
            .is_some_and(|value| value.trim_matches('"') == "-1")),
        "{tag}"
    );
    assert_eq!(
        presentation.matches("class=\"ix-icon ").count(),
        1,
        "fallback icon remains useful without supplied icon"
    );
    assert_eq!(
        html.matches("role=\"button\"").count(),
        1,
        "owning role-button remains the interactive control"
    );
}
#[test]
fn dropdown_button_live_custom_icon_and_placement_keep_menu_state_and_owner() {
    let _guard = crate::test_event_converter_guard();
    let mut h = dropdown_button_harness();
    let context = h.context();
    let trigger = h.target("Task actions");
    let count = *h.listeners["click"]
        .iter()
        .find(|id| {
            h.attrs
                .get(&id.raw())
                .is_some_and(|attrs| attrs.contains_key("data-count-button"))
        })
        .unwrap();
    h.click("Task actions");
    dispatch(&h.dom, "click", count, ());
    h.settle();
    assert!(h.html().contains("Local 1"));
    assert!(!*context.open.peek());
    {
        let mut state = h.config.borrow_mut();
        state.button_text = String::new();
        state.button_icon = Some("cut".into());
        state.placement = DropdownPlacement::TopEnd;
    }
    h.update();
    assert_eq!(h.target("Task actions"), trigger);
    let html = h.html();
    let presentation = dropdown_button_presentation(&html);
    assert!(presentation.contains("ix-dropdown-trigger-icon-only"));
    assert_eq!(presentation.matches("class=\"ix-icon ").count(), 1);
    assert!(html.contains("data-configured-placement=\"top-end\""));
    h.click("Task actions");
    assert!(*context.open.peek());
    {
        let mut state = h.config.borrow_mut();
        state.button_text = "Semantic task actions".into();
        state.button_content = Some("Ready 3".into());
        state.placement = DropdownPlacement::BottomEnd;
    }
    h.update();
    let html = h.html();
    let presentation = dropdown_button_presentation(&html);
    assert!(!presentation.contains("ix-dropdown-trigger-icon-only"));
    assert!(presentation.contains("Ready 3"));
    assert!(
        !presentation.contains("Semantic task actions"),
        "custom content overrides visible text only"
    );
    assert!(html.contains("aria-label=\"Task actions\""));
    assert!(html.contains("aria-label=\"Semantic task actions\""));
    assert!(html.contains("data-configured-placement=\"bottom-end\""));
    assert!(html.contains("Local 1"));
    assert_eq!((h.config.borrow().mounts, h.config.borrow().drops), (1, 0));
    assert_eq!(h.target("Task actions"), trigger);
    assert!(*context.open.peek());
    dropdown_button_key(&mut h, Key::Escape);
    assert!(!*context.open.peek());
}
#[test]
fn dropdown_button_icon_mode_keeps_loading_disabled_and_cancelable_open_guards() {
    let _guard = crate::test_event_converter_guard();
    let mut h = dropdown_button_harness();
    {
        let mut state = h.config.borrow_mut();
        state.button_text = String::new();
        state.button_icon = Some("cut".into());
        state.veto_open = true;
    }
    h.update();
    h.click("Task actions");
    assert!(!*h.context().open.peek());
    assert_eq!(h.config.borrow().requests.len(), 1);
    h.config.borrow_mut().veto_open = false;
    h.update();
    h.click("Task actions");
    assert!(*h.context().open.peek());
    dropdown_button_key(&mut h, Key::Escape);
    assert!(!*h.context().open.peek());
    {
        let mut state = h.config.borrow_mut();
        state.loading = true;
        state.requests.clear();
    }
    h.update();
    assert!(dropdown_button_presentation(&h.html()).contains("ix-spinner"));
    h.click("Task actions");
    assert!(!*h.context().open.peek());
    assert!(h.config.borrow().requests.is_empty());
    {
        let mut state = h.config.borrow_mut();
        state.loading = false;
        state.blocked = true;
    }
    h.update();
    let html = h.html();
    assert!(dropdown_button_presentation(&html).contains("disabled"));
    assert!(html.contains("aria-disabled=\"true\""));
    h.click("Task actions");
    assert!(h.config.borrow().requests.is_empty());
    h.config.borrow_mut().blocked = false;
    h.update();
    dropdown_button_key(&mut h, Key::ArrowDown);
    assert!(*h.context().open.peek());
}
