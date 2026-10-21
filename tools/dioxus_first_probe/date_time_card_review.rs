//! Excluded actual-gallery DateTimeCard composition and native form replay.
#[path = "../../examples/gallery/date_time_card.rs"]
mod gallery_card;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn DateTimeCardPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class:"date-time-card-review",
            gallery_card::DateTimeCardExample {}
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_DATE_TIME_CARD_CHECK);let _=query.send("cardcomposition");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native DateTimeCard: {value}");window.close();}
                        value=>{eprintln!("Dioxus native DateTimeCard failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;eprintln!("DateTimeCard native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.date-time-card-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.date-time-card-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_DATE_TIME_CARD_CHECK: &str = r#"
let snapshots={};
try{
 await dioxus.recv();
 const get=id=>document.getElementById('date-card-'+id),card=()=>document.getElementById('date-time-card-demo'),form=()=>get('form'),note=()=>get('note'),status=()=>get('status'),saved=()=>get('snapshot'),body=()=>card()?.querySelector('.ix-date-card-content');
 const assert=(v,label)=>{if(!v)throw Error(label)},wait=async(label,predicate)=>{for(let i=0;i<180;i++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; status='+status()?.outerHTML)};
 const setValue=async(n,value)=>{const proto=n instanceof HTMLTextAreaElement?HTMLTextAreaElement.prototype:HTMLInputElement.prototype;Object.getOwnPropertyDescriptor(proto,'value').set.call(n,value);n.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual current field value',()=>n.value===value)};
 const toggle=async(id,checked)=>{if(get(id).checked!==checked)get(id).click();await wait(id+' state applied',()=>get(id).checked===checked)};
 const submit=()=>form().requestSubmit(get('show-footer').checked?get('save-footer'):get('save-external'));
 const payload=()=>Object.fromEntries(new FormData(form()).entries()),current=()=>({values:payload(),saved:saved()?{...saved().dataset}:null,status:{...status().dataset},generation:get('owner').dataset.generation});
 const style=()=>{const c=getComputedStyle(card()),p=getComputedStyle(body());return{corners:[c.borderTopLeftRadius,c.borderTopRightRadius,c.borderBottomRightRadius,c.borderBottomLeftRadius].map(parseFloat),border:parseFloat(c.borderTopWidth),padding:[p.paddingTop,p.paddingRight,p.paddingBottom,p.paddingLeft].map(parseFloat),class:card().className}};
 await wait('actual mounted DateTimeCard native form',()=>card()&&note()&&get('focus-note')&&!get('focus-note').disabled&&status().dataset.mounted==='true');assert(get('date').value==='2026-11-03'&&get('time').value==='09:00'&&note().value===''&&get('show-footer').checked&&!get('hide-header').checked,'initial native form and default header/footer');await setValue(get('date'),'2026-11-05');await setValue(get('time'),'10:30');await setValue(note(),'Inspect valves before the maintenance window.');submit();await wait('actual native FormEvent Save snapshot',()=>status().dataset.saved==='true'&&saved()?.dataset.note==='Inspect valves before the maintenance window.');assert(saved().dataset.asset==='Pump P-101'&&saved().dataset.date==='2026-11-05'&&saved().dataset.time==='10:30'&&JSON.parse(status().dataset.payload).note===note().value,'native named form values are captured exactly');snapshots.saved=current();
 await toggle('hide-header',true);await wait('header hidden',()=>!card().querySelector('.ix-date-card-header'));await toggle('show-footer',false);await wait('footer hidden and external real form Submit available',()=>!card().querySelector('.ix-date-card-footer')&&get('save-external'));assert(note().value==='Inspect valves before the maintenance window.'&&get('date').value==='2026-11-05'&&get('time').value==='10:30','header/footer presentation changes retain body native drafts');await setValue(note(),'Retained note with the footer hidden.');submit();await wait('external associated Submit snapshots real form',()=>saved()?.dataset.note==='Retained note with the footer hidden.');get('focus-note').click();await wait('actual mounted note handle focuses current native textarea',()=>document.activeElement===note());await toggle('hide-header',false);await toggle('show-footer',true);await wait('header/footer restored',()=>card().querySelector('.ix-date-card-header')&&get('save-footer'));assert(note().value==='Retained note with the footer hidden.','restoring optional regions preserves body draft');snapshots.regions=current();
 const corners={};for(const value of ['rounded','left','right','straight']){get('corners').value=value;get('corners').dispatchEvent(new Event('change',{bubbles:true}));await wait('current corner mode '+value,()=>card().classList.contains('ix-date-corners-'+value));corners[value]=style().corners;const c=corners[value];assert(value==='rounded'?c.every(v=>v>0):value==='left'?c[0]>0&&c[3]>0&&c[1]===0&&c[2]===0:value==='right'?c[1]>0&&c[2]>0&&c[0]===0&&c[3]===0:c.every(v=>v===0),'actual computed corner treatment '+value)}await toggle('embedded',true);await wait('embedded actual border removed',()=>style().border===0);const embedded=style();await toggle('embedded',false);await toggle('time-appearance',true);await wait('time-picker padding applied',()=>card().classList.contains('ix-date-time-picker-appearance')&&style().padding[0]===0&&style().padding[2]===0);const time=style();assert(time.padding[1]>0&&time.padding[3]>0,'time appearance retains inline padding');await toggle('no-padding',true);await wait('no_padding takes precedence',()=>style().padding.every(v=>v===0));const noPadding=style();assert(note().value==='Retained note with the footer hidden.','appearance changes retain native note draft');snapshots.appearance={corners,embedded,time,noPadding};
 const previous=saved().dataset.note;await setValue(get('date'),'');submit();assert(!get('date').validity.valid&&saved().dataset.note===previous,'native required empty date blocks replacement of saved snapshot');await setValue(get('date'),'2026-11-08');await setValue(get('time'),'11:45');await setValue(note(),'');submit();await wait('valid recovery accepts optional empty note',()=>saved()?.dataset.date==='2026-11-08'&&saved().dataset.time==='11:45'&&saved().dataset.note==='');const oldNote=note(),generation=get('owner').dataset.generation;get('reset').click();await wait('keyed Reset restores defaults/current mounted handles',()=>get('owner').dataset.generation!==generation&&note()!==oldNote&&status().dataset.mounted==='true'&&!get('focus-note').disabled&&status().dataset.saved==='false'&&!saved()&&get('date').value==='2026-11-03'&&get('time').value==='09:00'&&note().value==='');assert(!get('hide-header').checked&&get('show-footer').checked&&!get('embedded').checked&&!get('time-appearance').checked&&!get('no-padding').checked&&get('corners').value==='rounded','Reset restores ordinary appearance options');get('focus-note').click();await wait('fresh mounted textarea focus after Reset',()=>document.activeElement===note());await setValue(note(),'Freshly mounted preparation.');submit();await wait('new native form saves using current handles',()=>saved()?.dataset.note==='Freshly mounted preparation.');snapshots.recovered=current();
 return{ok:true,snapshots,checks:['actual native FormEvent Save captures exact asset/date/time/uncontrolled note values into immutable local snapshot','header/footer visibility retains body drafts; external associated Submit and mounted note focus remain usable','all four computed corner modes, embedded border, time-picker padding and no_padding precedence work without losing native drafts','required empty date cannot replace saved snapshot; valid empty-note recovery and keyed Reset renew defaults/current mounts/focus/Save'],input:'Excluded synthetic input/change/DOM click and explicit requestSubmit invoke actual shared gallery native forms and mounted Dioxus controls. Native constraint validation and FormData are real; trusted typing/pointer input is separate Chromium evidence. No application state/controller/callback/CSS injection.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),snapshots,status:document.getElementById('date-card-status')?.outerHTML}}
"#;
