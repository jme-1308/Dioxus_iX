//! Excluded actual-gallery SelectItem listbox workflow and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/select_item_review.rs"]
mod gallery_select_item;

#[component]
pub fn SelectItemPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "select-item-native-review",
            h1 { "Selectable inspection queue" }
            Button { id: "select-item-preview-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            button { id: "select-item-preview-outside", r#type: "button", class: "ix-button ix-button-secondary", "Independent destination" }
            div { id: "select-item-preview-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_select_item::SelectItemReviewExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = super::dom_query::eval(NATIVE_SELECT_ITEM_CHECK);
                        if let Err(error) = query.send("selectitems") {
                            eprintln!("SelectItem native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native SelectItem: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native SelectItem failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("SelectItem native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.select-item-native-review{box-sizing:border-box;min-height:100vh;padding:24px}.select-item-native-review>h1{font-size:28px;line-height:1.4;margin:0 0 24px}.select-item-native-review h2{font-size:22px;line-height:1.4}.select-item-native-review p{max-width:65ch;line-height:1.5}#select-item-preview-outside{margin-left:12px}#select-item-preview-frame{margin-top:24px}@media(max-width:600px){.select-item-native-review{padding:16px}}";

// Development-only replay exercises actual Rust gallery handlers and native focus.
// Its synthetic key/click input does not establish trusted defaults or screen-reader output.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SELECT_ITEM_CHECK: &str = r#"
let snapshots={},semantics={},geometry={};
try {
 await dioxus.recv();
 const byId=id=>document.getElementById(id),root=()=>byId('select-item-review'),result=()=>byId('select-item-result'),trigger=()=>root()?.querySelector('.select-item-filter[role="button"]'),panel=()=>byId(trigger()?.getAttribute('aria-controls')),option=value=>byId('select-item-option-'+value),active=()=>document.activeElement;
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let attempt=0;attempt<180;attempt++){if(predicate())return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; focus='+(active()?.id||active()?.tagName))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const opened=()=>trigger()?.getAttribute('aria-expanded')==='true'&&!panel()?.hidden;
 const closed=()=>trigger()?.getAttribute('aria-expanded')==='false'&&panel()?.hidden;
 const changes=()=>Number(result()?.dataset.changes),rows=()=>[...root().querySelectorAll('[id^="select-item-row-"]')].map(node=>node.id.replace('select-item-row-','')).sort();
 const snapshot=()=>({state:{...result().dataset},rows:rows(),focus:active()?.id,expanded:trigger().getAttribute('aria-expanded'),selected:[...panel().querySelectorAll('[role="option"][aria-selected="true"]')].map(node=>node.id)});
 const expectFilter=async(value,count,expectedChanges,expectedRows)=>{await wait(value+' updates real Rust inspection rows once',()=>result()?.dataset.filter===value&&Number(result()?.dataset.count)===count&&changes()===expectedChanges&&rows().join(',')===expectedRows.slice().sort().join(',')&&closed());assert(option(value).getAttribute('aria-selected')==='true','selected state follows actual '+value+' filter')};
 const open=async(selected,label)=>{await focus(trigger(),label+' trigger');key(trigger(),'ArrowDown');await wait(label+' selected opening focus',()=>opened()&&active()===option(selected));};
 const go=async(value,target,label)=>{key(active(),value);await wait(label,()=>active()===option(target));};
 const rect=node=>{const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};

 await wait('actual SelectItem gallery mounted',()=>root()&&result()&&trigger()&&panel()&&['all','open','completed','unavailable'].every(value=>option(value)));
 await wait('native document focused',()=>document.hasFocus());
 assert(result().dataset.filter==='open'&&result().dataset.count==='2'&&changes()===0&&rows().join(',')==='motor,pump'&&closed(),'initial selected filter drives two real open inspections');
 assert(trigger().getAttribute('aria-haspopup')==='listbox'&&trigger().getAttribute('aria-label')==='Filter inspections by status'&&panel().getAttribute('role')==='listbox'&&panel().getAttribute('aria-label')==='Filter inspections by status','named trigger explicitly owns named listbox');
 assert(panel().querySelectorAll('[role="option"]').length===4,'one semantic option per actual choice');
 semantics.options=['all','open','completed','unavailable'].map(value=>{const node=option(value),label={all:'All inspections',open:'Open inspections',completed:'Completed inspections',unavailable:'Unavailable inspections'}[value];assert(node.getAttribute('role')==='option'&&node.getAttribute('aria-hidden')!=='true'&&node.getAttribute('aria-label')===label,'public '+value+' has exposed option role and accessible name');assert(node.getAttribute('aria-selected')===String(value==='open')&&node.getAttribute('aria-disabled')===String(value==='unavailable'),'public '+value+' exposes current selected and disabled state');assert(!node.querySelector('[role="option"]'),'visual child cannot duplicate option semantics');return {id:node.id,role:node.getAttribute('role'),name:node.getAttribute('aria-label'),selected:node.getAttribute('aria-selected'),disabled:node.getAttribute('aria-disabled')}});
 snapshots.initial=snapshot();

 await open('open','initial keyboard');
 await go('ArrowDown','completed','ArrowDown reaches next enabled option');
 await go('ArrowDown','all','ArrowDown wraps and skips unavailable');
 await go('ArrowUp','completed','ArrowUp skips unavailable at wrapping edge');
 await go('Home','all','Home reaches first option');
 await go('End','completed','End reaches last enabled option');
 assert(changes()===0&&result().dataset.filter==='open','navigation alone leaves actual filter unchanged');
 assert(getComputedStyle(option('unavailable').querySelector('.ix-select-item-option')).opacity==='0.45','disabled composed row shares native disabled visual treatment');
 snapshots.navigation=snapshot();

 key(active(),'Enter');await expectFilter('completed',1,1,['valve']);snapshots.enter=snapshot();
 await open('completed','reopen after Enter');await go('Home','all','Home before Space');key(active(),' ');await expectFilter('all',3,2,['pump','motor','valve']);snapshots.space=snapshot();

 trigger().click();await wait('pointer opens actual options',opened);
 option('open').querySelector('.ix-select-item-option').click();await expectFilter('open',2,3,['pump','motor']);
 trigger().click();await wait('pointer opens disabled check',opened);
 const beforeDisabled=changes();option('unavailable').querySelector('.ix-select-item-option').click();
 await wait('disabled click remains inert',()=>changes()===beforeDisabled&&result().dataset.filter==='open');
 assert(option('unavailable').getAttribute('aria-disabled')==='true','unavailable option remains disabled');
 // Synthetic click bypasses CSS pointer hit testing, intentionally checking the Rust guard.
 if(opened()){key(trigger(),'Escape');await wait('close after disabled attempt',closed)}
 snapshots.pointerAndDisabled=snapshot();

 await open('open','Escape retention');await go('End','completed','move without selecting before Escape');key(active(),'Escape');await wait('Escape closes listbox',closed);assert(changes()===3&&result().dataset.filter==='open','Escape preserves applied filter');
 await open('open','outside retention');
 const outside=byId('select-item-preview-outside');await focus(outside,'independent outside destination');outside.click();await wait('outside click closes listbox and preserves meaningful focus',()=>closed()&&active()===outside);assert(changes()===3&&result().dataset.filter==='open','outside dismissal preserves applied filter');
 trigger().click();await wait('open before pointer completed selection',opened);option('completed').querySelector('.ix-select-item-option').click();await expectFilter('completed',1,4,['valve']);
 byId('select-item-reset').click();await expectFilter('open',2,0,['pump','motor']);snapshots.dismissalAndReset=snapshot();

 byId('select-item-preview-narrow').click();await wait('native actual340px gallery frame',()=>byId('select-item-preview-frame').getBoundingClientRect().width<=341);
 trigger().scrollIntoView({block:'center',inline:'nearest'});await focus(trigger(),'narrow trigger');
 const frame=rect(byId('select-item-preview-frame')),button=rect(trigger()),point=document.elementFromPoint(button.left+button.width/2,button.top+button.height/2);assert(button.width>0&&button.height>0&&button.left>=frame.left-1&&button.right<=frame.right+1&&(point===trigger()||trigger().contains(point)),'native narrow trigger fits and is hit-testable');
 await open('open','narrow keyboard');const popup=rect(panel());assert(popup.width>0&&popup.left>=-1&&popup.right<=innerWidth+1,'native narrow-frame popup stays inside actual desktop viewport');
 await go('End','completed','narrow selection target');key(active(),'Enter');await expectFilter('completed',1,1,['valve']);
 geometry={frame,trigger:button,popup,viewport:{width:innerWidth,height:innerHeight}};snapshots.narrow=snapshot();
 return {ok:true,semantics,snapshots,geometry,checks:['named Listbox and one exposed named selected/disabled option per visible choice','keyboard opening focuses selected option; arrows/Home/End skip disabled without selection','Enter and Space each update actual Rust queue once and close','pointer selection and disabled Rust guard','Escape/outside retain applied filter and Reset restores initial queue','actual340px frame trigger hit and viewport-contained popup with working selection'],input:'Excluded synthetic keydown/DOM click with explicit actual native focus exercises gallery Rust handlers. No physical input, trusted key defaults or assistive-technology output is claimed.',viewportQualification:'340px constrains actual gallery frame inside unchanged Linux WebKitGTK desktop viewport; true narrow viewport is verified separately by lead browser evidence.',renderer:navigator.userAgent};
}catch(error){return {ok:false,error:String(error),stack:error.stack,semantics,snapshots,geometry,focus:document.activeElement?.id,renderer:navigator.userAgent}}
"#;
