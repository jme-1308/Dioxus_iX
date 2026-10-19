//! Excluded responsive workspace and bounded synthetic Linux resize checks.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn Workspace(note: Signal<String>, force: Option<ApplicationBreakpoint>) -> Element {
    let mut note = note;
    rsx! {
        Application {class:"r0251-auto",force_breakpoint:force,
            header:rsx! {ApplicationHeader {name:"Equipment workspace",show_menu:false}},
            div {class:"r0251-content",
                Input {id:"r0251-note",label:"Equipment name",value:note(),onvaluechange:move |value|note.set(value)}
                p {"Equipment details remain available when the workspace changes size."}
                div {id:"r0251-nested",
                    Application {class:"r0251-nested-app",force_breakpoint:ApplicationBreakpoint::Small,
                        header:rsx! {ApplicationHeader {name:"Local inspection",show_menu:false}},
                        p {"This nested workspace keeps its own compact layout."}
                    }
                }
            }
        }
    }
}

#[component]
pub fn ApplicationViewportReview() -> Element {
    let mut force = use_signal(|| None::<ApplicationBreakpoint>);
    let mut mounted = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let note = use_signal(|| "Pump P-101".to_string());
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    let resize_controls = {
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        {
            rsx! {
                div {class:"r0251-controls","aria-label":"Desktop window size",
                    for width in [700,1000,1360,1040] {
                        Button {id:format!("r0251-width-{width}"),variant:ButtonVariant::Secondary,
                            onclick:move |_|dioxus::desktop::window().set_inner_size(dioxus::desktop::LogicalSize::new(f64::from(width),850.0)),
                            "Window {width} px"
                        }
                    }
                }
            }
        }
        #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
        {
            VNode::empty()
        }
    };
    rsx! {
        style {{STYLES}}
        IxTheme {class:"application-viewport-review",
            h1 {"Responsive equipment workspace"}
            p {"Resize the document or choose a workspace layout. The equipment name stays in Rust, while the nested inspection and standalone pane keep independent responsive owners."}
            div {class:"r0251-controls",
                Button {id:"r0251-auto",variant:ButtonVariant::Secondary,onclick:move |_|force.set(None),"Automatic layout"}
                Button {id:"r0251-small",variant:ButtonVariant::Secondary,onclick:move |_|force.set(Some(ApplicationBreakpoint::Small)),"Compact layout"}
                Button {id:"r0251-large",variant:ButtonVariant::Secondary,onclick:move |_|force.set(Some(ApplicationBreakpoint::Large)),"Wide layout"}
                Button {id:"r0251-visible",variant:ButtonVariant::Secondary,onclick:move |_|mounted.toggle(),if mounted(){"Hide workspace"}else{"Restore workspace"}}
                Button {id:"r0251-remount",variant:ButtonVariant::Secondary,onclick:move |_|{epoch+=1;mounted.set(true);},"Remount workspace"}
            }
            {resize_controls}
            section {"aria-label":"Responsive workspace",
                p {"In the equipment field, F2 restores automatic layout, F3 chooses compact, and F4 chooses wide without changing focus."}
                div {id:"r0251-stage",onkeydown:move |event:KeyboardEvent|match event.key(){
                    Key::F2=>{event.prevent_default();force.set(None);},
                    Key::F3=>{event.prevent_default();force.set(Some(ApplicationBreakpoint::Small));},
                    Key::F4=>{event.prevent_default();force.set(Some(ApplicationBreakpoint::Large));},
                    _=>{},
                },
                    if mounted() {Workspace {key:"workspace-{epoch()}",note,force:force()}}
                    else {p {id:"r0251-removed","Workspace hidden. Other responsive surfaces remain active."}}
                }
            }
            section {
                h2 {"Independent surfaces"}
                div {class:"r0251-independent-row",
                    div {id:"r0251-independent",
                        Application {class:"r0251-independent-app",force_breakpoint:ApplicationBreakpoint::Large,
                            header:rsx! {ApplicationHeader {name:"Service overview",show_menu:false}},
                            p {"This overview keeps its wide layout independently."}
                        }
                    }
                    div {id:"r0251-pane",
                        Pane {heading:"Document-sized pane",composition:PaneComposition::Right,size:"33%",expanded:false,
                            p {"Its available size follows the document viewport."}
                        }
                    }
                }
            }
            output {id:"r0251-state",role:"status","aria-live":"polite","data-force":match force(){None=>"auto",Some(ApplicationBreakpoint::Small)=>"small",Some(ApplicationBreakpoint::Medium)=>"medium",Some(ApplicationBreakpoint::Large)=>"large"},"data-mounted":mounted().to_string(),"data-epoch":epoch().to_string(),"data-note":note(),
                "Equipment: {note()}; workspace generation: {epoch()}"
            }
            p {class:"r0251-evidence","The Linux check uses synthetic input and Dioxus window resizing. Browser verification covers ordinary typing, keyboard focus and responsive layout."}
            div {onmounted:move |_|{
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let task=super::dom_query::eval(NATIVE_VIEWPORT_CHECK);let _=task.send("viewport");
                    match task.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus native shared viewport: {value}");window.close();}
                        other=>{eprintln!("Dioxus native shared viewport failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;eprintln!("Dioxus native shared viewport timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.application-viewport-review{box-sizing:border-box;min-height:100vh;padding:24px}.application-viewport-review h1{font-size:28px;margin:0 0 16px}.application-viewport-review h2{font-size:20px;margin:0 0 16px}.application-viewport-review p{max-width:70ch;line-height:1.5}.application-viewport-review section{margin:24px 0}.r0251-controls{display:flex;flex-wrap:wrap;gap:10px;margin:16px 0;max-width:960px}#r0251-stage{height:400px;max-width:960px;border:1px solid var(--ix-color-border)}.r0251-content{padding:16px}.r0251-content .ix-field{max-width:420px}#r0251-nested{max-width:560px;height:150px}#r0251-nested .ix-application{min-height:0}#r0251-nested .ix-application-main>p{padding:12px;margin:0}.r0251-independent-row{display:flex;flex-wrap:wrap;gap:20px;max-width:960px}.r0251-independent-row>div{position:relative;box-sizing:border-box;min-width:0;max-width:100%;height:160px}#r0251-independent{width:540px}#r0251-independent .ix-application{min-height:0}#r0251-independent .ix-application-main>p{padding:12px;margin:0}#r0251-pane{width:360px;border:1px solid var(--ix-color-border);overflow:hidden}.application-viewport-review output{display:block;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}.r0251-evidence{color:var(--ix-color-text-secondary)}@media(max-width:600px){.application-viewport-review{padding:16px}#r0251-stage{height:440px}.r0251-content{padding:12px}}";

// Synthetic Linux actions; the oracle uses the upstream literal CSS media queries.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_VIEWPORT_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),state=()=>byId('r0251-state'),field=()=>byId('r0251-note');
const app=()=>byId('r0251-stage')?.querySelector(':scope > .ix-application');
const nested=()=>byId('r0251-nested')?.querySelector(':scope > .ix-application');
const independent=()=>byId('r0251-independent')?.querySelector(':scope > .ix-application');
const pane=()=>byId('r0251-pane')?.querySelector('.ix-pane');
const mode=node=>['small','medium','large'].find(value=>node?.classList.contains('ix-app-'+value));
const raw=()=>({sm:matchMedia('(max-width: 48em)').matches,md:matchMedia('(min-width: 48.0625em) and (max-width: 80em)').matches,lg:matchMedia('(min-width: 80.0625em)').matches});
const expected=()=>{const facts=raw();return facts.lg?'large':facts.md?'medium':facts.sm?'small':'small';};
const headerSmall=node=>node?.querySelector('.ix-header-layout')?.dataset.small==='true';
const paneSize=()=>Number.parseFloat(pane()?.style.getPropertyValue('--ix-pane-size'));
const near=(a,b)=>Number.isFinite(a)&&Math.abs(a-b)<=2;
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' width='+innerWidth+' app='+mode(app())+' pane='+paneSize());};
const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true}));
const shared=()=>independent()&&mode(independent())==='large'&&!headerSmall(independent())&&pane()?.dataset.mobile===(raw().sm?'true':'false')&&near(paneSize(),Math.round((window.visualViewport?.width??innerWidth)/3));
const auto=()=>app()&&mode(app())===expected()&&headerSmall(app())===(expected()==='small')&&mode(nested())==='small'&&headerSmall(nested())&&shared();
await wait('initial responsive owners',auto);
const initial=field();initial.focus();initial.value='Pump P-102';initial.dispatchEvent(new Event('input',{bubbles:true}));
await wait('edited equipment retained in Rust',()=>state().dataset.note==='Pump P-102');
const resized=[];
for(const width of [700,1000,1360,1000,1040]){
  const before=innerWidth;byId('r0251-width-'+width).click();
  await wait('settled actual media at '+width,()=>near(innerWidth,width)&&auto()&&field()===initial&&document.activeElement===initial&&field().value==='Pump P-102');
  resized.push({width:innerWidth,mode:mode(app()),queries:raw(),pane:paneSize(),changed:before!==innerWidth});
}
key(initial,'F3');await wait('compact force and independent owners',()=>state().dataset.force==='small'&&mode(app())==='small'&&headerSmall(app())&&shared());
key(initial,'F4');await wait('wide force leaves nested compact',()=>state().dataset.force==='large'&&mode(app())==='large'&&!headerSmall(app())&&mode(nested())==='small'&&shared()&&document.activeElement===initial);
key(initial,'F2');await wait('force release uses current native facts',()=>state().dataset.force==='auto'&&auto()&&document.activeElement===initial);
byId('r0251-visible').click();await wait('one owner removed',()=>!app()&&!initial.isConnected&&shared());
byId('r0251-width-700').click();await wait('remaining owners still resize',()=>near(innerWidth,700)&&shared());
byId('r0251-visible').click();await wait('restored owner uses current facts and value',()=>auto()&&field()!==initial&&field().value==='Pump P-102');
const retired=field();byId('r0251-remount').click();await wait('keyed owner replacement',()=>state().dataset.epoch==='1'&&auto()&&field()!==retired&&!retired.isConnected&&field().value==='Pump P-102');
byId('r0251-width-1000').click();await wait('final settled desktop geometry',()=>near(innerWidth,1000)&&auto());
return {ok:true,syntheticInputs:true,literalMediaOracle:true,resizes:resized,retainedValueAndFocus:true,nestedAndForcedIndependent:true,forceRelease:true,standalonePaneDimensions:true,dimensionsOnlyResize:true,ownerDropAndRemount:true};
"#;
