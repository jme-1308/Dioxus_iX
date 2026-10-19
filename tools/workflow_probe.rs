//! Bounded Linux/WebView regression for G04.1. Input/focus events are synthetic;
//! this verifies renderer wiring, not physical keyboard or assistive technology.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/workflow_review.rs"]
mod workflow_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX workflow probe")
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
            eprintln!("G04.1 native probe timed out");
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
                        println!("G04.1 native synthetic renderer checks: {result}");
                        desktop.close();
                    }
                    result => { eprintln!("G04.1 native probe failed: {result:?}"); std::process::exit(1); }
                }
            });
        }, workflow_review::WorkflowReview {} }
    } }
}

const VERIFY: &str = r#"
try {
 await dioxus.recv();
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const get=id=>document.getElementById(id);
 const wait=async(test,label)=>{const end=performance.now()+3000;while(!test()){if(performance.now()>end)throw Error(label);await sleep(20)}};
 const events=()=>JSON.parse(get('g04-events').textContent);
 const state=()=>JSON.parse(get('g04-state').textContent);
 const current=root=>[...get(root).querySelectorAll('.ix-workflow-step[data-selected=true]')].filter(node=>node.closest('.ix-workflow-steps')===get(root)).map(node=>node.id);
 const action=id=>get(id).querySelector('.ix-workflow-step-content')||get(id);
 const assert=(ok,label)=>{if(!ok)throw Error(label+'; events='+JSON.stringify(events())+'; selected='+JSON.stringify(current('g04-workflow')))};
 await wait(()=>get('g04-step-prepare')&&current('g04-workflow')[0]==='g04-step-prepare','initial index zero');
 get('g04-step-inspect').querySelector('.ix-workflow-step-content').click();
 await wait(()=>events().length===1&&current('g04-workflow')[0]==='g04-step-inspect','wrapped child click selection');
 assert(events()[0].previous===0&&events()[0].requested===1,'request reports previous selection and wrapped ordinal');
 assert(get('g04-step-prepare').dataset.position==='first'&&get('g04-step-finish').dataset.position==='last','connector endpoints');
 assert(state().selected_index===0,'activation leaves selected_index unchanged');
 action('g04-step-inspect').click();await wait(()=>events().length===2,'repeat request');assert(events()[1].previous===1,'repeat previous');
 get('g04-rerender').click();await wait(()=>current('g04-workflow')[0]==='g04-step-prepare','parent render reapplies index');
 get('g04-veto').click();await sleep(60);action('g04-step-inspect').click();await wait(()=>events().length===3,'veto request');
 assert(events()[2].vetoed===true&&current('g04-workflow')[0]==='g04-step-prepare','veto preserves selection');
 action('g04-step-disabled').click();action('g04-step-disabled').dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,cancelable:true}));await sleep(60);
 assert(events().length===3,'disabled has no activation');
 assert(action('g04-step-disabled').getAttribute('tabindex')==='-1'&&action('g04-step-disabled').getAttribute('aria-disabled')==='true','disabled accessibility');
 get('g04-veto').click();await sleep(60);
 action('g04-step-inspect').focus();action('g04-step-inspect').dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,cancelable:true}));
 await wait(()=>events().length===4&&current('g04-workflow')[0]==='g04-step-inspect','Enter selection');
 action('g04-step-finish').dispatchEvent(new KeyboardEvent('keydown',{key:' ',bubbles:true,cancelable:true}));await wait(()=>events().length===5&&current('g04-workflow')[0]==='g04-step-finish','Space selection');
 assert(action('g04-step-inspect').getAttribute('tabindex')==='0','enabled tab stop');
 get('g04-select-disabled').click();await wait(()=>current('g04-workflow')[0]==='g04-step-disabled','disabled selected programmatically');
 assert(getComputedStyle(get('g04-step-disabled').querySelector('.ix-workflow-status-icon')).color===getComputedStyle(action('g04-step-disabled')).color,'disabled selected status and text share disabled color');
 get('g04-select-out-of-range').click();await wait(()=>current('g04-workflow').length===0,'out-of-range clears');
 get('g04-select-zero').click();await wait(()=>current('g04-workflow')[0]==='g04-step-prepare','external zero');
 get('g04-status').click();await wait(()=>!!get('g04-step-prepare').querySelector('[aria-label="Error"]'),'error status icon');
 get('g04-status').click();await wait(()=>!!get('g04-step-prepare').querySelector('[aria-label="Circle dot"]'),'open selected icon restored');
 get('g04-orientation').click();await wait(()=>get('g04-workflow').dataset.vertical==='true'&&get('g04-step-prepare').dataset.vertical==='true','vertical propagation');
 const marker=get('g04-step-prepare').querySelector('.ix-workflow-marker').getBoundingClientRect(),label=get('g04-step-prepare').querySelector('.ix-workflow-label').getBoundingClientRect();
 const verticalLabelGap=label.left-marker.right;assert(verticalLabelGap>=8,'fallback vertical icon label spacing');
 get('g04-append').click();await wait(()=>!!get('g04-step-added-1'),'append');
 get('g04-reorder').click();await wait(()=>current('g04-workflow')[0]==='g04-step-added-1','reorder updates ordinal selection');
 action('g04-step-prepare').click();await wait(()=>events().length===6,'reordered activation');
 assert(events()[5].requested===4&&events()[5].previous===0,'live DOM ordinal after reverse');
 get('g04-remove').click();await wait(()=>!get('g04-step-prepare'),'remove reordered last');
 await wait(()=>current('g04-workflow')[0]==='g04-step-added-1','remove reapplies index');
 action('g04-other-b').click();await wait(()=>current('g04-other')[0]==='g04-other-b','independent workflow activation');
 assert(current('g04-workflow')[0]==='g04-step-added-1','independent workflow isolation');
 const before=events().length;get('g04-clickable').click();await sleep(60);action('g04-step-inspect').click();await sleep(60);
 assert(events().length===before&&action('g04-step-inspect').getAttribute('role')!=='button','nonclickable suppresses');
 assert(get('g04-custom-content')?.textContent==='Release ready','custom content retained');
 const nestedStart=events().length;
 action('g04-inner-b').click();await wait(()=>events().length===nestedStart+1&&current('g04-nested-inner')[0]==='g04-inner-b','nested click');
 assert(current('g04-nested-outer')[0]==='g04-outer-a'&&events()[nestedStart].source==='inner','nested click boundary');
 action('g04-inner-a').dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,cancelable:true}));await wait(()=>events().length===nestedStart+2&&current('g04-nested-inner')[0]==='g04-inner-a','nested Enter');
 action('g04-inner-b').dispatchEvent(new KeyboardEvent('keydown',{key:' ',bubbles:true,cancelable:true}));await wait(()=>events().length===nestedStart+3&&current('g04-nested-inner')[0]==='g04-inner-b','nested Space');
 assert(events().slice(nestedStart).every(event=>event.source==='inner'),'nested keyboard boundary');
 action('g04-outer-b').click();await wait(()=>events().length===nestedStart+4&&current('g04-nested-outer')[0]==='g04-outer-b','outer click');
 assert(current('g04-nested-inner')[0]==='g04-inner-b','outer leaves inner selected');
 const report={ok:true,verticalLabelGap,requests:events(),checks:['default/external/disabled/out-of-range selection','wrapped ordinal','repeat/veto/order','Enter/Space','parent rerender reset','status/orientation','keyed append/reverse/remove','independent workflow','nonclickable','custom content','nested click/Enter/Space boundaries'],input:'synthetic DOM',renderer:navigator.userAgent};
 get('g04-reset').click();await wait(()=>!!get('g04-step-prepare'),'reset mounted');
 get('g04-remove').click();await wait(()=>!get('g04-step-finish'),'remove to three');
 get('g04-remove').click();await wait(()=>!get('g04-step-disabled'),'remove to two');
 get('g04-remove').click();await wait(()=>!get('g04-step-inspect'),'remove to single');
 assert(current('g04-workflow')[0]==='g04-step-prepare','single retains index zero');
 assert(get('g04-step-prepare').dataset.position==='single','single connector');
 return report;
} catch(error) {return {ok:false,error:String(error)}}
"#;
