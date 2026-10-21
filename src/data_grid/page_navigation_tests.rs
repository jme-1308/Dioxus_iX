use super::page_edit_tests::{
    Converter, Keyboard, Observation, Targets, accept, assert_idle, assert_loading, cell_dom_id,
    dispatch, finish_loading, settle, setup,
};
use super::*;

fn navigation(host: bool, page: usize) -> Rc<Observation> {
    let mut data = Observation::new(false, page);
    let owner = Rc::get_mut(&mut data).unwrap();
    owner.settings.host = host;
    owner.settings.navigate = true;
    owner.initial.selected.insert("row3".into());
    data
}
fn key_at(dom: &VirtualDom, id: dioxus::core::ElementId, key: Key) -> bool {
    let data = Rc::new(dioxus::html::PlatformEventData::new(Box::new(Keyboard {
        key,
        backwards: false,
    }))) as Rc<dyn std::any::Any>;
    let event = Event::new(data, true);
    dom.runtime().handle_event("keydown", event.clone(), id);
    event.default_action_enabled()
}
fn focus_cell(
    dom: &mut VirtualDom,
    targets: &mut Targets,
    data: &Observation,
    row: &str,
    column: &str,
) -> dioxus::core::ElementId {
    let id = cell_dom_id(dom, row, column);
    let id = targets.target("id", &id);
    dispatch(dom, "focus", id, ());
    settle(dom, targets, data);
    id
}
fn cell_key(
    dom: &mut VirtualDom,
    targets: &mut Targets,
    data: &Observation,
    row: &str,
    column: &str,
    key: Key,
) -> bool {
    let id = focus_cell(dom, targets, data, row, column);
    key_at(dom, id, key)
}
fn assert_cell_focus(dom: &VirtualDom, data: &Observation, row: &str, column: &str) {
    assert_eq!(
        data.focuses.borrow().last(),
        Some(&cell_dom_id(dom, row, column))
    );
}
fn selection_key(
    dom: &mut VirtualDom,
    targets: &mut Targets,
    data: &Observation,
    row: usize,
    key: Key,
) {
    let id = targets.target("aria-label", &format!("Select Value{row}"));
    dispatch(dom, "focus", id, ());
    settle(dom, targets, data);
    key_at(dom, id, key);
}
fn group_key(
    dom: &mut VirtualDom,
    targets: &mut Targets,
    data: &Observation,
    group: &str,
    key: Key,
) {
    dispatch(
        dom,
        "focus",
        targets.target("aria-label", &format!("Group: {group}")),
        (),
    );
    settle(dom, targets, data);
    key_at(dom, targets.target("data-group-value", group), key);
}
fn assert_unchanged_selection(data: &Observation) {
    assert_eq!(
        data.state.get().unwrap().peek().selected,
        BTreeSet::from(["row3".into()])
    );
    assert!(
        data.changes.borrow().is_empty(),
        "navigation never publishes an edit"
    );
}

#[test]
fn default_boundary_and_native_tab_remain_local_while_opt_in_roundtrips_global_endpoints() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Observation::new(false, 0);
    let (mut dom, mut targets) = setup(data.clone());
    cell_key(
        &mut dom,
        &mut targets,
        &data,
        "row1",
        "note",
        Key::ArrowDown,
    );
    settle(&mut dom, &mut targets, &data);
    assert!(data.requests.borrow().is_empty());
    assert!(
        cell_key(&mut dom, &mut targets, &data, "row1", "group", Key::Tab),
        "ordinary Tab still exits the current page"
    );

    let data = navigation(false, 0);
    let (mut dom, mut targets) = setup(data.clone());
    assert!(!dioxus_ssr::render(&dom).contains("data-page-cell=\"row2:note\""));
    cell_key(
        &mut dom,
        &mut targets,
        &data,
        "row1",
        "note",
        Key::ArrowDown,
    );
    assert_eq!(
        data.requests
            .borrow()
            .last()
            .unwrap()
            .pagination
            .unwrap()
            .page,
        1
    );
    // Read-only is an editing restriction, not a reason to reject navigation.
    dom.in_runtime(|| {
        data.options
            .get()
            .unwrap()
            .with_mut(|options| options.read_only = true)
    });
    accept(&mut dom, &mut targets, &data);
    assert_cell_focus(&dom, &data, "row2", "note");
    cell_key(&mut dom, &mut targets, &data, "row2", "note", Key::ArrowUp);
    accept(&mut dom, &mut targets, &data);
    assert_cell_focus(&dom, &data, "row1", "note");
    cell_key(&mut dom, &mut targets, &data, "row1", "note", Key::End);
    accept(&mut dom, &mut targets, &data);
    assert_cell_focus(&dom, &data, "row5", "group");
    cell_key(&mut dom, &mut targets, &data, "row5", "group", Key::Home);
    accept(&mut dom, &mut targets, &data);
    assert_cell_focus(&dom, &data, "row0", "value");
    let previous = data.requests.borrow().len();
    assert!(
        cell_key(&mut dom, &mut targets, &data, "row1", "group", Key::Tab),
        "navigation opt-in does not capture normal Tab exit"
    );
    assert_eq!(data.requests.borrow().len(), previous);
    assert_unchanged_selection(&data);
    assert_idle(&dom);
}

