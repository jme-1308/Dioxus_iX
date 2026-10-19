//! Gallery URL state. Dioxus owns browser history and Rust owns navigation policy.
//! Desktop alone needs native URI plumbing because its renderer supplies MemoryHistory.
use dioxus::prelude::*;
#[cfg(feature = "desktop")]
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

pub(super) const MAX_QUERY_CHARS: usize = 160;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct GalleryLocation {
    pub page: String,
    pub dark: bool,
    pub compact: bool,
    pub example: String,
    pub query: String,
}

impl Default for GalleryLocation {
    fn default() -> Self {
        Self {
            page: "actions".into(),
            dark: true,
            compact: false,
            example: "buttons".into(),
            query: String::new(),
        }
    }
}

impl GalleryLocation {
    pub(super) fn from_hash(hash: &str) -> Self {
        let mut location = Self::default();
        for part in hash.trim_start_matches('#').split('&') {
            let Some((key, value)) = part.split_once('=') else {
                continue;
            };
            let value = decode(value);
            match key {
                "page" if valid_page(&value) => location.page = value,
                "theme" => location.dark = value != "light",
                "density" => location.compact = value == "compact",
                "example" if dioxus_ix::UPSTREAM_EXAMPLE_NAMES.contains(&value.as_str()) => {
                    location.example = value;
                }
                "q" => location.query = bounded_query(&value),
                _ => {}
            }
        }
        location
    }

    pub(super) fn to_hash(&self) -> String {
        format!(
            "#page={}&theme={}&density={}&example={}&q={}",
            encode(&self.page),
            if self.dark { "dark" } else { "light" },
            if self.compact { "compact" } else { "normal" },
            encode(&self.example),
            encode(&bounded_query(&self.query)),
        )
    }

    pub(super) fn with_page(&self, page: &str) -> Self {
        let mut next = self.clone();
        if valid_page(page) {
            next.page = page.into();
        }
        next
    }
}

fn valid_page(page: &str) -> bool {
    matches!(
        page,
        "actions"
            | "sources"
            | "forms"
            | "layout"
            | "datetime"
            | "grid"
            | "tree"
            | "application"
            | "feedback"
            | "composition"
            | "chat"
            | "patterns"
            | "dialogs"
            | "icons"
            | "theme"
            | "coverage"
    )
}

fn bounded_query(query: &str) -> String {
    query
        .chars()
        .filter(|character| !character.is_control())
        .take(MAX_QUERY_CHARS)
        .collect()
}

fn ordinary_fragment(hash: &str) -> bool {
    let fragment = hash.trim_start_matches('#');
    !fragment.is_empty() && !fragment.contains('=')
}

fn encode(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 15)]));
        }
    }
    encoded
}

fn decode(value: &str) -> String {
    fn hex(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }
    let mut bytes = Vec::with_capacity(value.len());
    let mut input = value.bytes().peekable();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let mut candidate = input.clone();
            if let (Some(high), Some(low)) = (candidate.next(), candidate.next())
                && let (Some(high), Some(low)) = (hex(high), hex(low))
            {
                bytes.push((high << 4) | low);
                input = candidate;
                continue;
            }
        }
        bytes.push(if byte == b'+' { b' ' } else { byte });
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn history_push(previous: &GalleryLocation, next: &GalleryLocation) -> bool {
    previous.page != next.page || previous.example != next.example
}

#[cfg(any(not(feature = "desktop"), test))]
fn route_hash(route: &str) -> &str {
    route.split_once('#').map_or("", |(_, hash)| hash)
}

#[cfg(not(feature = "desktop"))]
fn write_history_location(
    history: &dyn dioxus::history::History,
    previous: &GalleryLocation,
    next: &GalleryLocation,
) {
    // WebHistory accepts full internal paths, including path/query/fragment.
    // Retaining the current path also preserves hosted gallery subdirectories.
    let route = history.current_route();
    let path = route
        .split_once('#')
        .map_or(route.as_str(), |(path, _)| path);
    let route = format!("{path}{}", next.to_hash());
    if history_push(previous, next) {
        history.push(route);
    } else {
        history.replace(route);
    }
}

