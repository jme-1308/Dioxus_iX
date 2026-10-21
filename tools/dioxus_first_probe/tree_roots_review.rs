//! Excluded actual-gallery Tree roots, mounted scroll and ordinary model workflows.
use super::tree_actions_review::gallery_tree;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn TreeRootsPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { "body{{margin:0}}.tree-roots-review{{box-sizing:border-box;min-height:100vh;padding:24px}}" }
        style { {gallery_tree::STYLES} }
        IxTheme { class: "tree-roots-review",
            gallery_tree::TreeExamples {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_TREE_ROOTS_CHECK);
                        if let Err(error) = query.send("treeroots") {
                            eprintln!("Tree roots native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native Tree roots: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native Tree roots failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(45)).await;
                        eprintln!("Tree roots native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TREE_ROOTS_CHECK: &str = r#"
let snapshots={},checks=[];
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),inventory=()=>get('inventory-tree'),model=()=>get('mutable-inventory-tree'),section=()=>get('tree-virtualized'),modelHost=()=>get('tree-model'),top=()=>get('inventory-tree-top'),row=(host,id)=>host?.querySelector('[data-tree-node-id="'+id+'"]'),selected=()=>section()?.querySelector('[role=status]')?.textContent,modelSelection=()=>get('tree-model-status')?.dataset.selected;
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<190;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; selected='+selected()+'; scroll='+inventory()?.scrollTop+'; focus='+document.activeElement?.dataset.treeNodeId)};
 const focus=async node=>{assert(node,'actual current native control exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait('actual current control focused',()=>document.activeElement===node)},click=async node=>{await focus(node);node.click()},key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true})),button=(host,label)=>[...host.querySelectorAll('button')].find(n=>n.textContent.trim()===label);
 const fill=async(node,value)=>{const proto=node instanceof HTMLTextAreaElement?HTMLTextAreaElement.prototype:HTMLInputElement.prototype;Object.getOwnPropertyDescriptor(proto,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual controlled draft accepted',()=>node.value===value)};
 const snapshot=()=>({scrollTop:inventory().scrollTop,visible:inventory().dataset.visibleCount,selected:selected(),mounted:[...inventory().querySelectorAll('[data-tree-node-id]')].map(n=>({id:n.dataset.treeNodeId,expanded:n.getAttribute('aria-expanded'),selected:n.getAttribute('aria-selected'),tabindex:n.tabIndex})),focus:{id:document.activeElement?.id,row:document.activeElement?.dataset.treeNodeId}});
 await wait('actual native tree roots mounted',()=>inventory()&&model()&&top()&&!top().disabled&&row(inventory(),'inventory-line-1'));
 const oldInventory=inventory(),oldModel=model();assert(inventory().matches('ul.ix-tree[role=tree]')&&inventory().getAttribute('aria-label')==='Large device tree'&&model().matches('ul.ix-tree[role=tree]')&&model().getAttribute('aria-label')==='Mutable inventory tree','public ids name actual native tree roots');assert(inventory().dataset.visibleCount==='2020'&&inventory().querySelectorAll('[data-tree-node-id]').length<100,'expanded2000-device tree retains bounded native virtualization');snapshots.initial=snapshot();checks.push('actual Tree/ModelTree roots carry public IDs while expanded2020 logical rows remain virtualized');
 await focus(row(inventory(),'inventory-line-1'));key(document.activeElement,'End');await wait('actual End reveals and focuses last logical device',()=>document.activeElement===row(inventory(),'inventory-20-100')&&inventory().scrollTop>0);key(document.activeElement,'Enter');await wait('actual last device selected',()=>selected().includes('inventory-20-100')&&row(inventory(),'inventory-20-100')?.getAttribute('aria-selected')==='true');snapshots.end=snapshot();await click(top());await wait('mounted Back to top scrolls current root without changing selection/focus/hierarchy',()=>inventory().scrollTop<=1&&row(inventory(),'inventory-line-1')&&row(inventory(),'inventory-1-1')&&row(inventory(),'inventory-line-1').getAttribute('aria-expanded')==='true'&&inventory().dataset.visibleCount==='2020'&&selected().includes('inventory-20-100')&&document.activeElement===top());snapshots.top=snapshot();checks.push('End/Enter selects last actual device; mounted Back to top restores first rows while retaining last selection/full expansion/button focus');
 const entry=inventory().querySelector('[data-tree-node-id][tabindex="0"]');assert(entry===row(inventory(),'inventory-line-1'),'top mounted roving entry is current first branch');await focus(entry);key(entry,'ArrowRight');await wait('ordinary keyboard reenters first current child',()=>document.activeElement===row(inventory(),'inventory-1-1'));key(document.activeElement,'Enter');await wait('actual first child selection committed',()=>selected().includes('inventory-1-1'));await click(button(section(),'Collapse inventory'));await wait('normal collapse leaves20 branches',()=>inventory().dataset.visibleCount==='20');await click(button(section(),'Expand inventory'));await wait('normal expand restores2020 rows and retained selected child',()=>inventory().dataset.visibleCount==='2020'&&selected().includes('inventory-1-1'));await click(top());await wait('current mounted root can scroll again',()=>inventory().scrollTop<=1&&row(inventory(),'inventory-line-1'));assert(inventory()===oldInventory,'same native root retained across ordinary expansion/selection');snapshots.reentry=snapshot();checks.push('explicit native top-row focus plus handled Arrow/Enter reenters normally; collapse/expand preserves selection and current root');
 await click(row(model(),'model-device-1'));await wait('actual model device selected',()=>modelSelection()==='model-device-1');const name=modelHost().querySelector('input');await fill(name,'Mounted root motor');await click(button(modelHost(),'Rename selected'));await wait('actual model rename committed',()=>row(model(),'model-device-1').textContent.includes('Mounted root motor'));await focus(row(model(),'model-remote'));key(document.activeElement,'ArrowRight');await wait('actual existing catalog loads default two devices',()=>get('tree-catalog-status').dataset.state==='loaded'&&get('tree-catalog-status').dataset.count==='2'&&row(model(),'model-remote-1')&&row(model(),'model-remote-2'));assert(model()===oldModel&&modelSelection()==='model-device-1'&&row(model(),'model-device-1').textContent.includes('Mounted root motor'),'ModelTree root/selection/rename survive normal catalog load');snapshots.model={selected:modelSelection(),catalog:{...get('tree-catalog-status').dataset},root:model().id};checks.push('actual mutable model root retains ordinary selection/rename/catalog operations and current root identity');
 return{ok:true,checks,snapshots,viewport:{width:innerWidth,height:innerHeight},input:'Excluded synthetic DOM clicks/input/handled keys and explicit native focus exercise actual unchanged TreeExamples. Back to top invokes actual Rust mounted scroll; native reentry explicitly focuses current roving row because synthetic Tab does not execute browser defaults. Trusted Chromium separately proves actual Tab reentry/native input. No application/model/controller/callback/CSS injection, copied model, arbitrary scrollTop writes or exact timing claims.'};
}catch(error){return{ok:false,error:String(error),checks,snapshots,focus:{id:document.activeElement?.id,row:document.activeElement?.dataset.treeNodeId}}}
"#;
