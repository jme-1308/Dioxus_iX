//! Excluded actual-gallery CSS grid workspace and bounded synthetic native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/css_grid_workspace.rs"]
mod gallery_workspace;

#[component]
pub fn CssGridWorkspacePreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "css-grid-native-review",
            h1 { "Responsive inspection workspace" }
            gallery_workspace::CssGridWorkspaceExample {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_CSS_GRID_WORKSPACE_CHECK);
                        if let Err(error) = query.send("cssgridworkspace") {
                            eprintln!("CSS grid workspace native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native css grid workspace: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native css grid workspace failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("CSS grid workspace native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.css-grid-native-review{box-sizing:border-box;min-height:100vh;padding:24px}.css-grid-native-review>h1{font-size:28px;line-height:40px;margin:0 0 24px}@media(max-width:600px){.css-grid-native-review{padding:16px}}";

// Development replay only: synthetic DOM input/clicks exercise the real gallery
// handlers and native forms. The shipped example's measurement, area scrolling
// and input focus use Dioxus mounted handles; this driver is not a fallback.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CSS_GRID_WORKSPACE_CHECK: &str = r#"
let snapshots={},geometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById('css-grid-'+id),root=()=>get('workspace-grid')||get('workspace-grid-alternate'),owner=()=>get('workspace-owner'),status=()=>get('status'),active=()=>document.activeElement;
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<180;n++){if(predicate())return;await hold(20)}throw Error(label+'; status='+status()?.outerHTML+'; focus='+(active()?.id||active()?.tagName))};
 const rect=node=>{const r=node.getBoundingClientRect();return{left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const layout=()=>({root:rect(root()),areas:Object.fromEntries(['overview','equipment','editor','activity'].map(name=>[name,{...rect(get(name)),area:getComputedStyle(get(name)).gridArea}])),template:getComputedStyle(root()).gridTemplateAreas,viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth});
 const capture=()=>({generation:owner()?.dataset.generation,equipment:status()?.dataset.equipment,saved:Number(status()?.dataset.savedCount),title:get('title')?.value,note:get('note')?.value,preparation:get('preparation')?.value,root:root()?.id,payload:status()?.dataset.payload,focus:active()?.id||active()?.tagName});
 const focus=async(node,label)=>{node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const click=id=>{assert(get(id),id+' exists');get(id).click()};
 const input=async(id,value)=>{const node=get(id);await focus(node,id);const proto=node instanceof HTMLTextAreaElement?HTMLTextAreaElement.prototype:HTMLInputElement.prototype;Object.getOwnPropertyDescriptor(proto,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(id+' actual controlled value',()=>node.value===value)};
 await wait('actual CSS grid workspace and native controls mounted',()=>root()&&owner()&&status()&&get('title')&&get('note')&&get('preparation')&&get('save'));
 await wait('native document focus',()=>document.hasFocus());
 const original={root:root(),overview:get('overview'),equipment:get('equipment'),editor:get('editor'),activity:get('activity'),title:get('title'),note:get('note'),preparation:get('preparation')};
 assert(getComputedStyle(root()).display==='grid'&&status().dataset.equipment==='pump'&&Number(status().dataset.savedCount)===0,'native workspace starts with real CSS grid and selected pump');
 geometry.initial=layout();snapshots.initial=capture();
 await input('title','Native inlet inspection');await input('note','Keep seal draft through layout updates');await input('preparation','Retain private native preparation');
 click('fan');await wait('actual equipment selection',()=>status().dataset.equipment==='fan');
 click('editor-first');await wait('actual live area template changes',()=>getComputedStyle(root()).gridTemplateAreas!==geometry.initial.template);geometry.editorFirst=layout();
 assert(Object.entries(original).every(([name,node])=>name==='root'?root()===node:get(name)===node),'template updates retain every mounted area and native field');
 click('rename-area');await wait('actual activity area and template renamed together',()=>getComputedStyle(get('activity')).gridArea.startsWith('notes')&&getComputedStyle(root()).gridTemplateAreas.includes('notes'));
 click('alternate-id');await wait('actual grid root ID changes',()=>root().id==='css-grid-workspace-grid-alternate');
 assert(root()===original.root&&get('title')===original.title&&get('note')===original.note&&get('preparation')===original.preparation,'root id and area names update without replacing mounted owners');
 assert(get('title').value==='Native inlet inspection'&&get('note').value==='Keep seal draft through layout updates'&&get('preparation').value==='Retain private native preparation','private drafts survive real selected-equipment/template/area-name/root-id updates');
 geometry.updated=layout();snapshots.liveUpdates=capture();
 click('check-space');await wait('actual current mounted grid measurement',()=>get('space-status')?.dataset.state==='measured');const measured=get('space-status').dataset;assert(Number(measured.width)>0&&Number(measured.height)>0&&Math.abs(Number(measured.width)-root().getBoundingClientRect().width)<2,'Dioxus current mounted root returns actual native grid width');
 await focus(get('preparation'),'preparation before mounted editor navigation');click('show-editor');await wait('Dioxus area scroll and current input mounted focus',()=>active()===get('title'));geometry.measured={reported:{width:Number(measured.width),height:Number(measured.height)},actual:rect(root()),active:active().id};snapshots.mountedActions=capture();
 const native=Object.fromEntries(new FormData(get('inspection-form')).entries());assert(native.equipment==='fan'&&native.title==='Native inlet inspection'&&native.note==='Keep seal draft through layout updates'&&!Object.values(native).includes('Retain private native preparation'),'actual native successful controls own exact equipment/title/note and exclude independent preparation');
 await focus(get('save'),'native Save inspection');click('save');await wait('actual native FormEvent saves inspection',()=>Number(status().dataset.savedCount)===1);const payload=JSON.parse(status().dataset.payload);assert(payload.equipment===native.equipment&&payload.title===native.title&&payload.note===native.note,'actual FormEvent payload equals native successful controls');const saved=get('saved-records').querySelector('li');assert(saved?.dataset.equipment==='fan'&&saved.dataset.title===native.title&&saved.dataset.note===native.note,'saved native record contains exact current equipment and draft');snapshots.saved=capture();
 click('disabled');await wait('consumer disables current editing fields and Save',()=>get('title').disabled&&get('note').disabled&&get('save').disabled);click('save');assert(Number(status().dataset.savedCount)===1&&get('preparation').value==='Retain private native preparation','disabled native Save preserves saved record and private preparation');click('disabled');await wait('editing restored',()=>!get('title').disabled&&!get('save').disabled);
 const generation=Number(owner().dataset.generation);click('reset');await wait('keyed owner Reset creates fresh current mounted fields',()=>Number(owner()?.dataset.generation)>generation&&get('title')!==original.title&&get('title')?.value==='Inspect inlet seals'&&Number(status()?.dataset.savedCount)===0);
 assert(!original.root.isConnected&&!original.title.isConnected&&!original.preparation.isConnected&&root().id==='css-grid-workspace-grid'&&get('note').value===''&&get('preparation').value==='Bring the inspection kit'&&status().dataset.equipment==='pump','Reset retires old owner and restores exact initial layout/fields/selection');
 click('check-space');await wait('fresh current root handle measurement',()=>get('space-status')?.dataset.state==='measured');click('show-editor');await wait('fresh current input handle receives actual native focus',()=>active()===get('title'));await input('title','Fresh mounted inspection');click('save');await wait('fresh native form Save uses new owner',()=>Number(status().dataset.savedCount)===1&&JSON.parse(status().dataset.payload).title==='Fresh mounted inspection');
 assert(document.documentElement.scrollWidth<=innerWidth+1,'actual native grid workspace has no horizontal page overflow');geometry.reset=layout();snapshots.reset=capture();
 return{ok:true,snapshots,geometry,checks:['actual CSS grid workspace retains native controls and private drafts through live template, area name and root ID updates','Dioxus current mounted grid measurement and area scroll/input focus act on retained handles','native successful controls and real FormEvent save exact equipment/title/note without independent preparation','consumer disabled fields and Save retain current draft and saved record','keyed Reset retires old owners and fresh mounted measurement/focus/native Save remain usable'],input:'Excluded synthetic native focus/input/click events exercise actual gallery handlers and actual native FormData/FormEvent. Trusted input and physical keyboard behavior are not established by this replay.',renderer:navigator.userAgent};
} catch(error){return{ok:false,error:String(error),snapshots,geometry,status:document.getElementById('css-grid-status')?.outerHTML,focus:document.activeElement?.id||document.activeElement?.tagName}}
"#;
