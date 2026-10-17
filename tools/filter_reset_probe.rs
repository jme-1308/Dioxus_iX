//! Bounded Linux/WebView regression for B10.1. Input/focus events are synthetic;
//! this verifies renderer wiring, not physical keyboard or assistive technology.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/filter_reset_review.rs"]
mod filter_reset_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX filter reset probe")
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
            eprintln!("B10.1 native probe timed out");
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
                        println!("B10.1 native synthetic renderer checks: {result}");
                        desktop.close();
                    }
                    result => { eprintln!("B10.1 native probe failed: {result:?}"); std::process::exit(1); }
                }
            });
        }, filter_reset_review::FilterResetReview {} }
    } }
}

const VERIFY: &str = r#"
try {
 await dioxus.recv();
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const get=id=>document.getElementById(id),scope=kind=>get(`b10-${kind}-filter`);
 const input=kind=>scope(kind)?.querySelector('.ix-category-filter-text');
 const reset=kind=>scope(kind).querySelector('[aria-label="Clear filters"]');
 const toggle=kind=>scope(kind).querySelector('[aria-label="Toggle comparison operator"]');
 const chips=kind=>(scope(kind)?.querySelectorAll('.ix-category-filter-chip').length||0);
 const preview=kind=>scope(kind)?.querySelector('.ix-category-filter-preview');
 const open=kind=>scope(kind).querySelector('.ix-category-filter').getAttribute('data-open')==='true';
 const clickReset=kind=>{reset(kind).focus();reset(kind).click()};
 const events=()=>JSON.parse(get('b10-events').textContent);
 const wait=async(test,label)=>{const end=performance.now()+3000;while(!test()){if(performance.now()>end)throw Error(label+'; state='+JSON.stringify({localChips:chips('local'),controlledChips:chips('controlled'),events:events(),localValue:input('local')?.value,controlledValue:input('controlled')?.value}));await sleep(20)}};
 const assert=(ok,label)=>{if(!ok)throw Error(label+'; events='+JSON.stringify(events()))};
 const focus=kind=>{input(kind).focus();input(kind).dispatchEvent(new FocusEvent('focusin',{bubbles:true}));};
 const edit=(kind,value)=>{input(kind).value=value;input(kind).dispatchEvent(new InputEvent('input',{bubbles:true,data:value,inputType:'insertText'}));};
 const choose=async kind=>{focus(kind);if(!open(kind))input(kind).click();await wait(()=>open(kind),'input click opens');edit(kind,'');await sleep(40);const option=[...scope(kind).querySelectorAll('button.ix-category-filter-option')].find(node=>node.textContent.trim()==='Vendor');assert(!!option,'Vendor choice');option.click();await wait(()=>preview(kind)?.textContent==='Vendor'&&toggle(kind),'category preview');};
 const clearLog=(start,kind)=>events().slice(start).filter(event=>event.filter===kind).map(event=>event.event);
 const expected=(start,kind,names,label)=>assert(JSON.stringify(clearLog(start,kind))===JSON.stringify(names),label);
 let freshCount=0;
 const fresh=async()=>{const oldInput=input('local');get('b10-reset').click();freshCount++;await wait(()=>input('local')!==oldInput&&chips('local')===1&&chips('controlled')===1&&events().length===0,'fresh fixtures '+freshCount);};
 await wait(()=>input('local')&&input('controlled'),'mount');await fresh();
 focus('local');edit('local','pending plain');await wait(()=>input('local').value==='pending plain'&&events().length>0,'plain draft');
 let start=events().length;clickReset('local');await wait(()=>chips('local')===0,'plain reset');await sleep(80);
 expected(start,'local',['onbeforeclear','onchange','onclear'],'plain callback order/count');
 assert(input('local').value==='pending plain'&&!open('local'),'plain draft retained and closed');assert(document.activeElement!==input('local'),'no forced input focus after reset');
 focus('local');await sleep(40);assert(!open('local'),'focus alone keeps reset popup closed');input('local').click();await wait(()=>open('local'),'next intentional click opens');
 await fresh();await choose('local');toggle('local').click();await wait(()=>toggle('local')?.textContent.trim()==='=','operator not equal');
 edit('local','draft text');await sleep(40);get('b10-veto').click();await sleep(40);focus('local');if(!open('local'))input('local').click();await wait(()=>toggle('local')?.textContent.trim()==='=','reopen preview after outside veto toggle');start=events().length;clickReset('local');await sleep(80);
 expected(start,'local',['onbeforeclear'],'veto callback only');assert(chips('local')===1&&preview('local')&&input('local').value==='draft text'&&toggle('local')?.textContent.trim()==='=','veto preserves draft/operator/committed');
 get('b10-veto').click();await sleep(40);start=events().length;clickReset('local');await wait(()=>chips('local')===0&&!preview('local'),'preview reset');await sleep(80);
 expected(start,'local',['onbeforeclear','oncategory','onchange','onclear'],'preview callback order/count');assert(input('local').value==='draft text'&&!open('local'),'preview draft retained and closed');
 await choose('local');assert(toggle('local').textContent.trim()==='=','operator retained after reset');
 // Queue a value-selection input-focus request and reset in the same event turn.
 const queuedValue=[...scope('local').querySelectorAll('button.ix-category-filter-option')].find(node=>node.textContent.includes('Alpha'));assert(!!queuedValue,'queued Alpha value');queuedValue.click();clickReset('local');await wait(()=>!preview('local')&&!open('local'),'queued focus reset');await sleep(100);assert(!open('local')&&document.activeElement!==input('local'),'reset cancels queued input focus without reopening');
 await fresh();get('b10-reject').click();await sleep(40);focus('controlled');edit('controlled','controlled pending');await sleep(40);start=events().length;clickReset('controlled');await sleep(80);
 expected(start,'controlled',['onbeforeclear','onchange','onclear'],'controlled rejection order');assert(chips('controlled')===1&&input('controlled').value==='controlled pending'&&!open('controlled'),'rejection retains parent committed state and closes');assert(document.activeElement!==input('controlled'),'controlled rejection does not force focus');
 assert(JSON.parse(get('b10-controlled-state').textContent).tokens[0]==='existing','parent state rejection');
 get('b10-reject').click();await sleep(40);start=events().length;clickReset('controlled');await wait(()=>chips('controlled')===0,'controlled accepted reset');await sleep(80);
 expected(start,'controlled',['onbeforeclear','onchange','onclear'],'controlled acceptance order');assert(input('controlled').value==='controlled pending'&&!open('controlled'),'accepted controlled draft and closure');
 await fresh();await choose('controlled');toggle('controlled').click();await wait(()=>toggle('controlled')?.textContent.trim()==='=','top layer operator');edit('controlled','next draft');await sleep(40);start=events().length;clickReset('controlled');await wait(()=>chips('controlled')===0&&!preview('controlled'),'top layer preview clear');await sleep(80);
 expected(start,'controlled',['onbeforeclear','oncategory','onchange','onclear'],'top layer preview order');assert(!open('controlled')&&input('controlled').value==='next draft','top layer retained draft and closure');
 return {ok:true,checks:['plain and preview callback order/count','veto preserves draft/operator/state','retained draft/operator','no async reopening','focus alone stays closed, intentional click opens','controlled acceptance/rejection','inline and top-layer'],input:'synthetic DOM',renderer:navigator.userAgent};
} catch(error) {return {ok:false,error:String(error)}}
"#;
