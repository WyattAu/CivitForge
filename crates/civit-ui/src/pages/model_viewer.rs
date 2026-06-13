#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::components::{Button, ButtonVariant, Card, Spinner};
use crate::state::auth::use_auth;

#[derive(Debug, Clone, PartialEq)]
enum ModelFormat {
    Stl,
    Obj,
    Gltf,
    Unknown,
}

impl ModelFormat {
    fn from_path(path: &str) -> Self {
        let lower = path.to_lowercase();
        if lower.ends_with(".stl") {
            Self::Stl
        } else if lower.ends_with(".obj") {
            Self::Obj
        } else if lower.ends_with(".gltf") || lower.ends_with(".glb") {
            Self::Gltf
        } else {
            Self::Unknown
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Stl => "STL",
            Self::Obj => "OBJ",
            Self::Gltf => "GLTF",
            Self::Unknown => "Unknown",
        }
    }

    fn supported() -> &'static str {
        "STL, OBJ, GLTF/GLB"
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ViewerState {
    Idle,
    Loading,
    Loaded,
    Error(String),
}

#[component]
fn ThreeJsViewer(
    model_url: Signal<String>,
    wireframe: Signal<bool>,
    auto_rotate: Signal<bool>,
) -> impl IntoView {
    let canvas_id = "threejs-canvas";

    // Load Three.js
    Effect::new(move |_| {
        let url = model_url.get();
        if url.is_empty() {
            return;
        }
        leptos::task::spawn_local(async move {
            let _ = js_sys::eval(
                r#"
                (function() {
                    if (typeof window.__civit_three_loaded !== 'undefined') return;
                    window.__civit_three_loaded = true;
                    var s = document.createElement('script');
                    s.src = 'https://cdn.jsdelivr.net/npm/three@0.160.0/build/three.min.js';
                    s.onload = function() {
                        var c = document.createElement('script');
                        c.src = 'https://cdn.jsdelivr.net/npm/three@0.160.0/examples/js/controls/OrbitControls.js';
                        c.onload = function() { window.__civit_three_ready = true; };
                        document.head.appendChild(c);
                    };
                    document.head.appendChild(s);
                })();
                "#,
            );
        });
    });

    // Load model when ready
    let model_url_load = model_url.clone();
    Effect::new(move |_| {
        let url = model_url_load.get();
        if url.is_empty() {
            return;
        }
        leptos::task::spawn_local(async move {
            for _ in 0..50 {
                let _ = js_sys::eval("void 0");
                gloo_timers::future::TimeoutFuture::new(200).await;
                let ready = js_sys::eval("typeof window.__civit_three_ready !== 'undefined' && window.__civit_three_ready")
                    .map(|v| v.as_bool().unwrap_or(false))
                    .unwrap_or(false);
                if ready {
                    break;
                }
            }
            let _ = js_sys::eval(&format!(
                r#"
                (function() {{
                    try {{
                        var container = document.getElementById('{cid}');
                        if (!container) return;
                        if (window.__civit_renderer) {{
                            window.__civit_renderer.dispose();
                            container.innerHTML = '';
                        }}
                        var w = container.clientWidth || 800;
                        var h = container.clientHeight || 600;
                        var scene = new THREE.Scene();
                        scene.background = new THREE.Color(0x1a1a2e);
                        var camera = new THREE.PerspectiveCamera(75, w / h, 0.1, 1000);
                        camera.position.set(0, 2, 5);
                        var renderer = new THREE.WebGLRenderer({{ antialias: true }});
                        renderer.setSize(w, h);
                        renderer.setPixelRatio(window.devicePixelRatio);
                        container.appendChild(renderer.domElement);
                        var controls = new THREE.OrbitControls(camera, renderer.domElement);
                        controls.enableDamping = true;
                        scene.add(new THREE.AmbientLight(0xffffff, 0.6));
                        var dl = new THREE.DirectionalLight(0xffffff, 0.8);
                        dl.position.set(5, 5, 5);
                        scene.add(dl);
                        scene.add(new THREE.GridHelper(10, 10, 0x444444, 0x222222));
                        scene.add(new THREE.AxesHelper(3));
                        var url = '{url}';
                        var ext = url.split('?')[0].split('.').pop().toLowerCase();
                        var loader;
                        if (ext === 'stl') loader = new THREE.STLLoader();
                        else if (ext === 'obj') loader = new THREE.OBJLoader();
                        else if (ext === 'gltf' || ext === 'glb') loader = new THREE.GLTFLoader();
                        if (!loader) return;
                        loader.load(url, function(result) {{
                            var mesh;
                            if (ext === 'stl') {{
                                var geo = result;
                                var mat = new THREE.MeshPhongMaterial({{ color: 0x4a90d9, wireframe: false, side: THREE.DoubleSide }});
                                mesh = new THREE.Mesh(geo, mat);
                            }} else if (ext === 'obj') {{
                                mesh = result;
                                mesh.traverse(function(c) {{ if (c.isMesh) c.material = new THREE.MeshPhongMaterial({{ color: 0x4a90d9 }}); }});
                            }} else {{
                                mesh = result.scene || result;
                            }}
                            if (mesh) {{
                                var box = new THREE.Box3().setFromObject(mesh);
                                var center = box.getCenter(new THREE.Vector3());
                                var size = box.getSize(new THREE.Vector3());
                                var maxDim = Math.max(size.x, size.y, size.z);
                                mesh.scale.multiplyScalar(3 / maxDim);
                                mesh.position.sub(center.multiplyScalar(3 / maxDim));
                                scene.add(mesh);
                                window.__civit_mesh = mesh;
                            }}
                            window.__civit_scene = scene;
                            window.__civit_camera = camera;
                            window.__civit_renderer = renderer;
                            window.__civit_controls = controls;
                            function animate() {{ requestAnimationFrame(animate); controls.update(); renderer.render(scene, camera); }}
                            animate();
                        }});
                    }} catch(e) {{ console.error(e); }}
                }})();
                "#,
                cid = canvas_id,
                url = url,
            ));
        });
    });

    // Update wireframe
    let wf = wireframe.clone();
    Effect::new(move |_| {
        let w = wf.get();
        let _ = js_sys::eval(&format!(
            "if(window.__civit_mesh)window.__civit_mesh.traverse(function(c){{if(c.isMesh&&c.material)c.material.wireframe={w};}});",
            w = w
        ));
    });

    // Update auto-rotate
    let ar = auto_rotate.clone();
    Effect::new(move |_| {
        let a = ar.get();
        let _ = js_sys::eval(&format!(
            "if(window.__civit_controls)window.__civit_controls.autoRotate={a};",
            a = a
        ));
    });

    view! {
        <div
            id=canvas_id
            class="w-full rounded-lg overflow-hidden bg-gray-900 border border-gray-700"
            style="height: 500px; min-height: 400px;"
        ></div>
    }
}

#[component]
pub fn ModelViewerPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let path = move || params.with(|p| p.get("path").unwrap_or_default());
    let _auth = use_auth();

