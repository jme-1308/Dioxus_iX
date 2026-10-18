//! Small shared web/native preview. Development tooling, excluded from the crate.
use dioxus::prelude::*;
use dioxus_ix::*;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/progress_review.rs"]
mod progress_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Progress indicator preview")
                .with_visible(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(1000.0, 900.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Preview);
}
#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(Preview);
}
#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
fn main() {
    let _preview = Preview;
    eprintln!("Use web or desktop features for the progress preview");
}

#[component]
fn Preview() -> Element {
    let mut dark = use_signal(|| false);
    let mut compact = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
            eprintln!("Progress native preview timed out");
            std::process::exit(1);
        });
    });
    rsx! {
        style { {"body{margin:0} .progress-preview{padding:24px;box-sizing:border-box;min-height:100vh} @media(max-width:480px){.progress-preview{padding:16px}}"} }
        IxTheme {
            scheme: if dark() {ColorScheme::Dark} else {ColorScheme::Light},
            density: if compact() {Density::Compact} else {Density::Normal},
            class: "progress-preview",
            div {
                onmounted: move |_| {
                    #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                    {let window=window.clone();spawn(async move {
                        let query = dom_query::eval(NATIVE_READ);
                        let _ = query.send(19);
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"]==true => {println!("Progress WebView: {value}");window.close();}
                            result => {eprintln!("Progress WebView failed: {result:?}");std::process::exit(1);}
                        }
                    });}
                },
                Button { variant: ButtonVariant::Secondary, onclick: move |_| dark.set(!dark()),
                    if dark() {"Use light theme"} else {"Use dark theme"}
                }
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| compact.set(!compact()),
                    if compact() {"Use normal density"} else {"Use compact density"}
                }
            }
            progress_review::ProgressReview {}
        }
    }
}

// Native DOM observation only; application state and interactions stay in Rust.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_READ: &str = r#"
const expected = await dioxus.recv();
const root=document.querySelector('#progress-review');
const bars=[...root.querySelectorAll('[role=progressbar]')];
const demo=[...root.querySelectorAll('.progress-demo [role=progressbar]')];
const ring=root.querySelector('.progress-demo .ix-progress-circle');
const complete=root.querySelector('.ix-progress-status-success .ix-progress-circle-fill');
const rect=ring.getBoundingClientRect();
const trackWidth=root.querySelector('.progress-demo .ix-progress-track').getBoundingClientRect().width;
const result={count:bars.length,values:demo.map(n=>n.getAttribute('aria-valuenow')),
 ring:[rect.width,rect.height],trackWidth,completeOffset:complete.getAttribute('stroke-dashoffset'),
 viewport:document.documentElement.clientWidth,scroll:document.documentElement.scrollWidth,
 fits:document.documentElement.scrollWidth<=document.documentElement.clientWidth};
result.ok=result.count===expected&&result.values.every(v=>v==='35')&&rect.width===64&&rect.height===64&&trackWidth>100&&result.completeOffset==='0'&&result.fits;
return result;
"#;
