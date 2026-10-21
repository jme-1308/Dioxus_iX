use crate::data_grid_group::GroupAutoCapacity;

#[test]
fn grouped_capacity_fits_every_window_and_is_maximal_for_unequal_nested_and_collapsed_groups() {
    let rows = vec![
        DataGridRow::new("a")
            .cell("status", "A")
            .cell("owner", "One"),
        DataGridRow::new("b")
            .cell("status", "A")
            .cell("owner", "One"),
        DataGridRow::new("c")
            .cell("status", "A")
            .cell("owner", "Two"),
        DataGridRow::new("d")
            .cell("status", "B")
            .cell("owner", "One"),
        DataGridRow::new("e")
            .cell("status", "B")
            .cell("owner", "Two"),
        DataGridRow::new("f")
            .cell("status", "C")
            .cell("owner", "Three"),
    ];
    let mut state = DataGridState {
        grouping: DataGridGrouping::new(["status", "owner"]),
        ..Default::default()
    };
    for collapsed in [
        vec![],
        vec![path(&[("status", "A"), ("owner", "One")])],
        vec![path(&[("status", "A")])],
    ] {
        state.grouping.collapsed = collapsed.into_iter().collect();
        // Exercise the same interval model with different current typed ordering.
        for direction in [
            DataGridSortDirection::Ascending,
            DataGridSortDirection::Descending,
        ] {
            state.sort = Some(("owner".into(), direction));
            let full: Vec<_> = state
                .visible_rows_with_columns(&rows, &grouping_columns())
                .into_iter()
                .cloned()
                .collect();
            let capacity = GroupAutoCapacity::new(&full, &state.grouping);
            for budget in [1, 3, 5, 7, 12] {
                let actual = capacity.page_size(budget);
                let fits = |size| {
                    (0..=full.len() - size).all(|start| {
                        group_slots(&full, &state.grouping, start..start + size)
                            .0
                            .len()
                            <= budget
                    })
                };
                let expected = (1..=full.len())
                    .filter(|size| fits(*size))
                    .max()
                    .unwrap_or(1);
                assert_eq!(actual, expected, "budget={budget}, direction={direction:?}");
                if actual > 1 {
                    assert!(fits(actual));
                }
                if actual < full.len() {
                    assert!(!fits(actual + 1));
                }
            }
        }
    }
    state.collapse_all_groups(&rows, &grouping_columns());
    let full: Vec<_> = state
        .visible_rows_with_columns(&rows, &grouping_columns())
        .into_iter()
        .cloned()
        .collect();
    assert_eq!(
        GroupAutoCapacity::new(&full, &state.grouping).page_size(3),
        rows.len(),
        "hidden descendants remain logical records"
    );
    assert_eq!(GroupAutoCapacity::new(&[], &state.grouping).page_size(7), 7);
    assert_eq!(GroupAutoCapacity::new(&[], &state.grouping).page_size(0), 1);
}

struct GroupResize(f64);
impl dioxus::html::HasResizeData for GroupResize {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_content_box_size(
        &self,
    ) -> dioxus::html::ResizeResult<dioxus::html::geometry::PixelsSize> {
        Ok(dioxus::html::geometry::PixelsSize::new(640.0, self.0))
    }
}
struct GroupAutoConverter;
macro_rules! group_auto_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for GroupAutoConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {
                let _=event;group_auto_converter!(@data $group,event)
            })*
        }
    };
    (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(RenderMouse) };
    (@data Keyboard,$event:ident) => { dioxus::html::KeyboardData::new($event.downcast::<RenderKey>().unwrap().clone()) };
    (@data Focus,$event:ident) => { dioxus::html::FocusData::new(RenderFocus) };
    (@data Form,$event:ident) => { dioxus::html::FormData::new(EditorForm($event.downcast::<String>().unwrap().clone())) };
    (@data Resize,$event:ident) => { dioxus::html::ResizeData::new(GroupResize(*$event.downcast::<f64>().unwrap())) };
    (@data $other:ident,$event:ident) => { panic!("Unexpected grouped auto test event") };
}
dioxus::html::with_html_event_groups!(group_auto_converter);
fn group_resize(
    dom: &VirtualDom,
    attrs: &BTreeMap<usize, BTreeMap<String, String>>,
    budget: usize,
) {
    let root = render_target(attrs, "aria-label", "Grouped records");
    render_dispatch(dom, "resize", root, Box::new(48.0 + 32.0 * budget as f64));
}
fn stored_page(data: &GroupObservation) -> DataGridPagination {
    data.state.get().unwrap().peek().pagination.unwrap()
}

