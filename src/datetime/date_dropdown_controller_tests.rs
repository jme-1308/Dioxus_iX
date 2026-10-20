use super::*;

#[derive(Clone)]
struct QuerySettings {
    controller: Option<DateDropdownController>,
    use_hook_controller: bool,
    hook_handles: Rc<RefCell<Vec<DateDropdownController>>>,
    changes: Rc<RefCell<Vec<DateRangeChange>>>,
    id: String,
    from: Option<IxDate>,
    to: Option<IxDate>,
    copies: usize,
}

fn day(day: u8) -> IxDate {
    IxDate::new(2026, 10, day).unwrap()
}

fn expected_range(id: &str, from: Option<IxDate>, to: Option<IxDate>) -> DateRangeChange {
    DateRangeChange {
        id: id.into(),
        dates: DateSelection { from, to },
    }
}

fn query_settings(controller: DateDropdownController) -> Rc<RefCell<QuerySettings>> {
    Rc::new(RefCell::new(QuerySettings {
        controller: Some(controller),
        use_hook_controller: false,
        hook_handles: Rc::default(),
        changes: Rc::default(),
        id: "recent".into(),
        from: None,
        to: None,
        copies: 1,
    }))
}

fn query_fixture(settings: Rc<RefCell<QuerySettings>>) -> Element {
    let settings = settings.borrow().clone();
    let hook_controller = use_date_dropdown();
    settings
        .hook_handles
        .borrow_mut()
        .push(hook_controller.clone());
    let controller = if settings.use_hook_controller {
        Some(hook_controller)
    } else {
        settings.controller
    };
    rsx! {
        for index in 0..settings.copies {
            DateDropdown {
                key: "{index}",
                controller: controller.clone(),
                today: day(20),
                from: settings.from,
                to: settings.to,
                date_range_id: settings.id.clone(),
                options: vec![DateDropdownOption {
                    id: "recent".into(),
                    label: "Recent inspections".into(),
                    from: day(14),
                    to: day(20),
                }],
                onrangechange: {
                    let changes = settings.changes.clone();
                    move |range| changes.borrow_mut().push(range)
                },
            }
        }
    }
}

fn settle_query(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
}

fn refresh_query(dom: &mut VirtualDom) {
    dom.mark_dirty(ScopeId::APP);
    settle_query(dom);
}

#[test]
fn date_dropdown_query_reads_closed_preset_without_publication_and_returns_owned_values() {
    let controller = DateDropdownController::default();
    assert_eq!(
        controller.get_date_range(),
        Err(DateDropdownError::NotMounted)
    );
    let settings = query_settings(controller.clone());
    let mut dom = VirtualDom::new_with_props(query_fixture, settings.clone());
    dom.rebuild_in_place();
    settle_query(&mut dom);
    let before = dioxus_ssr::render(&dom);
    assert!(before.contains("aria-expanded=false") || before.contains("aria-expanded=\"false\""));
    let expected = expected_range("recent", Some(day(14)), Some(day(20)));
    let mut snapshot = controller.get_date_range().unwrap();
    assert_eq!(snapshot, expected);
    snapshot.id = "caller-owned".into();
    snapshot.dates.from = Some(day(1));
    assert_eq!(controller.get_date_range(), Ok(expected));
    settle_query(&mut dom);
    assert_eq!(dioxus_ssr::render(&dom), before);
    assert!(settings.borrow().changes.borrow().is_empty());
}

#[test]
fn date_dropdown_hook_stays_stable_and_query_follows_controlled_range_updates() {
    let settings = query_settings(DateDropdownController::default());
    settings.borrow_mut().use_hook_controller = true;
    let mut dom = VirtualDom::new_with_props(query_fixture, settings.clone());
    dom.rebuild_in_place();
    settle_query(&mut dom);
    let controller = settings.borrow().hook_handles.borrow()[0].clone();
    assert_eq!(
        controller.get_date_range(),
        Ok(expected_range("recent", Some(day(14)), Some(day(20))))
    );
    {
        let mut settings = settings.borrow_mut();
        settings.id = "custom".into();
        settings.from = Some(day(12));
        settings.to = Some(day(18));
    }
    refresh_query(&mut dom);
    assert_eq!(
        controller.get_date_range(),
        Ok(expected_range("custom", Some(day(12)), Some(day(18))))
    );
    {
        let mut settings = settings.borrow_mut();
        settings.from = None;
        settings.to = None;
    }
    refresh_query(&mut dom);
    assert_eq!(
        controller.get_date_range(),
        Ok(expected_range("custom", None, None))
    );
    let settings = settings.borrow();
    let handles = settings.hook_handles.borrow();
    assert!(handles.len() > 1);
    assert!(handles.iter().all(|handle| handle == &controller));
    assert!(settings.changes.borrow().is_empty());
}

#[test]
fn date_dropdown_query_handles_ambiguity_replacement_detachment_and_scope_teardown() {
    let original = DateDropdownController::default();
    let settings = query_settings(original.clone());
    let mut dom = VirtualDom::new_with_props(query_fixture, settings.clone());
    dom.rebuild_in_place();
    settle_query(&mut dom);
    let expected = expected_range("recent", Some(day(14)), Some(day(20)));
    assert_eq!(original.get_date_range(), Ok(expected.clone()));

    settings.borrow_mut().copies = 2;
    refresh_query(&mut dom);
    assert_eq!(
        original.get_date_range(),
        Err(DateDropdownError::MultipleMounted)
    );
    settings.borrow_mut().copies = 1;
    refresh_query(&mut dom);
    assert_eq!(original.get_date_range(), Ok(expected.clone()));

    let replacement = DateDropdownController::default();
    settings.borrow_mut().controller = Some(replacement.clone());
    refresh_query(&mut dom);
    assert_eq!(
        original.get_date_range(),
        Err(DateDropdownError::NotMounted)
    );
    assert_eq!(replacement.get_date_range(), Ok(expected.clone()));
    settings.borrow_mut().controller = None;
    refresh_query(&mut dom);
    assert_eq!(
        replacement.get_date_range(),
        Err(DateDropdownError::NotMounted)
    );
    settings.borrow_mut().controller = Some(replacement.clone());
    refresh_query(&mut dom);
    assert_eq!(replacement.get_date_range(), Ok(expected.clone()));

    settings.borrow_mut().copies = 0;
    refresh_query(&mut dom);
    assert_eq!(
        replacement.get_date_range(),
        Err(DateDropdownError::NotMounted)
    );
    settings.borrow_mut().copies = 1;
    refresh_query(&mut dom);
    assert_eq!(replacement.get_date_range(), Ok(expected));
    drop(dom);
    assert_eq!(
        replacement.get_date_range(),
        Err(DateDropdownError::NotMounted)
    );
    assert!(settings.borrow().changes.borrow().is_empty());
}