#[cfg(not(feature = "desktop"))]
pub(super) fn use_gallery_location() -> Signal<GalleryLocation> {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    let history = use_hook(dioxus::history::history);
    // Read before any effect can write: a copied URL is authoritative on startup.
    let mut location =
        use_signal(|| GalleryLocation::from_hash(route_hash(&history.current_route())));
    let observed = use_hook(|| Rc::new(RefCell::new(location.peek().clone())));
    let revision = use_signal_sync(|| 0_u64);
    let alive = use_hook(|| Arc::new(AtomicBool::new(true)));
    let cleanup_alive = alive.clone();
    use_drop(move || cleanup_alive.store(false, Ordering::Relaxed));

    let subscription_history = history.clone();
    use_hook(move || {
        // Pinned WebHistory only notifies popstate and has no unsubscribe API.
        // The gallery root lives for the application lifetime; this guard retires
        // the callback safely if that owner is removed. No custom event runtime.
        subscription_history.updater(Arc::new(move || {
            let mut revision = revision;
            if alive.load(Ordering::Relaxed)
                && let Ok(mut revision) = revision.try_write()
            {
                *revision = revision.wrapping_add(1);
            }
        }));
    });

    let received = observed.clone();
    let read_history = history.clone();
    use_effect(move || {
        revision();
        let route = read_history.current_route();
        let hash = route_hash(&route);
        // Leave ordinary native fragment navigation and scrolling intact. The
        // next gallery control change writes a shareable gallery URL again.
        if ordinary_fragment(hash) {
            return;
        }
        let next = GalleryLocation::from_hash(hash);
        *received.borrow_mut() = next.clone();
        if *location.peek() != next {
            location.set(next);
        }
    });

    use_effect(move || {
        let next = location();
        let previous = observed.borrow().clone();
        if previous == next {
            return;
        }
        write_history_location(history.as_ref(), &previous, &next);
        *observed.borrow_mut() = next;
    });
    location
}

#[cfg(feature = "desktop")]
pub(super) fn use_gallery_location() -> Signal<GalleryLocation> {
    let mut location = use_signal(GalleryLocation::default);
    let mut ready = use_signal(|| false);
    let observed = use_hook(|| Rc::new(RefCell::new(None::<GalleryLocation>)));
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let bridge = use_hook(|| {
        Rc::new(RefCell::new(
            None::<(dioxus::document::Eval, dioxus::core::Task)>,
        ))
    });

    let cleanup_bridge = bridge.clone();
    let cleanup_alive = alive.clone();
    use_drop(move || {
        cleanup_alive.set(false);
        if let Some((eval, task)) = cleanup_bridge.borrow_mut().take() {
            let _ = eval.send(serde_json::json!({"finish": true}));
            task.cancel();
        }
    });

    let subscription = bridge.clone();
    let received = observed.clone();
    use_effect(move || {
        if subscription.borrow().is_some() {
            return;
        }
        let eval = dioxus::document::eval(DESKTOP_LOCATION_BRIDGE);
        let mut receiver = eval;
        let received = received.clone();
        let alive = alive.clone();
        let task = spawn(async move {
            while let Ok(hash) = receiver.recv::<String>().await {
                if !alive.get() {
                    break;
                }
                if *ready.peek() && ordinary_fragment(&hash) {
                    // Native in-page links have already scrolled to their
                    // target. Keep the selected gallery view and restore its
                    // shareable URL without adding another history entry.
                    let current = location.peek().clone();
                    let _ = eval.send(serde_json::json!({
                        "hash": current.to_hash(), "push": false
                    }));
                    *received.borrow_mut() = Some(current);
                    continue;
                }
                let next = GalleryLocation::from_hash(&hash);
                *received.borrow_mut() = Some(next.clone());
                if *location.peek() != next {
                    location.set(next);
                }
                // The initial browser location is authoritative. Never write
                // default state before this first message has been processed.
                if !*ready.peek() {
                    ready.set(true);
                }
            }
        });
        *subscription.borrow_mut() = Some((eval, task));
    });

    use_effect(move || {
        let initialized = ready();
        let next = location();
        if !initialized {
            return;
        }
        let previous = observed.borrow().clone();
        if previous.as_ref() == Some(&next) {
            return;
        }
        let push = previous
            .as_ref()
            .is_some_and(|old| history_push(old, &next));
        if let Some((eval, _)) = bridge.borrow().as_ref()
            && eval
                .send(serde_json::json!({"hash": next.to_hash(), "push": push}))
                .is_ok()
        {
            *observed.borrow_mut() = Some(next);
        }
    });
    location
}