#[test]
fn page_keys_preserve_the_measured_visual_step_and_selection_lane_across_client_and_host_pages() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for host in [false, true] {
        let data = navigation(host, 0);
        data.echo.set(true);
        let (mut dom, mut targets) = setup(data.clone());
        selection_key(&mut dom, &mut targets, &data, 1, Key::PageDown);
        settle(&mut dom, &mut targets, &data);
        // (192px measured height - 48px header) / 42px rows = three slots.
        assert_eq!(data.state.get().unwrap().peek().pagination.unwrap().page, 2);
        assert_eq!(
            data.focuses.borrow().last().map(String::as_str),
            Some("Select Value4")
        );
        selection_key(&mut dom, &mut targets, &data, 4, Key::PageUp);
        settle(&mut dom, &mut targets, &data);
        assert_eq!(data.state.get().unwrap().peek().pagination.unwrap().page, 0);
        assert_eq!(
            data.focuses.borrow().last().map(String::as_str),
            Some("Select Value1")
        );
        selection_key(&mut dom, &mut targets, &data, 1, Key::End);
        settle(&mut dom, &mut targets, &data);
        assert_eq!(
            data.focuses.borrow().last().map(String::as_str),
            Some("Select Value5")
        );
        selection_key(&mut dom, &mut targets, &data, 5, Key::Home);
        settle(&mut dom, &mut targets, &data);
        assert_eq!(
            data.focuses.borrow().last().map(String::as_str),
            Some("Select Value0")
        );
        assert_unchanged_selection(&data);
        assert_idle(&dom);
    }
}

