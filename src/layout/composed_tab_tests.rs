#[derive(Default)]
struct LabelObservation {
    stamp: Cell<Option<Signal<usize>>>,
    mounts: Cell<usize>,
    drops: Cell<usize>,
}
#[derive(Clone)]
struct LabelContext {
    stamp: Signal<usize>,
    observation: Rc<LabelObservation>,
}
#[component]
fn ComposedBody(item: TabItem) -> Element {
    let context = use_context::<LabelContext>();
    let mut local = use_signal(|| 0usize);
    let mounted = context.observation.clone();
    use_hook(move || mounted.mounts.set(mounted.mounts.get() + 1));
    let stamp = (context.stamp)();
    use_drop(move || {
        context
            .observation
            .drops
            .set(context.observation.drops.get() + 1)
    });
    rsx! {span {id:format!("composed-label-{}",item.key),onmounted:move |_|local+=1,
        "Custom {item.label} / update {stamp} / local {local}"
    }}
}
fn composed_fixture(fixture: WorkspaceFixture) -> Element {
    let config = workspace_config(&fixture);
    let observation = consume_context::<Rc<LabelObservation>>();
    let stamp = use_signal(|| 0usize);
    observation.stamp.set(Some(stamp));
    use_context_provider(|| LabelContext { stamp, observation });
    // The callback composes a component; its child owns hooks and subscribes to
    // live context. Updating stamp need not replace or refresh this callback.
    let renderer = use_callback(|item: TabItem| rsx! {ComposedBody {item}});
    let before = fixture.clone();
    let selected = fixture.clone();
    let close = fixture.clone();
    rsx! {
        crate::TabSet {items:config.items.clone(),render_label:Some(renderer),rounded:config.rounded,
            overflow_menu:config.overflow,aria_label_more_tabs:config.more_label.clone(),close_label:close_formatter(&config),
            onbeforechange:move|event:crate::TabKeySelectionChange|{before.0.requests.set(before.0.requests.get()+1);if config.veto {event.prevent_default();}},
            onselectionchange:move|key|selected.0.key_selected.borrow_mut().push(key),
            onclose:move|key|close.0.key_closes.borrow_mut().push(key),
            for item in config.items {crate::TabPanel {key:"{item.key}",tab_key:item.key.clone(),
                if item.key=="a" {RetainedWorkspaceBody {}}else{"{item.label} body"}
            }}
        }
    }
}
fn composed_setup() -> (
    VirtualDom,
    WorkspaceTargets,
    WorkspaceFixture,
    Rc<LabelObservation>,
) {
    let fixture = WorkspaceFixture::default();
    let observation = Rc::new(LabelObservation::default());
    let mut dom = VirtualDom::new_with_props(composed_fixture, fixture.clone())
        .with_root_context(observation.clone());
    let mut targets = WorkspaceTargets::default();
    targets.collect(dom.rebuild_to_vec());
    workspace_settle(&mut dom, &mut targets);
    (dom, targets, fixture, observation)
}
fn composed_button<'a>(html: &'a str, key: &str) -> &'a str {
    let at = html
        .find(&format!("-tab-{}\"", crate::composition::tab_key_id(key)))
        .unwrap();
    let start = html[..at].rfind("<button ").unwrap();
    let end = at + html[at..].find("</button>").unwrap() + "</button>".len();
    &html[start..end]
}
fn composed_attr<'a>(html: &'a str, name: &str) -> Option<&'a str> {
    let value = html
        .split('>')
        .next()
        .unwrap()
        .split_once(&format!(" {name}="))?
        .1;
    Some(if let Some(value) = value.strip_prefix('"') {
        value.split('"').next().unwrap()
    } else {
        value.split_whitespace().next().unwrap()
    })
}
#[test]
fn composed_labels_name_native_tabs_and_keep_counter_close_overflow_and_rounded_semantics() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(WorkspaceConverter));
    let (mut dom, mut targets, fixture, _) = composed_setup();
    let mut config = fixture.0.config.get().unwrap();
    dom.in_runtime(|| {
        let mut config = config.write();
        config.items[0].counter = Some(4);
        config.overflow = true;
    });
    workspace_settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    let tab = composed_button(&html, "a");
    assert_eq!(composed_attr(tab, "aria-label"), Some("Inspection"));
    assert_eq!(composed_attr(tab, "title"), Some("Inspection"));
    assert!(tab.contains("ix-tab-custom-label") && tab.contains("Custom Inspection"));
    assert!(tab.contains("ix-tab-counter") && tab.contains(">4<"));
    assert!(html.contains("aria-label=\"Close Inspection\""));
    assert!(
        html.contains("data-ix-dropdown-label=\"Inspection\""),
        "the menu uses the semantic item label"
    );
    let native = composed_attr(tab, "id").unwrap();
    let panel = composed_attr(tab, "aria-controls").unwrap();
    assert!(
        html.contains(&format!("id=\"{panel}\""))
            && html.contains(&format!("aria-labelledby=\"{native}\""))
    );
    workspace_dispatch(&dom, "keydown", targets.tab("a"), Some(Key::ArrowRight));
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(
        fixture.0.key_selected.borrow().last(),
        Some(&Some("c".into())),
        "ordinary navigation still skips disabled tabs"
    );
    workspace_dispatch(&dom, "keydown", targets.tab("b"), Some(Key::Delete));
    assert!(fixture.0.key_closes.borrow().is_empty());
    workspace_dispatch(&dom, "click", targets.labels["Close Inspection"], None);
    assert_eq!(fixture.0.key_closes.borrow().as_slice(), ["a"]);
    dom.in_runtime(|| config.write().rounded = true);
    workspace_settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    let rounded = composed_button(&html, "a");
    assert!(rounded.contains("ix-tab-rounded-circle") && rounded.contains("Custom Inspection"));
    assert!(!rounded.contains("ix-tab-custom-label"));
    assert_eq!(composed_attr(rounded, "aria-label"), Some("Inspection"));
    assert!(
        !html.contains("ix-tab-close"),
        "rounded tabs keep their established control presentation"
    );
}
#[test]
fn reactive_composed_children_survive_icon_only_hide_show_and_keyed_reorder_with_panel_work() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(WorkspaceConverter));
    let (mut dom, mut targets, fixture, observation) = composed_setup();
    let tab = targets.tab("a");
    let label = targets.ids["composed-label-a"];
    let panel_control = targets.ids["workspace-local"];
    workspace_dispatch(&dom, "mounted", label, None);
    workspace_dispatch(&dom, "click", panel_control, None);
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(observation.mounts.get(), 3);
    let mut config = fixture.0.config.get().unwrap();
    for icon_only in [true, false, true] {
        dom.in_runtime(|| {
            let mut config = config.write();
            config.items[0].icon = Some("home".into());
            config.items[0].icon_only = icon_only;
        });
        workspace_settle(&mut dom, &mut targets);
        assert_eq!(targets.tab("a"), tab);
        assert_eq!(targets.ids["composed-label-a"], label);
        let html = dioxus_ssr::render(&dom);
        let button = composed_button(&html, "a");
        let custom = button
            .split('<')
            .find(|tag| tag.starts_with("span ") && tag.contains("ix-tab-custom-label"))
            .unwrap()
            .split('>')
            .next()
            .unwrap();
        assert_eq!(custom.contains("hidden"), icon_only);
        assert!(
            button.contains("local 1"),
            "hidden composed content retains its private mounted state"
        );
        assert!(html.contains("Local work 1"));
    }
    let mut stamp = observation.stamp.get().unwrap();
    dom.in_runtime(|| stamp.set(1));
    workspace_settle(&mut dom, &mut targets);
    assert!(
        composed_button(&dioxus_ssr::render(&dom), "a").contains("update 1"),
        "the child subscribes to current context without renderer revision machinery"
    );
    dom.in_runtime(|| {
        let mut config = config.write();
        config.items.reverse();
        config.items[2].label = "Inspection revised".into();
        config.items[2].icon_only = false;
    });
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(targets.tab("a"), tab);
    assert_eq!(targets.ids["composed-label-a"], label);
    assert_eq!(targets.ids["workspace-local"], panel_control);
    let html = dioxus_ssr::render(&dom);
    let button = composed_button(&html, "a");
    assert_eq!(
        composed_attr(button, "aria-label"),
        Some("Inspection revised")
    );
    assert!(
        button.contains("Custom Inspection revised")
            && button.contains("local 1")
            && html.contains("Local work 1")
    );
    assert_eq!((observation.mounts.get(), observation.drops.get()), (3, 0));
    dom.in_runtime(|| config.write().veto = true);
    workspace_settle(&mut dom, &mut targets);
    workspace_dispatch(&dom, "click", targets.tab("c"), None);
    workspace_settle(&mut dom, &mut targets);
    assert!(
        fixture.0.key_selected.borrow().is_empty(),
        "current vetoes still govern composed tab activation"
    );
    dom.in_runtime(|| config.write().veto = false);
    workspace_settle(&mut dom, &mut targets);
    workspace_dispatch(&dom, "click", targets.tab("c"), None);
    workspace_settle(&mut dom, &mut targets);
    assert_eq!(
        fixture.0.key_selected.borrow().last(),
        Some(&Some("c".into()))
    );
    workspace_dispatch(&dom, "keydown", tab, Some(Key::Delete));
    assert_eq!(fixture.0.key_closes.borrow().as_slice(), ["a"]);
}
