//! Native Dioxus drag ownership; real browser drag continuity is checked by the gallery replay.
use super::*;
use std::task::{Context, Wake};
use std::time::{Duration, Instant};

struct Notify(std::thread::Thread);
impl Wake for Notify {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}
fn until(h: &mut Harness, done: impl Fn(&Harness) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    let waker = Waker::from(Arc::new(Notify(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    loop {
        h.settle();
        if done(h) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "native drag workflow did not produce its controlled response"
        );
        let pending = std::pin::pin!(h.dom.wait_for_work())
            .poll(&mut context)
            .is_pending();
        if pending {
            std::thread::park_timeout(deadline.saturating_duration_since(Instant::now()));
        }
    }
}
fn start(h: &mut Harness, row: usize) -> (ElementId, Transfer) {
    let handle = h.target("aria-label", &format!("Move Asset {row}"));
    let transfer = Transfer::default();
    h.event(
        "dragstart",
        handle,
        Drag {
            transfer: transfer.clone(),
            ..Default::default()
        },
    );
    h.settle();
    (handle, transfer)
}
fn hover(h: &mut Harness, y: f64, transfer: &Transfer) {
    h.event(
        "dragover",
        h.target("aria-label", "Movable equipment"),
        Drag {
            y,
            transfer: transfer.clone(),
        },
    );
    h.settle();
}
fn page(h: &Harness) -> usize {
    h.data.state.get().unwrap().peek().pagination.unwrap().page
}
fn retained(h: &Harness) -> bool {
    dioxus_ssr::render(&h.dom).contains("data-drag-source-retained=\"true\"")
}
fn active(h: &Harness) -> bool {
    dioxus_ssr::render(&h.dom).contains("data-row-drag-source=")
}
fn new_drag(host: bool) -> Harness {
    Harness::new(Settings {
        host,
        across: true,
        ..Default::default()
    })
}

#[test]
fn owned_native_drag_retains_original_handle_across_client_and_host_pages_and_drops_globally() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for host in [false, true] {
        let mut h = new_drag(host);
        let (source, transfer) = start(&mut h, 1);
        let source_row = h.target("data-row-id", "r1");
        let mounts = h.data.mount_counts.borrow()["Move Asset 1"];
        hover(&mut h, 180.0, &transfer);
        until(&mut h, |h| page(h) == 1);
        hover(&mut h, 120.0, &transfer);
        assert!(retained(&h));
        assert_eq!(h.target("aria-label", "Move Asset 1"), source);
        assert_eq!(h.data.mount_counts.borrow()["Move Asset 1"], mounts);
        assert_eq!(
            dioxus::html::NativeDataTransfer::get_data(&transfer, "text/plain").as_deref(),
            Some("r1")
        );
        let gate = Rc::new(Gate::default());
        *h.data.geometry.borrow_mut() = Some(("r3".into(), gate.clone()));
        h.event(
            "drop",
            h.target("data-row-id", "r3"),
            Drag {
                y: 180.0,
                transfer: transfer.clone(),
            },
        );
        h.settle();
        assert_eq!(gate.reads.get(), 1);
        // A native dragend follows drop before the mounted geometry reply.
        h.event(
            "dragend",
            source,
            Drag {
                transfer: transfer.clone(),
                ..Default::default()
            },
        );
        h.settle();
        assert!(active(&h));
        h.dom.in_runtime(|| gate.release());
        h.settle();
        assert_eq!(h.order(), ["r0", "r2", "r3", "r1", "r4", "r5"]);
        assert!(!active(&h) && !retained(&h));
        assert_eq!(h.target("data-row-id", "r1"), source_row);
        assert_eq!(h.target("aria-label", "Move Asset 1"), source);
        // The renderer preserves omitted style properties, so returning this
        // actual node must explicitly restore visibility and pointer admission.
        let current_style = &h.attrs[&source_row]["style"];
        assert!(
            current_style
                .split(';')
                .any(|property| property == "opacity:1")
        );
        assert!(
            current_style
                .split(';')
                .any(|property| property == "pointer-events:auto")
        );
        assert_eq!(h.data.mount_counts.borrow()["Move Asset 1"], mounts);
        if host {
            assert_eq!(
                h.data.moves.borrow().as_slice(),
                [DataGridPageRowMove {
                    row_id: "r1".into(),
                    from_index: 1,
                    to_index: 3,
                    total_rows: 6
                }]
            );
        } else {
            assert!(h.data.moves.borrow().is_empty());
        }
        assert_eq!(
            h.data.state.get().unwrap().peek().selected,
            BTreeSet::from(["r1".into(), "r5".into()])
        );
        *h.data.geometry.borrow_mut() = None;
        let (source, transfer) = start(&mut h, 1);
        hover(&mut h, 60.0, &transfer);
        until(&mut h, |h| page(h) == 0);
        hover(&mut h, 120.0, &transfer);
        assert!(retained(&h));
        assert_eq!(h.target("aria-label", "Move Asset 1"), source);
        h.event(
            "drop",
            h.target("data-row-id", "r0"),
            Drag { y: 10.0, transfer },
        );
        h.settle();
        assert_eq!(h.order(), ["r1", "r0", "r2", "r3", "r4", "r5"]);
        assert!(!active(&h));
    }
}

#[test]
fn default_boundaries_and_declined_or_altered_page_requests_do_not_retry_or_revive_drag() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let mut plain = Harness::new(Settings::default());
    let (_, transfer) = start(&mut plain, 1);
    hover(&mut plain, 180.0, &transfer);
    assert!(!active(&plain));
    assert!(plain.data.page_requests.borrow().is_empty());
    for reply in [PageReply::Hold, PageReply::Alter] {
        let mut h = new_drag(true);
        h.data.page_reply.set(reply);
        let (_, transfer) = start(&mut h, 1);
        hover(&mut h, 180.0, &transfer);
        until(&mut h, |h| {
            !h.data.page_requests.borrow().is_empty() && !active(h)
        });
        assert_eq!(
            h.data.page_requests.borrow().as_slice(),
            [crate::DataGridPagination::new(1, 2)]
        );
        h.state(|state| state.pagination = Some(crate::DataGridPagination::new(1, 2)));
        hover(&mut h, 180.0, &transfer);
        h.event(
            "drop",
            h.target("data-row-id", "r3"),
            Drag { y: 180.0, transfer },
        );
        h.settle();
        assert!(h.data.moves.borrow().is_empty());
        assert_eq!(h.data.page_requests.borrow().len(), 1);
        assert_eq!(h.order(), ["r0", "r1", "r2", "r3", "r4", "r5"]);
    }
    let mut h = new_drag(true);
    h.state(|state| state.pagination = Some(crate::DataGridPagination::new(2, 2)));
    let (source, transfer) = start(&mut h, 5);
    hover(&mut h, 180.0, &transfer);
    assert!(
        h.data.page_requests.borrow().is_empty(),
        "last page has no next request"
    );
    h.event(
        "dragend",
        source,
        Drag {
            transfer,
            ..Default::default()
        },
    );
    h.settle();
    assert!(!active(&h));
}