#[test]
fn grouped_navigation_visits_real_headers_but_skips_repeated_or_closed_host_headers_to_finite_ends()
{
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let mut data = navigation(false, 0);
    Rc::get_mut(&mut data).unwrap().initial.grouping = crate::DataGridGrouping::new(["group"]);
    let (mut dom, mut targets) = setup(data.clone());
    cell_key(
        &mut dom,
        &mut targets,
        &data,
        "row1",
        "note",
        Key::ArrowDown,
    );
    accept(&mut dom, &mut targets, &data);
    assert_eq!(
        data.focuses.borrow().last().map(String::as_str),
        Some("Group: B")
    );
    group_key(&mut dom, &mut targets, &data, "B", Key::ArrowDown);
    settle(&mut dom, &mut targets, &data);
    assert_eq!(
        data.focuses.borrow().last().map(String::as_str),
        Some("Select Value2")
    );

    // A manually focused continuation header represents this page boundary,
    // not the start of the same group on a previous page.
    let mut data = navigation(false, 1);
    Rc::get_mut(&mut data).unwrap().initial.grouping = crate::DataGridGrouping::new(["group"]);
    data.echo.set(true);
    let (mut dom, mut targets) = setup(data.clone());
    dom.in_runtime(|| {
        data.rows.get().unwrap().with_mut(|rows| {
            for row in &mut rows[2..4] {
                row.cells.insert("group".into(), "A".into());
            }
            for row in &mut rows[4..] {
                row.cells.insert("group".into(), "B".into());
            }
        })
    });
    settle(&mut dom, &mut targets, &data);
    group_key(&mut dom, &mut targets, &data, "A", Key::ArrowUp);
    settle(&mut dom, &mut targets, &data);
    assert_eq!(data.state.get().unwrap().peek().pagination.unwrap().page, 0);
    assert_eq!(
        data.focuses.borrow().last().map(String::as_str),
        Some("Select Value1")
    );
    dom.in_runtime(|| {
        data.rows.get().unwrap().with_mut(|rows| {
            for row in rows {
                row.cells.insert("group".into(), "A".into());
            }
        });
        data.state.get().unwrap().with_mut(|state| {
            state.pagination.as_mut().unwrap().page = 1;
            state.set_group_expanded(&crate::DataGridGroupPath::new([("group", "A")]), false);
        });
    });
    settle(&mut dom, &mut targets, &data);
    let previous = data.requests.borrow().len();
    group_key(&mut dom, &mut targets, &data, "A", Key::ArrowDown);
    settle(&mut dom, &mut targets, &data);
    assert_eq!(data.state.get().unwrap().peek().pagination.unwrap().page, 1);
    assert_eq!(
        data.requests.borrow().len(),
        previous,
        "a trailing closed continuation cannot wrap backward to its first header"
    );

    let mut data = navigation(true, 0);
    Rc::get_mut(&mut data).unwrap().initial.grouping = crate::DataGridGrouping::new(["group"]);
    data.echo.set(true);
    let (mut dom, mut targets) = setup(data.clone());
    dom.in_runtime(|| {
        data.rows.get().unwrap().with_mut(|rows| {
            for row in &mut rows[2..4] {
                row.cells.insert("group".into(), "A".into());
            }
            for row in &mut rows[4..] {
                row.cells.insert("group".into(), "B".into());
            }
        })
    });
    settle(&mut dom, &mut targets, &data);
    cell_key(
        &mut dom,
        &mut targets,
        &data,
        "row1",
        "note",
        Key::ArrowDown,
    );
    settle(&mut dom, &mut targets, &data);
    assert_cell_focus(&dom, &data, "row2", "note");
    cell_key(
        &mut dom,
        &mut targets,
        &data,
        "row3",
        "note",
        Key::ArrowDown,
    );
    settle(&mut dom, &mut targets, &data);
    assert_eq!(
        data.focuses.borrow().last().map(String::as_str),
        Some("Group: B")
    );
    dom.in_runtime(|| {
        data.state.get().unwrap().with_mut(|state| {
            state.pagination.as_mut().unwrap().page = 0;
            state.set_group_expanded(&crate::DataGridGroupPath::new([("group", "A")]), false);
        })
    });
    settle(&mut dom, &mut targets, &data);
    let previous = data.requests.borrow().len();
    group_key(&mut dom, &mut targets, &data, "A", Key::ArrowDown);
    settle(&mut dom, &mut targets, &data);
    assert_eq!(data.requests.borrow().len(), previous + 2);
    assert_eq!(data.state.get().unwrap().peek().pagination.unwrap().page, 2);
    assert_eq!(
        data.focuses.borrow().last().map(String::as_str),
        Some("Group: B")
    );
    group_key(&mut dom, &mut targets, &data, "B", Key::ArrowDown);
    settle(&mut dom, &mut targets, &data);
    assert_eq!(
        data.focuses.borrow().last().map(String::as_str),
        Some("Select Value4")
    );
    selection_key(&mut dom, &mut targets, &data, 4, Key::End);
    settle(&mut dom, &mut targets, &data);
    let previous = data.requests.borrow().len();
    selection_key(&mut dom, &mut targets, &data, 5, Key::ArrowDown);
    settle(&mut dom, &mut targets, &data);
    assert_eq!(data.requests.borrow().len(), previous);
    assert_unchanged_selection(&data);
    assert_idle(&dom);
}

