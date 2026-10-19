//! Bounded Linux WebView check of the actual gallery location hook.
//!
//! JavaScript is an excluded development driver: it activates Rust controls,
//! observes the real URI, and calls native Back/Forward. Navigation policy and
//! state stay in the imported gallery module. Input events are synthetic.
use dioxus::prelude::*;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/navigation.rs"]
mod navigation;

const UNICODE_QUERY: &str = "& + = # / Blåbær 🦀";

fn main() {
    // This must run before the hook mounts: it protects restoration of the
    // native startup URI against an early write of GalleryLocation::default().
    let index = r##"<!doctype html><html><head><meta charset="utf-8"></head><body><script>history.replaceState(null,'','#page=sources&theme=light&density=compact&example=buttons&q=Button');</script><div id="main"></div><!-- MODULE LOADER --></body></html>"##;
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index.to_string())
        .with_window(dioxus::desktop::WindowBuilder::new().with_visible(false))
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

#[component]
fn Probe() -> Element {
    let mut location = navigation::use_gallery_location();
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(25)).await;
            eprintln!("Gallery location WebView timed out");
            std::process::exit(1);
        });
    });

    let current = location();
    rsx! {
        main {
            onmounted: move |_| {
                let window = window.clone();
                spawn(async move {
                    let initial = navigation::GalleryLocation {
                        page: "sources".into(),
                        dark: false,
                        compact: true,
                        example: "buttons".into(),
                        query: "Button".into(),
                    };
                    let page = initial.with_page("coverage");
                    let mut theme = page.clone();
                    theme.dark = true;
                    let mut density = theme.clone();
                    density.compact = false;
                    let mut query_state = density.clone();
                    query_state.query = UNICODE_QUERY.into();
                    let mut example = query_state.clone();
                    example.example = "button-group".into();
                    let query = dom_query::eval(VERIFY);
                    if let Err(error) = query.send(serde_json::json!({
                        "initial": initial.to_hash(),
                        "page": page.to_hash(),
                        "theme": theme.to_hash(),
                        "density": density.to_hash(),
                        "query": query_state.to_hash(),
                        "example": example.to_hash(),
                        "unicode": UNICODE_QUERY,
                    })) {
                        eprintln!("Gallery native driver failed to start: {error:?}");
                        std::process::exit(1);
                    }
                    match query.join::<serde_json::Value>().await {
                        Ok(result) if result["ok"] == true => {
                            println!("Gallery WebView location checks: {result}");
                            window.close();
                        }
                        result => {
                            eprintln!("Gallery native location failed: {result:?}");
                            std::process::exit(1);
                        }
                    }
                });
            },
            output {
                id: "gallery-location-state",
                "data-hash": current.to_hash(),
                "data-theme": if current.dark { "dark" } else { "light" },
                "data-density": if current.compact { "compact" } else { "normal" },
                "data-query": current.query.clone(),
                "Gallery location: {current.page} / {current.example}"
            }
            button {
                id: "gallery-location-page", r#type: "button",
                onclick: move |_| {
                    let next = location().with_page("coverage");
                    location.set(next);
                },
                "Coverage"
            }
            button {
                id: "gallery-location-theme", r#type: "button",
                onclick: move |_| location.write().dark = true,
                "Dark theme"
            }
            button {
                id: "gallery-location-density", r#type: "button",
                onclick: move |_| location.write().compact = false,
                "Normal density"
            }
            label {
                r#for: "gallery-location-query", "Search",
            }
            input {
                id: "gallery-location-query",
                value: current.query,
                oninput: move |event| location.write().query = event.value(),
            }
            button {
                id: "gallery-location-example", r#type: "button",
                onclick: move |_| location.write().example = "button-group".into(),
                "Button group example"
            }
        }
    }
}

const VERIFY: &str = r#"
try {
 const expected=await dioxus.recv();
 const get=id=>document.getElementById(id);
 const state=()=>get('gallery-location-state');
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(test,label)=>{
  const deadline=performance.now()+3000;
  while(!test()){
   if(performance.now()>deadline)throw Error(label+'; URI='+window.location.hash+'; Rust='+state()?.dataset.hash);
   await new Promise(resolve=>setTimeout(resolve,20));
  }
 };
 const settled=(hash,label)=>wait(()=>window.location.hash===hash&&state()?.dataset.hash===hash,label);
 const click=async(id,hash,label)=>{
  assert(get(id),'missing control '+id);
  get(id).click();
  await settled(hash,label);
 };
 await settled(expected.initial,'startup URI restores Rust selection and settings');
 assert(state().dataset.theme==='light'&&state().dataset.density==='compact','initial light/compact settings');
 assert(state().dataset.query==='Button','initial search query');
 const initialLength=history.length;

 await click('gallery-location-page',expected.page,'page push reaches native URI');
 assert(history.length===initialLength+1,'page selection adds one history entry');
 await click('gallery-location-theme',expected.theme,'theme replace reaches native URI');
 assert(history.length===initialLength+1,'theme replaces current entry');
 await click('gallery-location-density',expected.density,'density replace reaches native URI');
 assert(history.length===initialLength+1,'density replaces current entry');
 const input=get('gallery-location-query');
 input.value=expected.unicode;
 input.dispatchEvent(new Event('input',{bubbles:true}));
 await settled(expected.query,'Unicode query replace reaches native URI');
 assert(history.length===initialLength+1,'query replaces current entry');
 assert(state().dataset.query===expected.unicode,'Rust preserves Unicode and reserved query characters');
 assert(new URLSearchParams(window.location.hash.slice(1)).get('q')===expected.unicode,'native URI query codec round trip');
 await click('gallery-location-example',expected.example,'example push reaches native URI');
 assert(history.length===initialLength+2,'example selection adds one history entry');

 history.back();
 await settled(expected.query,'native Back restores prior example with replaced settings/query');
 history.back();
 await settled(expected.initial,'native Back restores original page and startup settings/query');
 assert(state().dataset.theme==='light'&&state().dataset.density==='compact','Back restores initial light/compact settings');
 history.forward();
 await settled(expected.query,'native Forward restores page with replaced settings/query');
 history.forward();
 await settled(expected.example,'native Forward restores selected example');
 assert(history.length===initialLength+2,'traversal does not create extra entries');
 return {
  ok:true,
  checks:['startup selection/light/compact/query','page and example push','theme/density/query replace','Unicode/reserved query codec','native Back twice and Forward twice update Rust state'],
  addedEntries:history.length-initialLength,
  finalHash:window.location.hash,
  input:'synthetic controls and real native history traversal',
  renderer:navigator.userAgent
 };
} catch(error) {return {ok:false,error:String(error)}}
"#;
