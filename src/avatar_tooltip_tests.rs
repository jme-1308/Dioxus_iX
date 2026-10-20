// Uses Avatar's existing actual-VDOM fixture and event converter.
#[test]
fn avatar_styled_tooltip_tracks_text_labels_and_removal_without_a_native_title() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().tooltip_label = Some("Profile information".into());
    let (mut dom, mut targets) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    let panel = tag(&html, "role", "tooltip");
    let panel_id = panel
        .split("id=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let face = tag(&html, "id", "avatar-test");
    assert!(face.contains(&format!("aria-describedby=\"{panel_id}\"")));
    assert!(!face.contains("title=") && !face.contains("tabindex="));
    assert!(panel.contains("aria-label=\"Profile information\""));
    let anchor = targets.attr("id", "avatar-test");
    dispatch(&dom, "mouseenter", anchor, Box::new(()));
    settle(&mut dom, &mut targets);
    assert!(tag(&dioxus_ssr::render(&dom), "role", "tooltip").contains("data-open=\"true\""));
    assert!(dispatch(
        &dom,
        "keydown",
        anchor,
        Box::new(RawKey(Key::Escape))
    ));
    settle(&mut dom, &mut targets);
    assert!(tag(&dioxus_ssr::render(&dom), "role", "tooltip").contains("data-open=\"false\""));
    {
        let mut config = data.config.borrow_mut();
        config.tooltip = None;
        config.username = Some("Operator Two".into());
        config.tooltip_label = Some("Profilinformationen".into());
    }
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Operator Two"));
    assert!(!html.contains("Open operator details"));
    assert!(tag(&html, "role", "tooltip").contains("aria-label=\"Profilinformationen\""));
    data.config.borrow_mut().tooltip_label = None;
    refresh(&mut dom, &mut targets);
    assert!(!tag(&dioxus_ssr::render(&dom), "role", "tooltip").contains("aria-label="));
    // An explicit empty value suppresses the username fallback and clears ARIA.
    data.config.borrow_mut().tooltip = Some(String::new());
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("role=\"tooltip\""));
    assert!(!tag(&html, "id", "avatar-test").contains("aria-describedby="));
    assert!(html.contains("src=\"portrait-one.png\""));
}

#[test]
fn avatar_menu_open_suppresses_tooltip_and_descendant_reentry_preserving_draft() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().children = true;
    let (mut dom, mut targets) = setup(data.clone());
    let button = targets.attr("aria-label", "Operator One");
    dispatch(&dom, "focusin", button, Box::new(()));
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(trigger(&html).contains("aria-describedby="));
    assert!(tag(&html, "role", "tooltip").contains("data-open=\"true\""));
    assert_eq!(html.matches("role=\"tooltip\"").count(), 1);
    data.veto.set(true);
    dispatch(&dom, "click", button, Box::new(()));
    settle(&mut dom, &mut targets);
    assert!(!open(&dom));
    assert!(dioxus_ssr::render(&dom).contains("role=\"tooltip\""));
    data.veto.set(false);
    dispatch(&dom, "click", button, Box::new(()));
    settle(&mut dom, &mut targets);
    assert!(open(&dom));
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("role=\"tooltip\""));
    assert!(!trigger(&html).contains("aria-describedby="));
    assert!(!tag(&html, "id", "avatar-test").contains("title="));
    let draft = targets.attr("id", "avatar-draft");
    dispatch(
        &dom,
        "input",
        draft,
        Box::new(String::from("Prepared shift")),
    );
    dispatch(&dom, "focusin", draft, Box::new(()));
    dispatch(&dom, "mouseenter", draft, Box::new(()));
    settle(&mut dom, &mut targets);
    assert!(!dioxus_ssr::render(&dom).contains("role=\"tooltip\""));
    dispatch(&dom, "keydown", button, Box::new(RawKey(Key::Escape)));
    settle(&mut dom, &mut targets);
    assert!(!open(&dom));
    dispatch(&dom, "focusin", button, Box::new(()));
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(tag(&html, "role", "tooltip").contains("data-open=\"true\""));
    assert!(tag(&html, "id", "avatar-draft").contains("value=\"Prepared shift\""));
    assert_eq!(data.mounts.get(), 1);
    assert!(dispatch(
        &dom,
        "keydown",
        button,
        Box::new(RawKey(Key::Escape))
    ));
    settle(&mut dom, &mut targets);
    assert!(tag(&dioxus_ssr::render(&dom), "role", "tooltip").contains("data-open=\"false\""));
}
