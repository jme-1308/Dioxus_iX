//! Bounded Linux/WebView regression for B10.2. Input/focus events are synthetic;
//! this verifies renderer wiring, not physical keyboard or assistive technology.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/filter_mode_review.rs"]
mod filter_mode_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX filter mode probe")
                .with_visible(false)
                .with_inner_size(dioxus::desktop::LogicalSize::new(900.0, 1000.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

#[component]
fn Probe() -> Element {
    let desktop = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(40)).await;
            eprintln!("B10.2 native probe timed out");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        div { onmounted: move |_| {
            let desktop = desktop.clone();
            spawn(async move {
                let query = dom_query::eval(VERIFY);
                let _ = query.send(());
                match query.join::<serde_json::Value>().await {
                    Ok(result) if result["ok"] == true => {
                        println!("B10.2 native synthetic renderer checks: {result}");
                        desktop.close();
                    }
                    result => { eprintln!("B10.2 native probe failed: {result:?}"); std::process::exit(1); }
                }
            });
        }, filter_mode_review::FilterModeReview {} }
    } }
}

const VERIFY: &str = r#"
try {
 await dioxus.recv();
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const get=id=>document.getElementById(id), root=kind=>get(kind==='dynamic'?'b10-mode-dynamic':`b10-mode-${kind}-case`);
 const input=kind=>root(kind)?.querySelector('.ix-category-filter-text');
 const open=kind=>root(kind).querySelector('.ix-category-filter').getAttribute('data-open')==='true';
 const events=()=>JSON.parse(get('b10-mode-events').textContent), state=()=>JSON.parse(get('b10-mode-state').textContent);
 const options=kind=>[...root(kind).querySelectorAll('button.ix-category-filter-option')].map(node=>node.textContent.trim());
 const heading=kind=>root(kind).querySelector('.ix-category-filter-options-heading > span')?.textContent;
 const preview=()=>root('dynamic').querySelector('.ix-category-filter-preview');
 const operator=()=>root('dynamic').querySelector('.ix-category-filter-options-heading button');
 const reset=()=>root('dynamic').querySelector('.ix-category-filter-reset');
 const chips=kind=>root(kind).querySelectorAll('.ix-category-filter-chip').length;
 const assert=(ok,label)=>{if(!ok)throw Error(label+'; state='+JSON.stringify(state())+'; events='+JSON.stringify(events()))};
 const wait=async(test,label)=>{const end=performance.now()+3000;while(!test()){if(performance.now()>end)throw Error(label+'; events='+JSON.stringify(events()));await sleep(20)}};
 const focus=kind=>{input(kind).focus();input(kind).dispatchEvent(new FocusEvent('focusin',{bubbles:true}));};
 const openInput=kind=>{focus(kind);if(!open(kind))input(kind).click();};
 const edit=(kind,value)=>{input(kind).value=value;input(kind).dispatchEvent(new InputEvent('input',{bubbles:true,data:value,inputType:'insertText'}));};
 const enter=kind=>input(kind).dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,cancelable:true,key:'Enter',code:'Enter'}));
 const fresh=async()=>{const old=input('dynamic');get('b10-mode-reset').click();await wait(()=>input('dynamic')!==old&&events().length===0,'fresh fixture');};
 const mode=async value=>{const node=input('dynamic');get(`b10-mode-${value}`).click();await wait(()=>state().mode===value&&!open('dynamic'),'mode update and outside-click dismissal '+value);assert(node===input('dynamic'),'mode must preserve input node');};
 const choose=async()=>{openInput('dynamic');await wait(()=>options('dynamic').includes('Vendor'),'Vendor choice');root('dynamic').querySelector('button.ix-category-filter-option').click();await wait(()=>preview()?.textContent==='Vendor'&&operator(),'Vendor preview');};
 const change=()=>events().filter(event=>event.filter==='dynamic'&&event.event==='onchange').at(-1)?.value;
 await wait(()=>input('dynamic')&&input('absent'),'mount');
 openInput('absent');await wait(()=>options('absent').length===2,'omitted suggestions');assert(heading('absent')==='Filter by text'&&options('absent').join(',')==='Apple,Banana','omitted mode heading/options');
 openInput('empty');await wait(()=>heading('empty')==='Categories','empty category heading');assert(options('empty').length===0,'explicit empty suppresses suggestions');edit('empty','free text');await sleep(40);enter('empty');await wait(()=>chips('empty')===1,'free text Enter in empty category mode');assert(events().some(e=>e.filter==='empty'&&e.event==='onchange'&&e.value.tokens[0]==='free text'&&e.value.categories.length===0),'empty category free text event');
 openInput('populated');await wait(()=>options('populated').includes('Vendor'),'populated category choice');assert(options('populated').join(',')==='Vendor'&&heading('populated')==='Categories','populated suppresses plain suggestions');
 await fresh();await choose();await wait(()=>root('dynamic').querySelector('[popover]')?.matches(':popover-open'),'native top-layer membership');operator().click();await wait(()=>operator()?.getAttribute('aria-pressed')==='true','operator !=');edit('dynamic','retained draft');await sleep(40);
 const count=events().length;get('b10-mode-labels').click();await wait(()=>state().labels==='norwegian'&&!open('dynamic'),'translated labels and outside-click dismissal');openInput('dynamic');await wait(()=>operator(),'reopen translated operator');assert(input('dynamic').getAttribute('aria-label')==='Filtertekst'&&reset().getAttribute('aria-label')==='Nullstill filtre'&&operator()?.getAttribute('aria-label')==='Bytt sammenligningsoperator','translated labels on actual controls');assert(events().length===count,'labels do not emit state events');
 await mode('empty');openInput('dynamic');await wait(()=>operator(),'empty mode retained operator');assert(preview()&&preview().textContent===''&&heading('dynamic')===''&&options('dynamic').length===0&&input('dynamic').value==='retained draft','empty mode retained draft blank preview');assert(operator().getAttribute('aria-pressed')==='true'&&chips('dynamic')===1&&events().length===count,'empty mode retains operator/committed/events');
 await mode('absent');openInput('dynamic');await wait(()=>heading('dynamic')==='Filter by text','absent heading');assert(!preview()&&!operator()&&reset()&&input('dynamic').placeholder===''&&input('dynamic').value==='retained draft','absent mode hides presentation retains draft');assert(events().length===count&&chips('dynamic')===1,'absent mode no state change');
 await mode('populated');openInput('dynamic');await wait(()=>preview()?.textContent==='Vendor'&&operator(),'restored preview');assert(operator().getAttribute('aria-pressed')==='true'&&operator().getAttribute('aria-label')==='Bytt sammenligningsoperator'&&input('dynamic').value==='retained draft'&&events().length===count,'restored draft/operator/labels without events');
 await mode('absent');openInput('dynamic');edit('dynamic','');await wait(()=>options('dynamic').includes('Apple'),'plain suggestion with retained category');root('dynamic').querySelector('button.ix-category-filter-option').click();await wait(()=>change()?.tokens.includes('Apple'),'plain suggestion committed');assert(change().categories.length===0&&change().tokens.join(',')==='existing,Apple','suggestion click free text despite retained category');
 get('b10-mode-labels').click();await wait(()=>state().labels==='defaults','default labels restored');assert(input('dynamic').getAttribute('aria-label')==='Filter'&&reset().getAttribute('aria-label')==='Clear filters','fallback native labels');
 await fresh();await choose();operator().click();await wait(()=>operator()?.getAttribute('aria-pressed')==='true','typed operator');edit('dynamic','typed retained');await sleep(40);await mode('absent');enter('dynamic');await wait(()=>change()?.categories.length===1,'typed Enter retained category');assert(change().categories[0].id==='vendor'&&change().categories[0].value==='typed retained'&&change().categories[0].operator==='!='&&change().tokens.join(',')==='existing','typed Enter uses retained category/operator');
 const before=events().length;reset().focus();reset().click();await wait(()=>chips('dynamic')===0,'reset after mode transition');await sleep(80);assert(events().slice(before).map(e=>e.event).join(',')==='onbeforeclear,onchange,onclear','reset sequence preserved');assert(!open('dynamic')&&document.activeElement!==input('dynamic'),'reset remains closed without forced focus');
 return {ok:true,checks:['omitted/empty/populated modes','free-text Enter with explicit empty categories','live actual-control labels and fallbacks','in-place mode transitions preserve draft/operator/committed state without events','blank/missing/restored preview','plain suggestion click versus typed Enter with retained category','top-layer mode rendering','reset sequence/focus preserved'],input:'synthetic DOM',renderer:navigator.userAgent};
} catch(error) {return {ok:false,error:String(error)}}
"#;
