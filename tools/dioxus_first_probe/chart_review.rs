//! Excluded, bounded review of Dioxus-owned chart sizing and interactions.
use dioxus::prelude::*;
use dioxus_ix::*;

fn review_surface() -> ChartSurface {
    // The gallery's source-backed sine surface and central hole, on a lighter grid.
    ChartSurface::from_equation(21, |x, y| {
        if x.abs() < 0.1 && y.abs() < 0.1 {
            None
        } else {
            Some((x * std::f64::consts::PI).sin() * (y * std::f64::consts::PI).sin())
        }
    })
}

fn review_series() -> (Vec<f64>, Vec<String>) {
    let values = (0..20_000)
        .map(|index| {
            let x = index as f64;
            60.0 + (x / 170.0).sin() * 20.0 + (x / 37.0).cos() * 5.0 + x / 1500.0
        })
        .collect();
    let labels = (0..20_000).map(|index| format!("Sample {index}")).collect();
    (values, labels)
}

#[component]
pub fn ChartsPreview() -> Element {
    let mut dark = use_signal(|| false);
    let mut narrow = use_signal(|| false);
    let mut mounted = use_signal(|| true);
    let mut export_busy = use_signal(|| false);
    let mut export_status = use_signal(|| "No export yet".to_string());
    let mut export_width = use_signal(|| 0_u32);
    let mut export_height = use_signal(|| 0_u32);
    let mut export_bytes = use_signal(|| 0_usize);
    let mut export_url = use_signal(String::new);
    let surface = use_hook(review_surface);
    let series = use_hook(review_series);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
            eprintln!("Chart review native check timed out");
            std::process::exit(1);
        });
    });
    rsx! {
        style { {STYLES} }
        IxTheme { scheme: if dark() { ColorScheme::Dark } else { ColorScheme::Light }, class: "r025-chart-review",
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = super::dom_query::eval(NATIVE_CHECK);
                        let _ = query.send("charts");
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus chart review native: {value}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus chart review native failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                }
            },
                h1 { "Dioxus chart interactions" }
                p { "Responsive plots, surface inspection and a sampled 20,000-point series." }
                div { class: "r025-controls",
                    button { id: "r025-resize", r#type: "button", onclick: move |_| narrow.toggle(), "Resize charts" }
                    button { id: "r025-theme", r#type: "button", onclick: move |_| dark.toggle(), "Switch theme" }
                    button { id: "r025-mount", r#type: "button", onclick: move |_| mounted.toggle(), "Mount/unmount advanced charts" }
                    button { id: "r025-export", r#type: "button", disabled: export_busy(), onclick: move |_| {
                        export_busy.set(true);
                        export_status.set("Exporting…".to_string());
                        spawn(async move {
                            match export_chart_png("r025-cartesian", 1.0).await {
                                Ok(png) => match png.bytes() {
                                    Ok(bytes) => {
                                        export_width.set(png.width);
                                        export_height.set(png.height);
                                        export_bytes.set(bytes.len());
                                        export_status.set(format!("PNG: {} × {} pixels, {} bytes", png.width, png.height, bytes.len()));
                                        export_url.set(png.data_url);
                                    }
                                    Err(error) => export_status.set(format!("Export failed: {error}")),
                                },
                                Err(error) => export_status.set(format!("Export failed: {error}")),
                            }
                            export_busy.set(false);
                        });
                    }, "Export PNG" }
                }
                output { id: "r025-export-result", "data-width": export_width(), "data-height": export_height(), "data-bytes": export_bytes(), "{export_status}" }
            }
            div { id: "r025-chart-frame", style: if narrow() { "width:300px;max-width:100%" } else { "width:800px;max-width:100%" },
                if mounted() {
                    section { id: "r025-surface", class: "r025-section",
                        h2 { "Surface inspection" }
                        SurfaceChart { surface: surface.clone(), height: 320.0, label: "Sine surface with a central hole" }
                    }
                    section { id: "r025-zoom", class: "r025-section",
                        h2 { "Sampled history" }
                        ZoomChart { values: series.0.clone(), labels: series.1.clone(), label: "20,000 production samples" }
                    }
                }
                section { class: "r025-section",
                    h2 { "Production trend" }
                    div { id: "r025-cartesian",
                        CartesianChart {
                            label: "Weekly production", categories: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map(str::to_string).to_vec(),
                            series: vec![ChartSeries::new("Produced", vec![42.0, 51.0, 47.0, 65.0, 58.0, 61.0, 72.0], ChartKind::Line)],
                            y_title: "Units", axis_tooltip: true, brush_enabled: true,
                        }
                    }
                }
                section { class: "r025-section",
                    h2 { "Output mix" }
                    PieChart { label: "Output by line", donut: true, slices: vec![
                        ChartSlice { name: "Line A".into(), value: 42.0 },
                        ChartSlice { name: "Line B".into(), value: 35.0 },
                        ChartSlice { name: "Line C".into(), value: 23.0 },
                    ] }
                }
                section { class: "r025-section",
                    h2 { "Completion" }
                    GaugeChart { value: 72.0, variant: GaugeVariant::Arc }
                }
                if !export_url().is_empty() {
                    section { class: "r025-section",
                        h2 { "Export preview" }
                        img { id: "r025-export-image", src: export_url(), alt: "Exported weekly production chart", style: "display:block;max-width:100%;height:auto" }
                    }
                }
            }
        }
    }
}

