use crate::utils::read_file_to_vec;
use leptos::prelude::*;
use leptos::task::spawn_local;
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
        <div class="p-4 h-full flex flex-col">
            <h2 class="text-xl font-bold mb-2">"BBF Reader"</h2>
            <div class="mb-4 flex gap-4 items-center">
                <input type="file" accept=".bbf" on:change=handle_file class="border p-1" />
                <span class="text-blue-600">{status}</span>
                <Show when=move || book.get().is_some()>
                     <button on:click=verify_integrity class="bg-yellow-500 text-white px-3 py-1 rounded">"Verify Integrity"</button>
                </Show>
            </div>

            <Show when=move || book.get().is_some() fallback=|| view! { <div>"Select a BBF file to begin."</div> }>
                <div class="flex flex-1 overflow-hidden border">
                    <div class="w-64 bg-gray-50 border-r p-2 overflow-y-auto hidden md:block">
                        <h3 class="font-bold border-b mb-2">"Sections"</h3>
                        <ul class="text-sm">
                            {move || {
                                book.get().map(|bk| {
                                    let reader = bk.reader.clone();
                                    let reader_for_closure = reader.clone();

                                    reader.sections().iter().enumerate().map(move |(_, s)| {
                                        let title = reader_for_closure.get_string(s.section_title_offset.get()).unwrap_or("?").to_string();
                                        let page = s.section_start_index.get();
                                        view! {
                                            <li class="cursor-pointer hover:text-blue-500 mb-1"
                                                on:click=move |_| set_page_idx.set(page)>
                                                {title} <span class="text-gray-400 text-xs">"(p. " {page + 1} ")"</span>
                                            </li>
                                        }
                                    }).collect_view()
                                })
                            }}
                        </ul>
                         <h3 class="font-bold border-b mt-4 mb-2">"Metadata"</h3>
                         <ul class="text-xs text-gray-600">
                             {move || {
                                book.get().map(|bk| {
                                    let reader = bk.reader.clone();
                                    let reader_for_closure = reader.clone();

                                    reader.metadata().iter().map(move |m| {
                                        let k = reader_for_closure.get_string(m.key_offset.get()).unwrap_or("?").to_string();
                                        let v = reader_for_closure.get_string(m.val_offset.get()).unwrap_or("?").to_string();
                                        view! { <li><b>{k}</b> ": " {v}</li> }
                                    }).collect_view()
                                })
                            }}
                         </ul>
                    </div>

                    <div class="flex-1 flex flex-col bg-gray-800 relative">
                        <div class="flex-1 flex items-center justify-center overflow-auto p-4"
                             on:click=move |ev| {
                                 let width = web_sys::window().unwrap().inner_width().unwrap().as_f64().unwrap();
                                 let x = ev.client_x() as f64;
                                 if x > width / 2.0 { next_page_logic(); } else { prev_page_logic(); }
                             }>
                             <img src=move || img_url.get() class="max-h-full max-w-full shadow-lg object-contain" />
                        </div>

                        <div class="bg-black bg-opacity-75 text-white p-2 flex justify-between items-center">
                             <button on:click=move |_| prev_page_logic() class="px-4 py-1 bg-gray-700 hover:bg-gray-600 rounded">"Previous"</button>
                             <span>"Page " {move || page_idx.get() + 1}</span>
                             <button on:click=move |_| next_page_logic() class="px-4 py-1 bg-gray-700 hover:bg-gray-600 rounded">"Next"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
