//! Bounded Linux/WebView regression for G03. Input/focus events are synthetic;
//! this verifies renderer wiring, not physical keyboard or assistive technology.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/text_commit_review.rs"]
mod text_commit_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX text commit probe")
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
            futures_timer::Delay::new(Duration::from_secs(25)).await;
            eprintln!("G03 native probe timed out");
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
                        println!("G03 native synthetic renderer checks: {result}");
                        desktop.close();
                    }
                    result => { eprintln!("G03 native probe failed: {result:?}"); std::process::exit(1); }
                }
            });
        }, text_commit_review::TextCommitReview {} }
    } }
}

const VERIFY: &str = r#"
try {
 await dioxus.recv();
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const get=id=>document.getElementById(id);
 const wait=async(test,label)=>{const end=performance.now()+2000;while(!test()){if(performance.now()>end)throw Error(label);await sleep(20)}};
 await wait(()=>get('g03-input')&&get('g03-events'),'fixture mount');
 const events=()=>JSON.parse(get('g03-events').textContent);
 const commits=field=>events().filter(e=>e.field===field&&e.event==='onchange');
 const assert=(ok,label)=>{if(!ok)throw Error(label+'; events='+JSON.stringify(events()))};
 // Hidden WebViews may not deliver native focus events. Explicit synthetic
 // focusin/focusout exercise the same Rust handlers without claiming OS input.
 const focus=id=>{const node=get(id);node.focus();node.dispatchEvent(new FocusEvent('focusin',{bubbles:true}));};
 const blur=id=>{const node=get(id);node.blur();node.dispatchEvent(new FocusEvent('focusout',{bubbles:true}));};
 const edit=(id,value)=>{const node=get(id);node.value=value;node.dispatchEvent(new InputEvent('input',{bubbles:true,data:value,inputType:'insertText'}));};
 const enter=id=>get(id).dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,cancelable:true,key:'Enter',code:'Enter'}));
 get('g03-reset').click();await sleep(60);
 focus('g03-input');edit('g03-input','changed');await wait(()=>events().some(e=>e.event==='onvaluechange'&&e.value==='changed'),'input value');
 enter('g03-input');await wait(()=>commits('input').length===1,'Enter commit');
 get('g03-input').dispatchEvent(new Event('change',{bubbles:true}));blur('g03-input');await sleep(80);
 assert(commits('input').length===1&&commits('input')[0].value==='changed','Enter/change/blur deduplication');
 focus('g03-input');blur('g03-input');await sleep(60);assert(commits('input').length===1,'unchanged refocus');
 focus('g03-input');edit('g03-input','blurred');await sleep(60);const blurStart=events().length;blur('g03-input');await wait(()=>commits('input').length===2,'changed blur');
 const blurLog=events().slice(blurStart).filter(e=>e.field==='input');
 assert(blurLog.findIndex(e=>e.event==='onixblur')<blurLog.findIndex(e=>e.event==='onchange')&&blurLog[0]?.event==='onixblur','ixBlur before current blur commit');
 assert(get('g03-submits').textContent.trim()==='0','suppressed submit');
 focus('g03-input');edit('g03-input','temporary');await sleep(60);edit('g03-input','blurred');await sleep(60);blur('g03-input');await sleep(60);
 assert(commits('input').length===2,'reverted edit');
 get('g03-programmatic').click();await sleep(60);assert(commits('input').length===2,'programmatic no immediate commit');
 focus('g03-input');blur('g03-input');await sleep(60);assert(commits('input').length===2,'programmatic refocus baseline');
 focus('g03-textarea');edit('g03-textarea','line one\nline two');await sleep(60);enter('g03-textarea');await sleep(60);
 assert(commits('textarea').length===0&&get('g03-textarea').value.includes('\n'),'Textarea Enter no commit/newline retained');
 blur('g03-textarea');await wait(()=>commits('textarea').length===1,'Textarea blur commit');
 assert(commits('textarea')[0].value==='line one\nline two','Textarea committed multiline value');
 const control=get('g03-tooltip-input'),description=control.getAttribute('aria-describedby'),tip=get(description);
 assert(tip?.getAttribute('role')==='tooltip','feedback description');
 assert(!get('g03-blank-tooltip')?.closest('.ix-field')?.querySelector('[role=tooltip]'),'blank feedback has no tooltip');
 const anchor=get(tip.dataset.anchorId);assert(tip.dataset.open!=='true','tooltip initially closed');
 const started=performance.now();
 const opened=new Promise(resolve=>{const observer=new MutationObserver(()=>{if(tip.dataset.open==='true'){observer.disconnect();resolve(performance.now()-started)}});observer.observe(tip,{attributes:true,attributeFilter:['data-open']})});
 anchor.dispatchEvent(new MouseEvent('mouseenter'));
 const tooltipDelayMs=await opened;assert(tooltipDelayMs>=450,'tooltip delayed opening');
 assert(tip.textContent.trim().length>0,'tooltip text');
 document.dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,key:'Escape'}));await wait(()=>tip.dataset.open==='false','Escape closes tooltip');
 const oldId=tip.id;get('g03-unmount-toggle').click();await wait(()=>!get(oldId),'tooltip unmount');
 return {ok:true,tooltipDelayMs,inputCommits:commits('input').length,textareaCommits:commits('textarea').length,checks:['Enter/blur deduplication','unchanged/reverted','ixBlur order','programmatic baseline','textarea multiline','tooltip delay/Escape/unmount','blank feedback'],input:'synthetic DOM',renderer:navigator.userAgent};
} catch(error) {return {ok:false,error:String(error)}}
"#;
