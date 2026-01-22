use crate::utils::read_file_to_vec;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_styling::inline_style_sheet;
use libbbf::BBFReader;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, Url, js_sys};
use xxhash_rust::xxh3::xxh3_64;

#[derive(Clone)]
struct LoadedBook {
    #[allow(dead_code)]
    name: String,
    reader: Arc<BBFReader<Arc<[u8]>>>,
}

#[component]
pub fn Reader() -> impl IntoView {
    let (book, set_book) = signal(Option::<LoadedBook>::None);
    let (page_idx, set_page_idx) = signal(0u32);
    let (img_url, set_img_url) = signal(String::new());
    let (status, set_status) = signal(String::new());

    inline_style_sheet! {
        reader_css,
        "reader",

        /* Layout Containers */
        .container {
            height: 100%;
            display: flex;
            flex-direction: column;
            color: #e2e8f0; /* text-slate-200 */
        }

        .main-content {
            flex: 1;
            display: flex;
            overflow: hidden;
        }

        /* Top Bar */
        .top-bar {
            background-color: #0f172a; /* bg-slate-900 */
            border-bottom: 1px solid #334155; /* border-slate-700 */
            padding: 1rem;
            display: flex;
            gap: 1rem;
            align-items: center;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            z-index: 10;
        }

        .spacer { flex: 1; }

        /* UI Components */
        .upload-label {
            background-color: #4f46e5; /* bg-indigo-600 */
            color: white;
            padding: 0.5rem 1rem;
            border-radius: 0.5rem;
            cursor: pointer;
            font-size: 0.875rem;
            font-weight: 500;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            transition: background-color 0.2s;
        }
        .upload-label:hover { background-color: #6366f1; }

        .status {
            color: #a5b4fc; /* text-indigo-300 */
            font-family: monospace;
            font-size: 0.875rem;
            border-left: 1px solid #334155;
            padding-left: 1rem;
        }

        .verify-btn {
            background-color: #1e293b; /* bg-slate-800 */
            border: 1px solid rgba(245, 158, 11, 0.3); /* amber-500/30 */
            color: #f59e0b; /* text-amber-500 */
            padding: 0.25rem 0.75rem;
            border-radius: 0.25rem;
            font-size: 0.875rem;
            cursor: pointer;
            transition: background-color 0.2s;
        }
        .verify-btn:hover { background-color: rgba(120, 53, 15, 0.2); }

        /* Empty State */
        .empty-state {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            height: 100%;
            color: #64748b; /* text-slate-500 */
        }
        .empty-icon { font-size: 3.75rem; margin-bottom: 1rem; opacity: 0.2; }
        .empty-text { font-size: 1.125rem; }

        /* Sidebar (Sections & Metadata) */
        .sidebar {
            width: 16rem; /* w-64 */
            background-color: #0f172a; /* bg-slate-900 */
            border-right: 1px solid #334155;
            overflow-y: auto;
            display: none; /* hidden by default on mobile */
        }

        /* Media query to show sidebar on md screens and up */
        @media (min-width: 768px) {
            .sidebar { display: block; }
        }

        .sidebar-header {
            padding: 1rem;
            background-color: #1e293b; /* bg-slate-800 */
            border-bottom: 1px solid #334155;
            font-weight: 700;
            color: #e2e8f0;
            position: sticky;
            top: 0;
        }

        .sidebar-header-meta {
            margin-top: 1rem;
            border-top: 1px solid #334155;
            border-bottom: 1px solid #334155;
        }

        .sidebar-list { padding: 0.5rem; list-style: none; margin: 0; }
        .meta-list { padding: 1rem; list-style: none; margin: 0; font-size: 0.75rem; color: #94a3b8; }

        /* Sidebar Items */
        .section-item {
            cursor: pointer;
            padding: 0.375rem 0.5rem;
            border-radius: 0.25rem;
            transition: background-color 0.2s;
            color: #94a3b8; /* Default text-slate-400 */
        }
        .section-item:hover { background-color: #1e293b; }

        /* Active State for Sections */
        .section-item[data-active="true"] {
            color: #818cf8; /* text-indigo-400 */
            background-color: #1e293b; /* Keep hover bg */
        }

        .section-title { font-weight: 500; font-size: 0.875rem; }
        .section-page { font-size: 0.75rem; opacity: 0.5; }

        .meta-item {
            display: flex;
            flex-direction: column;
            border-bottom: 1px solid #1e293b;
            padding-bottom: 0.25rem;
            margin-bottom: 0.5rem;
        }
        .meta-item:last-child { border-bottom: none; }
        .meta-key { color: #818cf8; font-weight: 700; }
        .meta-val { color: #cbd5e1; word-break: break-all; }

        /* Image Viewer Area */
        .viewer-area {
            flex: 1;
            display: flex;
            flex-direction: column;
            background-color: black;
            position: relative;
        }

        .image-container {
            flex: 1;
            display: flex;
            align-items: center;
            justify-content: center;
            overflow: auto;
            padding: 1rem;
            cursor: pointer;
        }

        .page-image {
            max-height: 100%;
            max-width: 100%;
            object-fit: contain;
            box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
        }

        /* Bottom Controls */
        .controls {
            background-color: #0f172a;
            border-top: 1px solid #334155;
            color: #e2e8f0;
            padding: 0.75rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            z-index: 20;
        }

        .nav-btn {
            padding: 0.375rem 1rem;
            background-color: #1e293b;
            border: 1px solid #475569;
            border-radius: 0.25rem;
            font-size: 0.875rem;
            color: inherit;
            cursor: pointer;
            transition: background-color 0.2s;
        }
        .nav-btn:hover { background-color: #334155; }

        .page-counter { font-family: monospace; font-size: 0.875rem; color: #a5b4fc; }
        .page-number { color: white; font-weight: 700; }
    }

    let handle_file = move |ev: web_sys::Event| {
        let target: HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(files) = target.files() {
            if let Some(file) = files.get(0) {
                let fname = file.name();
                spawn_local(async move {
                    set_status.set("Loading...".to_string());
                    match read_file_to_vec(&file).await {
                        Ok(vec) => {
                            let data_arc: Arc<[u8]> = Arc::from(vec);

                            match BBFReader::new(data_arc) {
                                Ok(r) => {
                                    set_book.set(Some(LoadedBook {
                                        name: fname,
                                        reader: Arc::new(r),
                                    }));
                                    set_page_idx.set(0);
                                    set_status.set("Loaded.".to_string());
                                }
                                Err(e) => set_status.set(format!("Invalid BBF: {:?}", e)),
                            }
                        }
                        Err(_) => set_status.set("Read error".to_string()),
                    }
                });
            }
        }
    };

    Effect::new(move |_| {
        if let Some(bk) = book.get() {
            let idx = page_idx.get();
            let pages = bk.reader.pages();
            if (idx as usize) < pages.len() {
                let page = &pages[idx as usize];
                let asset_idx = page.asset_index.get();
                if let Ok(asset_data) = bk.reader.get_asset(asset_idx) {
                    let assets = bk.reader.assets();
                    let asset_entry = &assets[asset_idx as usize];
                    let mime = libbbf::BBFMediaType::from(asset_entry.type_).as_extension();

                    let mime_str = match mime {
                        ".png" => "image/png",
                        ".jpg" | ".jpeg" => "image/jpeg",
                        ".avif" => "image/avif",
                        ".webp" => "image/webp",
                        _ => "application/octet-stream",
                    };

                    let array = js_sys::Array::new();
                    let u8arr = js_sys::Uint8Array::from(asset_data);
                    array.push(&u8arr.buffer());

                    let bag = web_sys::BlobPropertyBag::new();
                    bag.set_type(mime_str);

                    if let Ok(blob) =
                        web_sys::Blob::new_with_blob_sequence_and_options(&array, &bag)
                    {
                        if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                            let old = img_url.get_untracked();
                            if !old.is_empty() {
                                let _ = Url::revoke_object_url(&old);
                            }
                            set_img_url.set(url);
                        }
                    }
                }
            }
        }
    });

    let next_page_logic = move || {
        if let Some(bk) = book.get() {
            let max = bk.reader.pages().len() as u32;
            if page_idx.get() + 1 < max {
                set_page_idx.update(|i| *i += 1);
            }
        }
    };

    let prev_page_logic = move || {
        if page_idx.get() > 0 {
            set_page_idx.update(|i| *i -= 1);
        }
    };

    let verify_integrity = move |_| {
        spawn_local(async move {
            if let Some(bk) = book.get_untracked() {
                set_status.set("Verifying...".to_string());
                let reader = bk.reader;
                let assets = reader.assets();
                let mut bad = 0;
                for (i, asset) in assets.iter().enumerate() {
                    if let Ok(data) = reader.get_asset(i as u32) {
                        let hash = xxh3_64(data);
                        if hash != asset.xxh3_hash.get() {
                            bad += 1;
                        }
                    } else {
                        bad += 1;
                    }
                }
                if bad == 0 {
                    set_status.set("Integrity Check: OK".to_string());
                } else {
                    set_status.set(format!("Integrity Check: {} CORRUPT assets", bad));
                }
            }
        });
    };

    view! {
        <div class=reader_css::CONTAINER>
            <div class=reader_css::TOP_BAR>
                 <label class=reader_css::UPLOAD_LABEL>
                    "Open .bbf"
                    <input type="file" accept=".bbf" on:change=handle_file class="hidden" style="display:none" />
                </label>

                <span class=reader_css::STATUS>{status}</span>

                <div class=reader_css::SPACER></div>

                <Show when=move || book.get().is_some()>
                      <button
                        on:click=verify_integrity
                        class=reader_css::VERIFY_BTN
                    >
                        "Verify Integrity"
                    </button>
                </Show>
            </div>

            <Show when=move || book.get().is_some() fallback=|| view! {
                <div class=reader_css::EMPTY_STATE>
                    <div class=reader_css::EMPTY_ICON>"📖"</div>
                    <div class=reader_css::EMPTY_TEXT>"Select a BBF file to begin reading."</div>
                </div>
            }>
                <div class=reader_css::MAIN_CONTENT>
                    <div class=reader_css::SIDEBAR>
                        <div class=reader_css::SIDEBAR_HEADER>"Sections"</div>
                        <ul class=reader_css::SIDEBAR_LIST>
                            {move || {
                                book.get().map(|bk| {
                                    let reader = bk.reader.clone();
                                    let reader_for_closure = reader.clone();

                                    reader.sections().iter().enumerate().map(move |(_, s)| {
                                        let title = reader_for_closure.get_string(s.section_title_offset.get()).unwrap_or("?").to_string();
                                        let page = s.section_start_index.get();
                                        let is_active = page_idx.get() >= page;

                                        view! {
                                            <li
                                                class=reader_css::SECTION_ITEM
                                                attr:data-active=is_active.to_string()
                                                on:click=move |_| set_page_idx.set(page)
                                            >
                                                <div class=reader_css::SECTION_TITLE>{title}</div>
                                                <div class=reader_css::SECTION_PAGE>"Page " {page + 1}</div>
                                            </li>
                                        }
                                    }).collect_view()
                                })
                            }}
                        </ul>

                         <div class=format!("{} {}", reader_css::SIDEBAR_HEADER, reader_css::SIDEBAR_HEADER_META)>"Metadata"</div>

                         <ul class=reader_css::META_LIST>
                             {move || {
                                book.get().map(|bk| {
                                    let reader = bk.reader.clone();
                                    let reader_for_closure = reader.clone();

                                    reader.metadata().iter().map(move |m| {
                                        let k = reader_for_closure.get_string(m.key_offset.get()).unwrap_or("?").to_string();
                                        let v = reader_for_closure.get_string(m.val_offset.get()).unwrap_or("?").to_string();
                                        view! {
                                            <li class=reader_css::META_ITEM>
                                                <span class=reader_css::META_KEY>{k}</span>
                                                <span class=reader_css::META_VAL>{v}</span>
                                            </li>
                                        }
                                    }).collect_view()
                                })
                            }}
                         </ul>
                    </div>

                    <div class=reader_css::VIEWER_AREA>
                        <div
                            class=reader_css::IMAGE_CONTAINER
                            on:click=move |ev| {
                                 let width = web_sys::window().unwrap().inner_width().unwrap().as_f64().unwrap();
                                 let x = ev.client_x() as f64;
                                 if x > width / 2.0 { next_page_logic(); } else { prev_page_logic(); }
                            }
                        >
                            <img src=move || img_url.get() class=reader_css::PAGE_IMAGE />
                        </div>

                        <div class=reader_css::CONTROLS>
                             <button on:click=move |_| prev_page_logic() class=reader_css::NAV_BTN>
                                "Previous"
                             </button>

                             <span class=reader_css::PAGE_COUNTER>
                                "Page " <span class=reader_css::PAGE_NUMBER>{move || page_idx.get() + 1}</span>
                             </span>

                             <button on:click=move |_| next_page_logic() class=reader_css::NAV_BTN>
                                "Next"
                             </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
