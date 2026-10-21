//! Excluded preview and bounded Linux driver of the actual gallery row actions.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/tree.rs"]
// The gallery module also contains independent examples outside this preview.
#[allow(dead_code)]
pub(super) mod gallery_tree;

#[component]
pub fn TreeActionsPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_tree::STYLES} }
        IxTheme { class: "tree-actions-review",
            h1 { "Tree row actions" }
            p { "Inspect a drive without changing the selected row. Focus a row to navigate with Arrow keys; focus its Inspect button to use that action." }
            gallery_tree::TreeRowActionsExample {}
            label { r#for: "tree-actions-outside", "Outside focus target" }
            input { id: "tree-actions-outside", value: "Station monitor" }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_TREE_ACTIONS_CHECK);
                        if let Err(error) = query.send("treeactions") {
                            eprintln!("Tree row actions native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native Tree row actions: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native Tree row actions failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Dioxus native Tree row actions timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.tree-actions-review{box-sizing:border-box;min-height:100vh;padding:24px}.tree-actions-review h1{font-size:28px;margin:0 0 16px}.tree-actions-review h2{font-size:20px;margin:24px 0 12px}.tree-actions-review p{max-width:68ch;line-height:1.5}.tree-actions-review output{display:block;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}.tree-actions-review label{display:block;margin:24px 0 8px}.tree-actions-review>input{box-sizing:border-box;max-width:100%;width:320px;min-height:32px;padding:6px 8px;color:var(--ix-color-text);background:var(--ix-color-surface);border:1px solid var(--ix-color-border);font:inherit}@media(max-width:600px){.tree-actions-review{padding:16px}}";

// Synthetic keys do not perform trusted native button activation. Explicit clicks
// after Space/Enter test action cancellation separately; the lead verifies real
// keyboard defaults in the browser. Focus assertions observe mounted elements
// after bounded settling, without requiring immediate physical event chronology.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TREE_ACTIONS_CHECK: &str = r#"
let inputFocusFacts={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const section=()=>get('tree-row-actions'),status=()=>get('tree-row-action-status');
 const row=id=>section()?.querySelector('[data-tree-node-id="'+id+'"]');
 const inspect=id=>get('tree-row-inspect-'+id);
 const selected=()=>status()?.dataset.selected,count=()=>Number(status()?.dataset.inspections);
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await new Promise(resolve=>setTimeout(resolve,20));
  }
  throw Error(label+'; focus='+active()?.id+'/'+active()?.dataset.treeNodeId+'; status='+status()?.textContent);
 };
 const key=(node,value,type='keydown')=>{
  const event=new KeyboardEvent(type,{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});
  node.dispatchEvent(event);return event;
 };
 const unchanged=(button,inspections)=>active()===button&&selected()==='actions-drive-1'&&count()===inspections&&row('actions-plant')?.getAttribute('aria-expanded')==='true';
 await wait('gallery fixture mounted with initial selection',()=>row('actions-drive-1')&&row('actions-drive-2')&&inspect('actions-drive-2')&&selected()==='actions-drive-1'&&count()===0);
 await wait('WebView document ready for keyboard/focus input',()=>document.hasFocus());
 const button=inspect('actions-drive-2');
 inputFocusFacts={documentFocused:document.hasFocus(),active:active()?.id,node:active()?.dataset.treeNodeId};
 button.focus();
 await wait('embedded Inspect receives focus without changing selection',()=>unchanged(button,0));
 button.click();
 await wait('Inspect activates once and preserves selection and button focus',()=>unchanged(button,1)&&status().dataset.last==='actions-drive-2');
 assert(!key(button,'ArrowDown').defaultPrevented,'embedded ArrowDown must retain its default');
 await wait('embedded ArrowDown does not navigate/select/expand the tree',()=>unchanged(button,1));

 for(const [value,nextCount] of [[' ',2],['Enter',3]]){
  assert(!key(button,value).defaultPrevented,'embedded '+JSON.stringify(value)+' default canceled');
  assert(!key(button,value,'keyup').defaultPrevented,'embedded keyup default canceled');
  await wait('synthetic control key leaves row and action state unchanged',()=>unchanged(button,nextCount-1));
  button.click();
  await wait('explicit control click after synthetic key activates once',()=>unchanged(button,nextCount)&&status().dataset.last==='actions-drive-2');
 }

 row('actions-drive-2').click();
 await wait('ordinary row click selects and focuses Drive002',()=>selected()==='actions-drive-2'&&active()===row('actions-drive-2')&&count()===3);
 const plant=()=>row('actions-plant');
 plant().focus();
 await wait('ordinary branch row focused',()=>active()===plant());
 key(plant(),'ArrowLeft');
 await wait('ordinary ArrowLeft collapses branch',()=>plant()?.getAttribute('aria-expanded')==='false'&&!row('actions-drive-1')&&active()===plant());
 key(plant(),'ArrowRight');
 await wait('ordinary ArrowRight expands branch',()=>plant()?.getAttribute('aria-expanded')==='true'&&row('actions-drive-1')&&active()===plant());
 key(plant(),'ArrowRight');
 await wait('ordinary ArrowRight enters first child',()=>active()===row('actions-drive-1'));
 key(active(),'ArrowDown');
 await wait('ordinary ArrowDown navigates to next child',()=>active()===row('actions-drive-2'));
 key(active(),'ArrowLeft');
 await wait('ordinary child ArrowLeft returns to parent',()=>active()===plant());

 const disabled=inspect('actions-unavailable'),outside=get('tree-actions-outside');
 assert(disabled?.disabled===true,'unavailable row Inspect is a native disabled button');
 assert(row('actions-unavailable')?.getAttribute('aria-disabled')==='true','unavailable row is disabled');
 outside.focus();
 await wait('outside input focus established',()=>active()===outside);
 disabled.click();
 await wait('disabled Inspect does not activate or steal outside focus',()=>active()===outside&&count()===3&&selected()==='actions-drive-2');
 return {
  ok:true,
  checks:['initial Drive001 selection','Inspect Drive002 once without row selection','mounted button focus retained','control ArrowDown default untouched','synthetic Space/Enter boundaries plus explicit clicks','ordinary row click selection','branch Left/Right and child navigation','disabled Inspect','outside focus retained'],
  inputFocusFacts,inspections:count(),selected:selected(),last:status().dataset.last,
  input:'synthetic DOM keys/clicks with real mounted focus',
  trustedButtonActivation:'separate browser Space/Enter check',
  renderer:navigator.userAgent
 };
} catch(error) {return {ok:false,error:String(error),inputFocusFacts,documentFocusedNow:document.hasFocus()}}
"#;