    let (model_url, set_model_url) = signal(String::new());
    let (wireframe, set_wireframe) = signal(false);
    let (auto_rotate, set_auto_rotate) = signal(true);
    let (custom_url, set_custom_url) = signal(String::new());

    let model_format = move || ModelFormat::from_path(&path());

    Effect::new(move |_| {
        let p = path();
        if !p.is_empty() {
            let owner_val = owner();
            let name_val = name();
            let file_url = format!("/repos/{owner_val}/{name_val}/raw/HEAD/{p}");
            set_model_url.set(file_url);
        }
    });

    let handle_custom_url = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let url = custom_url.get();
        if !url.trim().is_empty() {
            set_model_url.set(url.trim().to_string());
        }
    };

    let download_model = move || {
        let url = model_url.get();
        if !url.is_empty() {
            let _ = js_sys::eval(&format!("window.open('{}', '_blank');", url.replace('\'', "\\'")));
        }
    };

    let reset_view = move || {
        let _ = js_sys::eval(
            "if(window.__civit_controls&&window.__civit_camera){window.__civit_controls.reset();window.__civit_camera.position.set(0,2,5);}",
        );
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                    <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <A href=format!("/repos/{}/{}", owner(), name())>
                            <span class="hover:text-blue-600 dark:hover:text-blue-400">
                                {move || format!("{}/{}", owner(), name())}
                            </span>
                        </A>
                        <span class="hidden sm:inline">"/"</span>
                        <span class="hidden sm:inline text-gray-700 dark:text-gray-300">"Model Viewer"</span>
                    </div>
                    <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-gray-100">"3D Model Viewer"</h1>
                </div>
            </div>

            <Card title="3D Viewer".to_string()>
                <div class="space-y-4">
                    <div class="flex flex-wrap items-center gap-3">
                        <div class="flex items-center gap-2">
                            <input
                                type="checkbox"
                                id="wireframe-toggle"
                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                prop:checked=wireframe
                                on:change=move |ev| set_wireframe.set(event_target_checked(&ev))
                            />
                            <label for="wireframe-toggle" class="text-sm text-gray-700 dark:text-gray-300">"Wireframe"</label>
                        </div>
                        <div class="flex items-center gap-2">
                            <input
                                type="checkbox"
                                id="auto-rotate-toggle"
                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                prop:checked=auto_rotate
                                on:change=move |ev| set_auto_rotate.set(event_target_checked(&ev))
                            />
                            <label for="auto-rotate-toggle" class="text-sm text-gray-700 dark:text-gray-300">"Auto-rotate"</label>
                        </div>
                        <Button variant=ButtonVariant::Secondary on:click=move |_| reset_view()>
                            "Reset View"
                        </Button>
                        <Button variant=ButtonVariant::Secondary on:click=move |_| download_model()>
                            "Download"
                        </Button>
                    </div>

                    {move || if model_url.get().is_empty() {
                        view! {
                            <div class="flex flex-col items-center justify-center h-96 bg-gray-900 rounded-lg text-gray-400">
                                <svg class="w-16 h-16 mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1v-2.5M6 18l-2-1v-2.5M18 18l2-1v-2.5"/>
                                </svg>
                                <p class="text-lg font-medium">"No model loaded"</p>
                                <p class="text-sm mt-1">"Navigate to a 3D file or enter a URL below"</p>
                                <p class="text-xs mt-2 text-gray-500">"Supported: {ModelFormat::supported()}"</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <ThreeJsViewer
                                model_url=model_url
                                wireframe=wireframe
                                auto_rotate=auto_rotate
                            />
                        }.into_any()
                    }}

                    <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                        <form on:submit=handle_custom_url class="flex gap-2">
                            <input
                                type="text"
                                class="flex-1 px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                placeholder="Or enter a direct model URL..."
                                prop:value=custom_url
                                on:input=move |ev| set_custom_url.set(event_target_value(&ev))
                            />
                            <Button variant=ButtonVariant::Primary>"Load"</Button>
                        </form>
                    </div>

                    <div class="text-xs text-gray-500 dark:text-gray-400 space-y-1">
                        <p>"Controls: Left-click drag to rotate, scroll to zoom, right-click drag to pan"</p>
                        <p>"Formats: STL, OBJ, GLTF/GLB (loaded via Three.js from CDN)"</p>
                    </div>
                </div>
            </Card>
        </div>
    }
}