const STYLES: &str = r#"
body{margin:0}.r025-chart-review{padding:24px;min-height:100vh;box-sizing:border-box}.r025-chart-review h1{font-size:28px;margin:0 0 12px}.r025-chart-review h2{font-size:20px;margin:0 0 16px}.r025-chart-review p{max-width:65ch}.r025-controls{display:flex;flex-wrap:wrap;gap:12px;margin:20px 0 16px}.r025-controls button{font:inherit;border:1px solid var(--ix-color-border);background:var(--ix-color-surface-1);color:var(--ix-color-text);padding:8px 14px;border-radius:2px;cursor:pointer}.r025-controls button:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.r025-controls button:disabled{opacity:.5;cursor:wait}.r025-section{margin:32px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border);min-width:0}.r025-chart-review output{display:block;overflow-wrap:anywhere}#r025-chart-frame{min-width:0}@media(max-width:600px){.r025-chart-review{padding:16px}}
"#;

// Development-only synthetic DOM driver. Real browser pointer dragging is
// verified separately; this driver never fakes capture or claims physical input.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CHECK: &str = r#"
await dioxus.recv();
const wait=async(label,predicate)=>{for(let n=0;n<250;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' '+JSON.stringify({frame:document.querySelector('#r025-chart-frame')?.getBoundingClientRect().width,surface:document.querySelector('#r025-surface svg[id^="ix-surface-svg-"]')?.getAttribute('viewBox'),zoom:document.querySelector('#r025-zoom svg[id^="ix-zoom-svg-"]')?.getAttribute('viewBox'),output:document.querySelector('#r025-export-result')?.textContent}));};
const surface=()=>document.querySelector('#r025-surface svg[id^="ix-surface-svg-"]');
const zoom=()=>document.querySelector('#r025-zoom svg[id^="ix-zoom-svg-"]');
const measured=()=>[surface(),zoom()].every(svg=>{if(!svg)return false;const r=svg.getBoundingClientRect(),p=svg.parentElement.getBoundingClientRect();return Math.abs(svg.viewBox.baseVal.width-r.width)<1&&r.width>100&&['left','top','width','height'].every(key=>Math.abs(r[key]-p[key])<1);});
const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true}));
await wait('initial framework measurements',measured);
const initialWidth=surface().viewBox.baseVal.width;
document.getElementById('r025-resize').click();
await wait('shrink via Rust state',()=>measured()&&surface().viewBox.baseVal.width<initialWidth-100);
const narrowWidth=surface().viewBox.baseVal.width;
document.getElementById('r025-resize').click();
await wait('widen via Rust state',()=>measured()&&surface().viewBox.baseVal.width>narrowWidth+100);
const face=surface().querySelector('.ix-surface-face:last-of-type');
if(!face)throw new Error('Surface faces missing');
const points=face.getAttribute('points').trim().split(/\s+/).map(point=>point.split(',').map(Number));
const center=points.reduce((sum,point)=>[sum[0]+point[0]/points.length,sum[1]+point[1]/points.length],[0,0]);
const sr=surface().getBoundingClientRect(),sv=surface().viewBox.baseVal;
face.dispatchEvent(new PointerEvent('pointermove',{pointerId:99,pointerType:'mouse',bubbles:true,clientX:sr.left+center[0]*sr.width/sv.width,clientY:sr.top+center[1]*sr.height/sv.height}));
await wait('surface inspection at visible face',()=>!!document.querySelector('#r025-surface .ix-surface-axis-pointer[data-source-index]')&&document.querySelector('#r025-surface output').textContent.includes('z:'));
const inspected=document.querySelector('#r025-surface output').textContent;
const initialYaw=+surface().dataset.yaw;
key(surface(),'ArrowRight');
await wait('surface keyboard rotation',()=>+surface().dataset.yaw!==initialYaw);
key(surface(),'Home');
await wait('surface keyboard reset',()=>+surface().dataset.yaw===initialYaw&&+surface().dataset.zoom===1);
const handles=()=>[...zoom().querySelectorAll('[role=slider]')];
const range=()=>handles().map(node=>+node.getAttribute('aria-valuenow'));
key(handles()[0],'ArrowRight');
await wait('range handle keyboard adjustment',()=>range()[0]>0);
key(handles()[0],'Home');
await wait('range keyboard reset',()=>range()[0]===0&&range()[1]===10);
const wheelAt=(node,x,y)=>{const svg=zoom(),r=svg.getBoundingClientRect(),v=svg.viewBox.baseVal;node.dispatchEvent(new WheelEvent('wheel',{bubbles:true,cancelable:true,deltaY:-200,clientX:r.left+x*r.width/v.width,clientY:r.top+y*r.height/v.height}));};
const selection=zoom().querySelector('.ix-zoom-selection');
wheelAt(selection,+selection.getAttribute('x')+(+selection.getAttribute('width'))*.5,568);
await wait('overview child wheel coordinates',()=>range()[1]-range()[0]<9);
const overviewRange=range();
key(zoom(),'Home');
await wait('reset before plot wheel',()=>range()[0]===0&&range()[1]===10);
wheelAt(zoom().querySelector('path'),zoom().viewBox.baseVal.width*.65,240);
await wait('plot child wheel coordinates',()=>range()[1]-range()[0]<9&&range()[0]>0);
const plotRange=range();
document.getElementById('r025-theme').click();
document.getElementById('r025-mount').click();
await wait('unmount advanced charts',()=>!surface()&&!zoom());
document.getElementById('r025-mount').click();
await wait('remount chart ownership and measurements',measured);
document.getElementById('r025-export').click();
await wait('validated Rust PNG export',()=>+document.getElementById('r025-export-result').dataset.bytes>33&&document.getElementById('r025-export-image')?.naturalWidth>0&&document.getElementById('r025-export-image').naturalWidth>0);
const output=document.getElementById('r025-export-result');
const width=+output.dataset.width,height=+output.dataset.height,bytes=+output.dataset.bytes;
if(width<300||height<100||!document.getElementById('r025-export-image').src.startsWith('data:image/png;base64,'))throw new Error('Invalid PNG result');
return {ok:true,initialWidth,narrowWidth,inspected,keyboardRotation:true,handleKeyboard:true,overviewRange,plotRange,remount:true,htmlViewportMatchesSvg:true,png:{width,height,bytes}};
"#;
