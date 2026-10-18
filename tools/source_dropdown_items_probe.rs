//! Excluded pinned-source mixed-item diagnostic; no source policy emulation.
use dioxus::prelude::*;
use std::time::Duration;
#[path = "../src/dom_query.rs"]
mod dom_query;

fn main() {
    let index = include_str!("upstream_dropdown_items_review.html").replace(
        "</body>",
        "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
    );
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Pinned source Dropdown mixed items")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0))
                .with_visible(true),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const VERIFY: &str = r#"
await dioxus.recv();
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const until=performance.now()+25000;
while(performance.now()<until&&!window.upstreamDropdownItemsReview?.snapshot()?.ready)await sleep(5);
if(!window.upstreamDropdownItemsReview?.snapshot()?.ready)return{ok:false,error:'Mixed-item source bootstrap timeout',status:document.getElementById('items-status')?.textContent};
document.getElementById('items-run').click();
const end=performance.now()+65000;
while(performance.now()<end&&document.getElementById('items-status')?.dataset.run!=='done')await sleep(5);
if(document.getElementById('items-status')?.dataset.run!=='done')return{ok:false,error:'Mixed-item source histories timeout',status:document.getElementById('items-status')?.textContent};
return JSON.parse(document.getElementById('items-result').textContent);
"#;

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(90)).await;
            eprintln!("Pinned source mixed-items global timeout");
            std::process::exit(1);
        });
    });
    rsx! { div { onmounted: move |_| {
        let window = window.clone();
        spawn(async move {
            let query = dom_query::eval(VERIFY);
            let _ = query.send(());
            match query.join::<serde_json::Value>().await {
                Ok(result) if result["ok"] == true => { println!("Pinned source Dropdown mixed items checks: {result}"); window.close(); }
                Ok(result) => { eprintln!("Pinned source Dropdown mixed items failure: {result}"); std::process::exit(1); }
                Err(error) => { eprintln!("Pinned source Dropdown mixed items eval failure: {error:?}"); std::process::exit(1); }
            }
        });
    } } }
}