// Pinned desktop/window_component.rs injects MemoryHistory, which cannot read
// or change the real WebView URI. MountedData and Document expose no location
// API, so this desktop-only primary adapter supplies native URI read/write and
// notifications. Rendering, state policy and teardown ownership remain Rust.
#[cfg(feature = "desktop")]
const DESKTOP_LOCATION_BRIDGE: &str = r#"
const report=()=>dioxus.send(window.location.hash);
window.addEventListener('hashchange',report);
window.addEventListener('popstate',report);
try {
 report();
 while(true){
  const command=await dioxus.recv();
  if(command.finish)break;
  if(typeof command.hash!=='string'||command.hash===window.location.hash)continue;
  try {
   if(command.push)window.history.pushState(null,'',command.hash);
   else window.history.replaceState(null,'',command.hash);
  } catch (_) { report(); }
 }
} finally {
 window.removeEventListener('hashchange',report);
 window.removeEventListener('popstate',report);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "desktop"))]
    #[test]
    fn framework_history_restores_copied_urls_and_tracks_navigation_without_startup_writes() {
        use dioxus::history::{History, MemoryHistory};
        use std::sync::Arc;

        struct NotifyingHistory {
            inner: MemoryHistory,
            callback: RefCell<Option<Arc<dyn Fn() + Send + Sync>>>,
        }
        impl History for NotifyingHistory {
            fn current_route(&self) -> String {
                self.inner.current_route()
            }
            fn go_back(&self) {
                self.inner.go_back();
            }
            fn go_forward(&self) {
                self.inner.go_forward();
            }
            fn push(&self, route: String) {
                self.inner.push(route);
            }
            fn replace(&self, route: String) {
                self.inner.replace(route);
            }
            fn updater(&self, callback: Arc<dyn Fn() + Send + Sync>) {
                *self.callback.borrow_mut() = Some(callback);
            }
        }
        type Fixture = Rc<(
            Rc<NotifyingHistory>,
            RefCell<Option<Signal<GalleryLocation>>>,
        )>;
        fn fixture(config: Fixture) -> Element {
            use_hook(|| dioxus::history::provide_history_context(config.0.clone()));
            let location = use_gallery_location();
            *config.1.borrow_mut() = Some(location);
            rsx! { div { "{location.read().page}" } }
        }
        fn settle(dom: &mut VirtualDom) {
            use std::{
                future::Future,
                pin::pin,
                task::{Context, Poll, Waker},
            };
            let mut context = Context::from_waker(Waker::noop());
            for _ in 0..8 {
                let work = { pin!(dom.wait_for_work()).poll(&mut context) };
                match work {
                    Poll::Ready(()) => {
                        dom.render_immediate_to_vec();
                    }
                    Poll::Pending => return,
                }
            }
            panic!("Gallery history effects did not settle");
        }

        let initial = "/gallery/?keep=1#page=forms&theme=light&density=compact&q=copied";
        let history = Rc::new(NotifyingHistory {
            inner: MemoryHistory::with_initial_path(initial),
            callback: RefCell::new(None),
        });
        let config = Rc::new((history.clone(), RefCell::new(None)));
        let mut dom = VirtualDom::new_with_props(fixture, config.clone());
        dom.rebuild_in_place();
        settle(&mut dom);
        let mut location = config.1.borrow().unwrap();
        let initial_location = location.peek().clone();
        assert_eq!(initial_location.page, "forms");
        assert!(!initial_location.dark);
        assert!(initial_location.compact);
        assert_eq!(initial_location.query, "copied");
        assert_eq!(history.current_route(), initial);
        assert!(!history.inner.can_go_back());

        dom.in_scope(ScopeId::ROOT, || location.write().page = "sources".into());
        settle(&mut dom);
        assert!(history.inner.can_go_back());
        assert!(
            history
                .current_route()
                .starts_with("/gallery/?keep=1#page=sources")
        );
        dom.in_scope(ScopeId::ROOT, || location.write().dark = true);
        settle(&mut dom);
        let replaced = history.current_route();
        assert!(replaced.contains("theme=dark"));
        dom.in_scope(ScopeId::ROOT, || location.write().query = "updated".into());
        settle(&mut dom);
        let current = location.peek().clone();
        assert_eq!(current.query, "updated");

        let notify = history.callback.borrow().as_ref().unwrap().clone();
        history.go_back();
        notify();
        settle(&mut dom);
        assert_eq!(*location.peek(), initial_location);
        assert_eq!(history.current_route(), initial);
        assert!(!history.inner.can_go_back());
        history.go_forward();
        notify();
        settle(&mut dom);
        assert_eq!(*location.peek(), current);
        let example = dioxus_ix::UPSTREAM_EXAMPLE_NAMES
            .iter()
            .find(|example| **example != current.example)
            .unwrap()
            .to_string();
        dom.in_scope(ScopeId::ROOT, || location.write().example = example);
        settle(&mut dom);
        let example_location = location.peek().clone();
        history.go_back();
        notify();
        settle(&mut dom);
        assert_eq!(*location.peek(), current);
        history.go_forward();
        notify();
        settle(&mut dom);
        assert_eq!(*location.peek(), example_location);

        history.inner.push("/gallery/?keep=1#review-content".into());
        notify();
        settle(&mut dom);
        assert_eq!(*location.peek(), example_location);
        assert!(history.current_route().ends_with("#review-content"));
        drop(dom);
        // The pinned provider retains its callback. Teardown must retire access
        // to the component-owned signal even when that callback runs again.
        notify();
    }

    #[test]
    fn framework_routes_parse_only_gallery_fragments() {
        assert_eq!(
            GalleryLocation::from_hash(route_hash("/gallery/?page=forms&q=outside")),
            GalleryLocation::default()
        );
        let route = "/gallery/?q=outside#page=unknown&example=missing&q=%3Cscript%3E%00%0A";
        let location = GalleryLocation::from_hash(route_hash(route));
        assert_eq!(location.page, "actions");
        assert_eq!(location.example, "buttons");
        assert_eq!(location.query, "<script>");
        assert!(location.to_hash().ends_with("q=%3Cscript%3E"));
    }

    #[test]
    fn shareable_location_preserves_selection_theme_density_and_encoded_query() {
        let location = GalleryLocation {
            page: "sources".into(),
            dark: false,
            compact: true,
            example: "buttons".into(),
            query: "& + = # / Blåbær 🦀".into(),
        };
        let hash = location.to_hash();
        assert!(hash.starts_with("#page=sources&theme=light&density=compact&example=buttons&q="));
        assert!(hash.contains("%26%20%2B%20%3D%20%23"));
        assert_eq!(GalleryLocation::from_hash(&hash), location);
        assert_eq!(
            GalleryLocation::from_hash("#q=buttons+and+icons").query,
            "buttons and icons"
        );
    }

    #[test]
    fn invalid_links_fall_back_without_losing_valid_parameters() {
        assert_eq!(GalleryLocation::from_hash(""), GalleryLocation::default());
        let location = GalleryLocation::from_hash(
            "#page=unavailable&theme=unknown&density=wide&example=missing&q=%25%GG%",
        );
        assert_eq!(location.page, "actions");
        assert!(location.dark);
        assert!(!location.compact);
        assert_eq!(location.example, "buttons");
        assert_eq!(location.query, "%%GG%");
        assert_eq!(location.with_page("forms").page, "forms");
        assert_eq!(location.with_page("missing"), location);
        let long = format!("#q={}", "æ".repeat(MAX_QUERY_CHARS + 10));
        assert_eq!(
            GalleryLocation::from_hash(&long).query.chars().count(),
            MAX_QUERY_CHARS
        );
        assert_eq!(GalleryLocation::from_hash("#q=a%0Ab%00c").query, "abc");
    }

    #[test]
    fn in_page_links_are_distinct_from_routes_and_root_history() {
        for anchor in ["#review-content", "#section", "#g04-menu-stage"] {
            assert!(ordinary_fragment(anchor));
        }
        for route in ["", "#", "#page=forms", "#page=missing", "#q=button"] {
            assert!(!ordinary_fragment(route));
        }
    }
}
