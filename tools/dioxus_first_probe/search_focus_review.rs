//! Excluded actual-gallery ExpandingSearch workflow for the desktop renderer.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/forms.rs"]
mod gallery_forms;

#[component]
pub fn SearchFocusReview() -> Element {
    let mut epoch = use_signal(|| 0_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {gallery_forms::STYLES} {STYLES} }
        IxTheme { class: "search-focus-review",
            h1 { "Search and upload choices" }
            p { "The gallery search keeps its native input while it opens, clears and closes." }
            Button { id: "search-focus-remount", variant: ButtonVariant::Secondary,
                onclick: move |_| epoch += 1, "Restart search example"
            }
            for generation in [epoch()] {
                div { key: "search-example-{generation}", id: "search-focus-example", "data-epoch": generation.to_string(),
                    gallery_forms::FormExamples {}
                }
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_SEARCH_FOCUS_CHECK);
                        if let Err(error) = query.send("searchfocus") {
                            eprintln!("Search focus native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native search focus: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native search focus failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Search focus native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

// Only the excluded preview hides unrelated examples; the gallery and all
// component styles remain the same bundled sources used by consuming apps.
const STYLES: &str = "body{margin:0}.search-focus-review{box-sizing:border-box;min-height:100vh;padding:24px}.search-focus-review>h1{font-size:28px;line-height:40px;margin:0 0 16px}.search-focus-review>p{max-width:70ch;margin:0 0 24px}.search-focus-review .demo-section{margin-top:32px}.search-focus-review .demo-section:not(#search-upload){display:none}.search-focus-review .demo-section>h2{font-size:22px;line-height:32px;margin:0 0 16px}.search-focus-review .demo-section>h3{font-size:18px;line-height:28px;margin:32px 0 12px}.search-focus-review .demo-section>p{max-width:70ch}.search-focus-review .demo-row{display:flex;flex-wrap:wrap;gap:12px}@media(max-width:600px){.search-focus-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SEARCH_FOCUS_CHECK: &str = r#"
let snapshots={},checks=[];
try {
 await dioxus.recv();
 const section=()=>document.getElementById('search-upload'),root=()=>section()?.querySelector('.ix-expanding-search');
 const input=()=>root()?.querySelector('input'),opener=()=>root()?.querySelector('.ix-search-trigger'),clear=()=>root()?.querySelector('.ix-search-clear');
 const folder=()=>[...section().querySelectorAll('button[role=checkbox]')].find(node=>node.textContent.includes('Choose folders'));
 const folders=()=>folder()?.getAttribute('aria-checked')==='true';
 const status=()=>section()?.querySelector('[role=status]')?.textContent.trim(),active=()=>document.activeElement;
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<160;n++){if(predicate())return;await hold(20)}throw Error(label+'; status='+status()+'; focus='+(active()?.id||active()?.tagName))};
 const frame=()=>new Promise(resolve=>requestAnimationFrame(resolve));
 const settle=async()=>{await frame();await Promise.all((root()?.getAnimations({subtree:true})||[]).map(animation=>animation.finished.catch(()=>{})));await frame();await frame()};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus({preventScroll:true});await wait(label+' receives native focus',()=>active()===node)};
 const capture=()=>({expanded:root()?.dataset.expanded,changed:root()?.dataset.changed,value:input()?.value,folder:folders(),focus:active()?.getAttribute('aria-label')||active()?.id||active()?.tagName,status:status()});
 const type=async(value)=>{input().value=value;input().dispatchEvent(new Event('input',{bubbles:true}));await wait('actual controlled value '+value,()=>input().value===value&&status().startsWith('Search text: '+value+'.'))};
 const open=async()=>{await focus(opener(),'actual gallery search trigger');opener().click();await wait('actual retained input opens and receives mounted focus',()=>root()?.dataset.expanded==='true'&&active()===input())};
 await wait('actual gallery search and folder controls',()=>section()&&root()&&input()&&folder());
 await wait('native document focus',()=>document.hasFocus());
 const retained=input();assert(root().dataset.expanded==='false'&&input().tabIndex===-1,'initial native input is retained outside tab order');
 await open();assert(input()===retained&&input().tabIndex===0,'opening retains the native input and enables its tab stop');
 await type('Motor coupling');
 const enter=new KeyboardEvent('keydown',{key:'Enter',code:'Enter',bubbles:true,cancelable:true});input().dispatchEvent(enter);
 await wait('actual FormExamples receives exact Enter search',()=>status()==='Search text: Motor coupling. Submitted search: Motor coupling.');
 assert(enter.defaultPrevented,'synthetic Enter handler prevents default');snapshots.submitted=capture();
 checks.push('actual FormExamples open uses retained native input; exact controlled value and synthetic Enter submission');

 await focus(clear(),'actual Clear search');clear().click();
 await wait('Clear empties controlled value and returns native focus',()=>!clear()&&input().value===''&&active()===input()&&root().dataset.changed==='false');
 assert(status()==='Search text: . Submitted search: Motor coupling.','Clear preserves submitted search');snapshots.cleared=capture();
 await focus(folder(),'adjacent Choose folders');folder().click();
 await wait('unchanged blur collapses while folder action works',()=>root().dataset.expanded==='false'&&folders());
 await settle();assert(active()===folder()&&input()===retained,'settled collapsed field preserves independently focused folder control');snapshots.emptyBlur=capture();
 checks.push('Clear empties and refocuses; unchanged blur collapses without stealing adjacent folder focus or checkbox state');

 await open();await type('Inspect guards');await focus(folder(),'changed-search adjacent folder');folder().click();
 await wait('changed blur retains search while folder action works',()=>root().dataset.expanded==='true'&&root().dataset.changed==='true'&&!folders());
 await settle();assert(active()===folder()&&input().value==='Inspect guards','settled changed field preserves independent focus and search value');snapshots.changedBlur=capture();
 checks.push('changed blur retains typed search; adjacent native checkbox action and focus remain usable after visual transitions settle');

 const restart=document.getElementById('search-focus-remount');await focus(restart,'restart actual example');restart.click();
 await wait('keyed actual example remounts cleanly',()=>document.getElementById('search-focus-example').dataset.epoch==='1'&&input()!==retained&&root().dataset.expanded==='false'&&input().value==='');
 await settle();assert(!retained.isConnected&&active()===restart&&!folders(),'retired field stays disconnected while independent restart focus remains');
 await open();await type('Replacement motor');assert(status()==='Search text: Replacement motor. Submitted search: .','remounted controlled search is independently usable');snapshots.remounted=capture();
 checks.push('keyed actual gallery remount retires old input, resets owned state and provides working fresh native focus');
 return {ok:true,checks,snapshots,enter:{synthetic:true,defaultPrevented:enter.defaultPrevented},renderer:navigator.userAgent,input:'Excluded WebKitGTK native DOM focus/click/input replay; Enter is an untrusted synthetic keydown exercising the Dioxus handler. Lead Chromium provides trusted pointer and keyboard evidence; disabled and stale-event guards are covered by focused library tests.',scope:'Actual examples/gallery/forms.rs FormExamples and bundled styles; unrelated sections hidden only in excluded preview. No exact timing acceptance.'};
} catch(error){return {ok:false,error:String(error),snapshots,checks,focus:document.activeElement?.outerHTML,search:document.querySelector('#search-upload .ix-expanding-search')?.outerHTML}}
"#;
