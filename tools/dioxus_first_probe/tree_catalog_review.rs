//! Excluded actual-gallery tree catalog recovery and bounded native replay.
use super::tree_actions_review::gallery_tree;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn TreeCatalogPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_tree::STYLES} }
        IxTheme { class:"tree-catalog-review",
            gallery_tree::MutableTreeExample {}
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_TREE_CATALOG_CHECK);let _=query.send("treecatalog");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native Tree catalog: {value}");window.close();}
                        value=>{eprintln!("Dioxus native Tree catalog failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Tree catalog native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.tree-catalog-review{box-sizing:border-box;min-height:100vh;padding:24px}.tree-catalog-review h2{font-size:24px;line-height:32px;margin:0 0 16px}.tree-catalog-review p{max-width:72ch;line-height:1.5}@media(max-width:600px){.tree-catalog-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TREE_CATALOG_CHECK: &str = r#"
let snapshots={};
try {
 await dioxus.recv();
 const host=()=>document.getElementById('tree-model'),get=id=>document.getElementById('tree-catalog-'+id),status=()=>get('status'),selection=()=>host()?.querySelector('.tree-model-status')?.dataset.selected,row=id=>host()?.querySelector('[data-tree-node-id="'+id+'"]'),active=()=>document.activeElement;
 const assert=(condition,label)=>{if(!condition)throw Error(label)},wait=async(label,predicate)=>{for(let i=0;i<180;i++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; status='+status()?.outerHTML+'; selected='+selection()+'; focus='+active()?.dataset.treeNodeId)};
 const button=label=>[...host().querySelectorAll('button')].find(n=>n.textContent.trim()===label),click=label=>{assert(button(label),label+' exists');button(label).click()},key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true}));
 const fill=async(node,value)=>{const prototype=node instanceof HTMLTextAreaElement?HTMLTextAreaElement.prototype:HTMLInputElement.prototype;Object.getOwnPropertyDescriptor(prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))};
 const load=async(text,state)=>{await fill(get('input'),text);get('load').click();await wait('actual catalog '+state,()=>status().dataset.state===state)};
 const visible=()=>[...host().querySelectorAll('[data-tree-node-id]')].map(n=>({id:n.dataset.treeNodeId,text:n.textContent,expanded:n.getAttribute('aria-expanded'),disabled:n.getAttribute('aria-disabled')}));
 const capture=()=>({...status()?.dataset,selected:selection(),rows:visible(),catalog:get('input').value});
 await wait('actual mutable tree/catalog mounted',()=>row('model-plant')&&row('model-remote')&&get('input')&&get('load'));
 const originalCatalog=get('input').value;row('model-remote').focus();key(row('model-remote'),'ArrowRight');await wait('default actual expansion loads original two remote devices',()=>row('model-remote-1')&&row('model-remote-2')&&status().dataset.state==='loaded'&&status().dataset.count==='2');snapshots.default=capture();
 row('model-device-1').click();await wait('local model device selected',()=>selection()==='model-device-1');await fill(host().querySelector('input'),'Retained local motor');click('Rename selected');await wait('actual local rename committed',()=>row('model-device-1').textContent.includes('Retained local motor'));const before=JSON.stringify(visible());await load('{','error');assert(get('error')?.textContent&&JSON.stringify(visible())===before&&selection()==='model-device-1','malformed catalog retains actual model rows, selection and unrelated rename');snapshots.invalid=capture();
 const nested=JSON.stringify([{id:'catalog-rack',label:'Remote rack',children:[{id:'catalog-drive',label:'Remote device 101'}]},{id:'catalog-disabled',label:'Unavailable remote',disabled:true}]);await load(nested,'loaded');await wait('nested catalog replaces old remote branch atomically',()=>row('catalog-rack')&&row('catalog-disabled')&&!row('model-remote-1')&&!row('model-remote-2'));assert(status().dataset.count==='3'&&row('catalog-disabled').getAttribute('aria-disabled')==='true'&&selection()==='model-device-1'&&row('model-device-1').textContent.includes('Retained local motor'),'nested disabled descendants retain unrelated selected local data');row('catalog-rack').focus();key(row('catalog-rack'),'ArrowRight');await wait('ordinary keyboard expands nested catalog',()=>row('catalog-drive'));key(row('catalog-rack'),'ArrowRight');await wait('ordinary keyboard enters child',()=>active()===row('catalog-drive'));key(active(),'ArrowLeft');await wait('ordinary child Left returns to parent',()=>active()===row('catalog-rack'));snapshots.recovered=capture();
 await load('[]','loaded');await wait('empty catalog makes known remote leaf',()=>status().dataset.count==='0'&&!row('catalog-rack')&&!row('catalog-drive')&&!row('catalog-disabled')&&row('model-remote')?.getAttribute('aria-expanded')===null);assert(!get('load').disabled,'known empty branch can be explicitly reloaded');await load(nested,'loaded');await wait('expanded/known leaf explicit reload restores actual catalog',()=>status().dataset.count==='3');snapshots.emptyAndReload=capture();
 row('model-remote').click();await wait('remote branch selected for removal',()=>selection()==='model-remote');click('Remove selected');await wait('actual branch and descendants removed',()=>!row('model-remote')&&status().dataset.remoteExists==='false'&&get('load').disabled);get('load').click();assert(!row('model-remote'),'disabled reload cannot resurrect removed parent');const retained=get('input').value;click('Restore inventory');await wait('inventory restored without replacing catalog preparation',()=>row('model-remote')&&status().dataset.state==='idle'&&get('input').value===retained);assert(row('model-device-1').textContent.includes('Motor 001')&&get('input').value===nested,'Restore resets inventory while retaining real catalog draft');get('load').click();await wait('restored current remote parent accepts retained catalog',()=>status().dataset.state==='loaded'&&status().dataset.count==='3');assert(document.documentElement.scrollWidth<=innerWidth+1,'actual native catalog/editor/tree fit viewport');snapshots.removeRestore=capture();
 return{ok:true,snapshots,checks:['ordinary expansion uses actual default JSON catalog and preserves its two original devices','malformed catalog exposes useful error without mutating current model/selection/unrelated rename','explicit retry replaces actual remote subtree with nested disabled nodes; ordinary keyboard enters/returns current children','empty catalog creates known leaf; explicit load works again without timers or delayed resurrection','parent removal disables load; Restore preserves catalog preparation and current branch can load again'],input:'Excluded synthetic DOM input/click/key dispatch exercises actual gallery Rust parse/replacement/model ownership and mounted focus. No model/controller/application callback injection or remote server claim; trusted interaction is separate Chromium evidence.',renderer:navigator.userAgent};
} catch(error){return{ok:false,error:String(error),snapshots,status:document.getElementById('tree-catalog-status')?.outerHTML,tree:document.getElementById('tree-model')?.textContent.slice(-1000)}}

"#;
