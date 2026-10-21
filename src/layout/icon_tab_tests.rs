fn tab_button<'a>(html: &'a str, key: &str) -> &'a str {
    let marker = format!("-tab-{}\"", crate::composition::tab_key_id(key));
    let at = html.find(&marker).expect("native tab ID");
    let start = html[..at].rfind("<button ").unwrap();
    let end = at + html[at..].find("</button>").unwrap() + "</button>".len();
    &html[start..end]
}
fn opening_attr<'a>(button: &'a str, name: &str) -> Option<&'a str> {
    let tag = button.split('>').next().unwrap();
    let value = tag.split_once(&format!(" {name}="))?.1;
    Some(if let Some(value) = value.strip_prefix('"') {
        value.split('"').next().unwrap()
    } else {
        value.split_whitespace().next().unwrap()
    })
}
#[test]
fn icon_only_tabs_keep_meaningful_names_counters_and_guarded_presentation_modes() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(WorkspaceConverter));
    let fixture = WorkspaceFixture::default();
    let (mut dom, mut targets) = workspace_overflow_setup(fixture.clone());
    let original = targets.tab("a");
    let html = dioxus_ssr::render(&dom);
    assert!(tab_button(&html, "a").contains(">Inspection<"));
    assert_eq!(opening_attr(tab_button(&html, "a"), "aria-label"), None);
    let mut config = fixture.0.config.get().unwrap();
    dom.in_runtime(|| {
        let mut config = config.write();
        config.items[0].icon = Some("home".into());
        config.items[0].icon_only = true;
        config.items[0].counter = Some(3);
        config.items[1].icon = Some("home".into());
        config.items[1].icon_only = true;
        config.items[2].icon = Some("  ".into());
        config.items[2].icon_only = true;
        config.items.push(TabItem {
            icon_only: true,
            ..TabItem::new("d", "History")
        });
    });
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(targets.tab("a"), original);
    let html = dioxus_ssr::render(&dom);
    let icon = tab_button(&html, "a");
    assert_eq!(opening_attr(icon, "aria-label"), Some("Inspection"));
    assert_eq!(opening_attr(icon, "title"), Some("Inspection"));
    assert_eq!(opening_attr(icon, "data-icon-only"), Some("true"));
    assert!(!icon.contains(">Inspection<"));
    assert!(
        icon.contains("aria-hidden=\"true\""),
        "the icon remains decorative"
    );
    assert!(icon.contains("ix-tab-counter") && icon.contains(">3<"));
    assert!(html.contains("aria-label=\"Close Inspection\""));
    assert!(
        html.contains("data-ix-dropdown-label=\"Inspection\""),
        "the all-tabs entry retains its complete name"
    );
    assert_eq!(
        opening_attr(tab_button(&html, "b"), "aria-label"),
        Some("Reference")
    );
    assert!(
        tab_button(&html, "b")
            .split('>')
            .next()
            .unwrap()
            .contains("disabled")
    );
    for (key, label) in [("c", "Checklist"), ("d", "History")] {
        let button = tab_button(&html, key);
        assert!(
            button.contains(&format!(">{label}<")),
            "missing or blank icons keep text visible"
        );
        assert_eq!(opening_attr(button, "data-icon-only"), None);
    }
    workspace_dispatch(&dom, "keydown", original, Some(Key::ArrowRight));
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(
        fixture.0.selected.borrow().last(),
        Some(&Some(2)),
        "ordinary keyboard navigation still skips the disabled icon tab"
    );
    workspace_dispatch(&dom, "keydown", targets.tab("b"), Some(Key::Delete));
    assert!(
        fixture.0.closes.borrow().is_empty(),
        "disabled icon tabs cannot request close"
    );
    workspace_dispatch(&dom, "click", targets.labels["Close Inspection"], None);
    assert_eq!(fixture.0.closes.borrow().as_slice(), [0]);
    dom.in_runtime(|| config.write().rounded = true);
    workspace_settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    let rounded = tab_button(&html, "a");
    assert!(rounded.contains("ix-tab-rounded-circle"));
    assert_eq!(opening_attr(rounded, "aria-label"), Some("Inspection"));
    assert_eq!(
        opening_attr(rounded, "data-icon-only"),
        None,
        "rounded presentation takes precedence"
    );
    assert!(
        !html.contains("ix-tab-close"),
        "existing rounded close-button presentation is unchanged"
    );
}

#[test]
fn changing_icon_only_presentation_keeps_native_tab_panel_ownership_and_private_work() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(WorkspaceConverter));
    let fixture = WorkspaceFixture::default();
    let (mut dom, mut targets) = workspace_setup(fixture.clone(), true);
    let original = targets.tab("a");
    let local = targets.ids["workspace-local"];
    workspace_dispatch(&dom, "click", local, None);
    workspace_settle(&mut dom, &mut targets);
    let mut config = fixture.0.config.get().unwrap();
    for icon_only in [true, false, true] {
        dom.in_runtime(|| {
            let mut config = config.write();
            config.items[0].icon = Some("home".into());
            config.items[0].icon_only = icon_only;
        });
        workspace_settle(&mut dom, &mut targets);
        assert_eq!(targets.tab("a"), original);
        assert_eq!(targets.ids["workspace-local"], local);
        let html = dioxus_ssr::render(&dom);
        assert!(
            html.contains("Local work 1"),
            "changing presentation keeps unpublished panel work"
        );
        let tab = tab_button(&html, "a");
        let panel = opening_attr(tab, "aria-controls").expect("associated panel");
        let native_id = opening_attr(tab, "id").unwrap();
        assert!(html.contains(&format!("id=\"{panel}\"")));
        assert!(html.contains(&format!("aria-labelledby=\"{native_id}\"")));
        assert_eq!(tab.contains(">Inspection<"), !icon_only);
    }
    workspace_dispatch(&dom, "click", targets.tab("c"), None);
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(
        fixture.0.key_selected.borrow().last(),
        Some(&Some("c".into()))
    );
    workspace_dispatch(&dom, "click", original, None);
    workspace_settle(&mut dom, &mut targets);
    assert!(dioxus_ssr::render(&dom).contains("Local work 1"));
    workspace_dispatch(&dom, "keydown", original, Some(Key::Delete));
    assert_eq!(fixture.0.key_closes.borrow().as_slice(), ["a"]);
    assert_eq!(
        fixture.0.key_selected.borrow().last(),
        Some(&Some("a".into()))
    );
}