#[test]
fn loading_keeps_only_the_owned_requested_page_and_escape_or_optout_cancels_late_work() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let mut h = new_drag(true);
    h.data.page_reply.set(PageReply::Loading);
    let (source, transfer) = start(&mut h, 1);
    hover(&mut h, 180.0, &transfer);
    until(&mut h, |h| h.data.settings.get().unwrap().peek().loading);
    hover(&mut h, 120.0, &transfer);
    assert_eq!(page(&h), 0);
    assert!(active(&h));
    h.state(|state| state.pagination = Some(crate::DataGridPagination::new(1, 2)));
    assert!(retained(&h));
    assert_eq!(h.target("aria-label", "Move Asset 1"), source);
    let html = dioxus_ssr::render(&h.dom);
    let source_tag = html
        .split('<')
        .find(|tag| tag.contains("aria-label=\"Move Asset 1\""))
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(
        !source_tag.contains("disabled"),
        "loading must not disable the native drag source"
    );
    h.settings(|settings| settings.loading = false);
    h.event(
        "keydown",
        source,
        Keyboard {
            key: Key::Escape,
            alt: false,
        },
    );
    h.settle();
    assert!(!active(&h) && !retained(&h));
    h.event(
        "drop",
        h.target("data-row-id", "r3"),
        Drag { y: 180.0, transfer },
    );
    h.settle();
    assert!(h.data.moves.borrow().is_empty());

    let mut h = new_drag(true);
    let (_, transfer) = start(&mut h, 1);
    hover(&mut h, 180.0, &transfer);
    until(&mut h, |h| page(h) == 1);
    hover(&mut h, 120.0, &transfer);
    h.settings(|settings| settings.across = false);
    assert!(!active(&h) && !retained(&h));
    h.event(
        "drop",
        h.target("data-row-id", "r3"),
        Drag { y: 180.0, transfer },
    );
    h.settle();
    assert!(h.data.moves.borrow().is_empty());
}

#[test]
fn changed_source_query_or_independent_page_retires_cross_page_native_ownership() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for changed in ["source", "sort", "page", "unmount"] {
        let mut h = new_drag(false);
        let (_, transfer) = start(&mut h, 1);
        match changed {
            "source" => {
                h.dom.in_runtime(|| {
                    h.data.rows.get().unwrap().write()[1]
                        .cells
                        .insert("value".into(), "Changed source".into());
                });
                h.settle();
            }
            "sort" => h.state(|state| {
                state.sort = Some(("value".into(), DataGridSortDirection::Descending))
            }),
            "page" => {
                h.state(|state| state.pagination = Some(crate::DataGridPagination::new(1, 2)))
            }
            _ => h.settings(|settings| settings.mounted = false),
        }
        assert!(!active(&h), "{changed} invalidates the source registration");
        assert!(h.data.moves.borrow().is_empty());
        if changed != "unmount" {
            hover(&mut h, 180.0, &transfer);
        }
        assert!(h.data.page_requests.borrow().is_empty());
    }
}