#[test]
fn held_or_incompatible_page_replies_retire_navigation_without_late_focus() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for refusal in ["held", "page", "query", "column", "source"] {
        let data = navigation(refusal != "source", 0);
        let (mut dom, mut targets) = setup(data.clone());
        let origin = cell_dom_id(&dom, "row1", "note");
        let previous_focuses = data.focuses.borrow().len();
        cell_key(
            &mut dom,
            &mut targets,
            &data,
            "row1",
            "note",
            Key::ArrowDown,
        );
        let requested = data.requests.borrow().last().unwrap().clone();
        dom.in_runtime(|| {
            if refusal == "held" {
                return;
            }
            let mut accepted = requested.clone();
            match refusal {
                "page" => accepted.pagination.as_mut().unwrap().page = 2,
                "query" => {
                    accepted.sort = Some(("value".into(), DataGridSortDirection::Descending))
                }
                "column" => {
                    accepted.column_order = vec!["note".into(), "value".into(), "group".into()]
                }
                "source" => data
                    .rows
                    .get()
                    .unwrap()
                    .write()
                    .retain(|row| row.id != "row2"),
                _ => unreachable!(),
            }
            data.state.get().unwrap().set(accepted);
        });
        settle(&mut dom, &mut targets, &data);
        assert_idle(&dom);
        if refusal == "held" {
            assert_eq!(data.state.get().unwrap().peek().pagination.unwrap().page, 0);
            assert_eq!(cell_dom_id(&dom, "row1", "note"), origin);
            assert_eq!(
                data.focuses.borrow().len(),
                previous_focuses,
                "the still-focused origin needs no redundant mounted focus"
            );
        }
        let focuses = data.focuses.borrow().len();
        dom.in_runtime(|| data.state.get().unwrap().set(requested));
        settle(&mut dom, &mut targets, &data);
        assert_eq!(
            data.focuses.borrow().len(),
            focuses,
            "late {refusal} response revived focus"
        );
        assert_eq!(data.requests.borrow().len(), 1);
        assert_unchanged_selection(&data);
    }
}

#[test]
fn delayed_navigation_defers_capacity_and_independent_input_option_or_teardown_retires_it() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for intent in [
        "accept", "input", "key", "pointer", "option", "source", "teardown",
    ] {
        let data = navigation(true, 0);
        let (mut dom, mut targets) = setup(data.clone());
        cell_key(
            &mut dom,
            &mut targets,
            &data,
            "row1",
            "note",
            Key::ArrowDown,
        );
        let requested = data.requests.borrow().last().unwrap().clone();
        dom.in_runtime(|| {
            data.options
                .get()
                .unwrap()
                .with_mut(|settings| settings.loading = true)
        });
        settle(&mut dom, &mut targets, &data);
        assert_loading(&dom);
        let grid = targets.target("aria-label", "Cross-page worklist");
        if intent == "accept" {
            dom.in_runtime(|| {
                data.options
                    .get()
                    .unwrap()
                    .with_mut(|settings| settings.auto = true)
            });
            dispatch(&dom, "resize", grid, 90.0);
            settle(&mut dom, &mut targets, &data);
            assert_eq!(
                data.requests.borrow().len(),
                1,
                "pending navigation defers automatic capacity"
            );
            finish_loading(&mut dom, &mut targets, &data, requested);
            assert_cell_focus(&dom, &data, "row2", "note");
        } else {
            match intent {
                "input" => {
                    dispatch(&dom, "focusout", grid, ());
                    dispatch(&dom, "focus", targets.target("id", "outside-page-grid"), ());
                }
                "key" => {
                    key_at(&dom, grid, Key::Escape);
                }
                "pointer" => dispatch(&dom, "pointerdown", grid, ()),
                "source" => dom.in_runtime(|| {
                    data.rows
                        .get()
                        .unwrap()
                        .write()
                        .retain(|row| row.id != "row2")
                }),
                _ => dom.in_runtime(|| {
                    data.options.get().unwrap().with_mut(|settings| {
                        if intent == "option" {
                            settings.navigate = false;
                        } else {
                            settings.mounted = false;
                        }
                    })
                }),
            }
            settle(&mut dom, &mut targets, &data);
            let focuses = data.focuses.borrow().len();
            finish_loading(&mut dom, &mut targets, &data, requested);
            dom.in_runtime(|| {
                data.options.get().unwrap().with_mut(|settings| {
                    settings.navigate = true;
                    settings.mounted = true;
                })
            });
            settle(&mut dom, &mut targets, &data);
            assert_eq!(
                data.focuses.borrow().len(),
                focuses,
                "late response after {intent} stole focus"
            );
            assert_eq!(data.requests.borrow().len(), 1);
        }
        assert_unchanged_selection(&data);
        assert_idle(&dom);
    }
}