#[test]
fn actual_group_auto_refreshes_on_expansion_filter_and_current_membership_with_controlled_acceptance(
) {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(GroupAutoConverter));
    let (mut dom, data, mut attrs) = group_setup();
    data.decline.set(true);
    dom.in_runtime(|| data.auto.get().unwrap().set(true));
    settle_render(&mut dom, &mut attrs);
    assert!(
        data.requests.borrow().is_empty(),
        "configured geometry is not a native measurement"
    );
    group_resize(&dom, &attrs, 4);
    settle_render(&mut dom, &mut attrs);
    assert!(
        data.requests.borrow().is_empty(),
        "the crossing Normal/Warning window prevents size two"
    );
    dom.in_runtime(|| {
        data.state
            .get()
            .unwrap()
            .with_mut(|state| state.set_group_expanded(&path(&[("status", "Normal")]), false))
    });
    settle_render(&mut dom, &mut attrs);
    let request = data.requests.borrow().last().unwrap().clone();
    assert_eq!(request.pagination, Some(DataGridPagination::new(0, 3)));
    assert!(request.selected.contains("outside"));
    assert_eq!(
        stored_page(&data),
        DataGridPagination::new(1, 1),
        "declined requests leave accepted paging truthful"
    );
    data.decline.set(false);
    dom.in_runtime(|| data.state.get().unwrap().set(request));
    settle_render(&mut dom, &mut attrs);
    dom.in_runtime(|| {
        data.state
            .get()
            .unwrap()
            .with_mut(DataGridState::expand_all_groups)
    });
    settle_render(&mut dom, &mut attrs);
    assert_eq!(stored_page(&data).page_size, 1);
    dom.in_runtime(|| {
        data.state.get().unwrap().with_mut(|state| {
            state.filters.insert(
                "status".into(),
                DataGridTextFilter {
                    first: DataGridFilterCondition {
                        value: "Normal".into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
        })
    });
    settle_render(&mut dom, &mut attrs);
    assert_eq!(
        stored_page(&data).page_size,
        2,
        "filtered membership reuses the latest measured budget"
    );
    dom.in_runtime(|| {
        data.rows.get().unwrap().with_mut(|rows| {
            rows[1].cells.insert("owner".into(), "Different".into());
        })
    });
    settle_render(&mut dom, &mut attrs);
    assert_eq!(
        stored_page(&data).page_size,
        1,
        "same row count with different ancestry invalidates capacity"
    );
    dom.in_runtime(|| data.rows.get().unwrap().set(Vec::new()));
    settle_render(&mut dom, &mut attrs);
    assert_eq!(
        stored_page(&data),
        DataGridPagination::new(0, 4),
        "empty results retain the measured flat capacity"
    );
}

#[test]
fn actual_group_auto_defers_until_edit_release_and_rejects_host_groups_without_metadata() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(GroupAutoConverter));
    let (mut dom, data, mut attrs) = group_setup();
    dom.in_runtime(|| {
        data.state
            .get()
            .unwrap()
            .with_mut(|state| state.pagination = Some(DataGridPagination::new(0, 2)));
        data.auto.get().unwrap().set(true);
    });
    settle_render(&mut dom, &mut attrs);
    group_resize(&dom, &attrs, 6);
    settle_render(&mut dom, &mut attrs);
    assert_eq!(stored_page(&data).page_size, 2);
    let cell = render_target(&attrs, "data-column-id", "status");
    render_dispatch(&dom, "dblclick", cell, Box::new(()));
    settle_render(&mut dom, &mut attrs);
    assert!(data.editor.borrow().as_ref().unwrap().active);
    data.requests.borrow_mut().clear();
    group_resize(&dom, &attrs, 1);
    settle_render(&mut dom, &mut attrs);
    assert!(data.requests.borrow().is_empty());
    assert_eq!(stored_page(&data).page_size, 2);
    dom.in_runtime(|| data.editor.borrow().as_ref().unwrap().cancel());
    settle_render(&mut dom, &mut attrs);
    assert_eq!(
        stored_page(&data).page_size,
        1,
        "one logical record survives impossible-small ancestry budget"
    );
    dom.in_runtime(|| data.auto.get().unwrap().set(false));
    settle_render(&mut dom, &mut attrs);
    data.requests.borrow_mut().clear();
    group_resize(&dom, &attrs, 10);
    settle_render(&mut dom, &mut attrs);
    assert!(data.requests.borrow().is_empty());
    assert_eq!(stored_page(&data).page_size, 1);
    dom.in_runtime(|| {
        data.model
            .get()
            .unwrap()
            .set(crate::DataGridRowModel::Page { total_rows: 50 });
        data.auto.get().unwrap().set(true);
    });
    settle_render(&mut dom, &mut attrs);
    group_resize(&dom, &attrs, 10);
    settle_render(&mut dom, &mut attrs);
    assert!(data.requests.borrow().is_empty());
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Grouped host pages require group metadata."));
    assert!(!html.contains("data-row-id="));
}
