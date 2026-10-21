//! Excluded actual-gallery controlled clock heading/period/confirmation composition.
// This probe imports only the clock case; neighboring calendar demos stay unused.
#[allow(dead_code)]
#[path = "../../examples/gallery/datetime.rs"]
pub(super) mod gallery_datetime;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn ClockCompositionPreview() -> Element {
    let mut time = use_signal(|| IxTime::new(13, 7, 4, 0).unwrap());
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_datetime::STYLES} }
        IxTheme { class:"clock-composition-review",
            gallery_datetime::ClockCompositionExample {time:time(),ontimechange:move |value|time.set(value)}
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_CLOCK_COMPOSITION_CHECK);let _=query.send("clockcomposition");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native Clock composition: {value}");window.close();}
                        value=>{eprintln!("Dioxus native Clock composition failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;eprintln!("Clock composition native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.clock-composition-review{box-sizing:border-box;min-height:100vh;padding:24px}@media(max-width:600px){.clock-composition-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CLOCK_COMPOSITION_CHECK: &str = r#"
let snapshots={};
try{
 await dioxus.recv();
 const get=id=>document.getElementById('clock-'+id),host=()=>document.getElementById('clock-composition'),status=()=>document.getElementById('clock-composition-status'),card=()=>host()?.querySelector('.ix-date-time-card'),period=label=>[...host().querySelectorAll('.ix-clock-meridiem button')].find(n=>n.textContent.trim()===label),confirm=()=>card()?.querySelector('.ix-date-card-footer button'),list=unit=>host()?.querySelector('.ix-clock-options[aria-label="'+unit+'"]'),selected=unit=>list(unit)?.querySelector('button[aria-selected="true"]'),option=(unit,value)=>[...list(unit).querySelectorAll('button')].find(n=>n.textContent.trim()===value);
 const assert=(v,label)=>{if(!v)throw Error(label)},wait=async(label,predicate)=>{for(let i=0;i<180;i++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; status='+status()?.outerHTML+'; focus='+(document.activeElement?.textContent||document.activeElement?.id))};
 const draft=()=>status().dataset.draft,confirmed=()=>status().dataset.confirmed,key=(n,k)=>n.dispatchEvent(new KeyboardEvent('keydown',{key:k,code:k,bubbles:true,cancelable:true}));
 const toggle=async(id,checked)=>{if(get(id).checked!==checked)get(id).click();await wait('actual '+id+' checkbox',()=>get(id).checked===checked)};
 const clockNames=()=>[...card().querySelectorAll('.ix-clock-column')].map(column=>{const heading=column.querySelector('.ix-clock-column-heading').textContent.trim(),options=column.querySelector('.ix-clock-options'),chosen=options.querySelector('[aria-selected=true]'),first=options.querySelector('button[role=option]');return{heading,list:options.getAttribute('aria-label'),selected:{text:chosen.textContent.trim(),name:chosen.getAttribute('aria-label')},first:{text:first.textContent.trim(),name:first.getAttribute('aria-label')}}});
 const assertClockNames=()=>{const names=clockNames();assert(names.length===3&&names.every((unit,index)=>unit.heading===['hr','min','sec'][index]&&unit.list===unit.heading&&[unit.selected,unit.first].every(option=>option.name===unit.heading+': '+option.text)),'actual default headings name their list and formatted selected/first options');return names};
 const snapshot=()=>({state:{...status().dataset},period:{AM:period('AM').getAttribute('aria-pressed'),PM:period('PM').getAttribute('aria-pressed')},selected:{hour:selected('hr').textContent.trim(),minute:selected('min').textContent.trim(),second:selected('sec').textContent.trim()},heading:!!card().querySelector('.ix-date-card-header'),footer:!!confirm(),embedded:card().classList.contains('ix-date-embedded')});
 await wait('actual controlled composition mounted',()=>status()&&period('AM')&&period('PM')&&confirm()&&selected('sec'));assert(draft()==='13:07:04'&&confirmed()===''&&get('hide-heading').checked&&get('embedded').checked&&get('show-confirmation').checked,'default controlled time and independent composition flags');assert(!card().querySelector('.ix-date-card-header')&&card().querySelector('.ix-date-card-content .ix-clock-period-body .ix-clock-meridiem')&&card().classList.contains('ix-date-embedded'),'hidden heading retains body period and explicit embedded confirmation');assert(period('PM').getAttribute('aria-pressed')==='true'&&selected('min').textContent.trim()==='07'&&selected('sec').textContent.trim()==='04','current PM and subordinate units are real selected controls');snapshots.initialNames=assertClockNames();period('AM').click();await wait('AM changes period preserving minutes and seconds',()=>draft()==='01:07:04'&&period('AM').getAttribute('aria-pressed')==='true');confirm().click();await wait('actual Confirm publishes current controlled AM time',()=>confirmed()==='01:07:04');snapshots.firstConfirmed=snapshot();
 period('PM').click();await wait('new PM draft retains confirmed AM record',()=>draft()==='13:07:04'&&confirmed()==='01:07:04');period('AM').click();await wait('AM request preserves subordinate units',()=>draft()==='01:07:04');period('PM').click();await wait('current PM draft restored independently',()=>draft()==='13:07:04');assert(confirmed()==='01:07:04','later period edits leave immutable confirmed value');confirm().click();await wait('second Confirm captures current PM time',()=>confirmed()==='13:07:04');snapshots.periodDraft=snapshot();
 selected('min').focus();key(selected('min'),'ArrowDown');await wait('Arrow moves provisional native minute focus without draft commit',()=>document.activeElement===option('min','08')&&selected('min').textContent.trim()==='07'&&draft()==='13:07:04'&&confirmed()==='13:07:04');key(document.activeElement,'Enter');await wait('unit Enter commits actual controlled minute only',()=>draft()==='13:08:04'&&selected('min').textContent.trim()==='08');assert(confirmed()==='13:07:04'&&selected('sec').textContent.trim()==='04','clock draft change preserves seconds and confirmed snapshot');snapshots.unitDraft=snapshot();snapshots.unitNames=assertClockNames();
 await toggle('hide-heading',false);await wait('visible heading restores existing header period placement',()=>card().querySelector('.ix-date-card-header .ix-clock-meridiem')&&!card().querySelector('.ix-clock-period-body'));await toggle('embedded',false);await wait('standalone card appearance',()=>!card().classList.contains('ix-date-embedded'));await toggle('show-confirmation',false);await wait('explicit false hides footer independently of standalone',()=>!confirm());assert(draft()==='13:08:04'&&confirmed()==='13:07:04','live heading/appearance/footer changes retain current draft and immutable record');await toggle('embedded',true);await toggle('hide-heading',true);await wait('embedded hidden heading retains functional period body without footer',()=>card().classList.contains('ix-date-embedded')&&card().querySelector('.ix-clock-period-body')&&!confirm());await toggle('show-confirmation',true);await wait('explicit true restores embedded footer',()=>confirm());confirm().click();await wait('live restored confirmation reports current draft',()=>confirmed()==='13:08:04');snapshots.composed=snapshot();
 period('AM').click();await wait('fresh unconfirmed change before Restore',()=>draft()==='01:08:04'&&confirmed()==='13:08:04');get('restore').click();await wait('Restore returns exact shared time/flags and clears record',()=>draft()==='13:07:04'&&confirmed()===''&&get('hide-heading').checked&&get('embedded').checked&&get('show-confirmation').checked&&period('PM').getAttribute('aria-pressed')==='true'&&confirm());assert(!card().querySelector('.ix-date-card-header')&&card().querySelector('.ix-clock-period-body')&&selected('min').textContent.trim()==='07'&&selected('sec').textContent.trim()==='04','restored current controls retain useful hidden-header composition');period('AM').click();await wait('restored current period controls remain operative',()=>draft()==='01:07:04');confirm().click();await wait('restored Confirm reports current time again',()=>confirmed()==='01:07:04');snapshots.restored=snapshot();
 return{ok:true,snapshots,checks:['default clock heading/list/selected and first-option accessible names agree on current formatted values before and after unit commit','hidden-heading embedded12h clock retains real body AM/PM controls and explicit footer; AM preserves07:04 and Confirm publishes exact current time','later period edits preserve immutable confirmed value until explicit current PM confirmation','minute ArrowDown is provisional focus; Enter commits current unit while seconds/confirmed snapshot remain','live heading/embedded/footer overrides preserve draft/record and current period controls; restored footer confirms actual draft','Restore resets shared13:07:04/default flags/confirmed record and current period/Confirm controls work again'],input:'Excluded synthetic DOM click/key events and explicit native focus invoke actual shared controlled ClockCompositionExample and TimePicker. Arrow/Enter component handlers are exercised; trusted native ToggleButton Enter/Space default activation is separate Chromium evidence. No application state/controller/callback/CSS injection or exact timing/frame chronology.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),snapshots,status:document.getElementById('clock-composition-status')?.outerHTML}}
"#;
