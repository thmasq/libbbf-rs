use crate::utils::{download_blob, read_file_to_vec};
use leptos::prelude::*;
use leptos::task::spawn_local;
use libbbf::{BBFBuilder, BBFMediaType};
use std::io::Cursor;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};

#[derive(Clone, Debug, PartialEq)]
pub struct SendFile(pub web_sys::File);

unsafe impl Send for SendFile {}
unsafe impl Sync for SendFile {}

impl std::ops::Deref for SendFile {
    type Target = web_sys::File;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
enum BuilderEntry {
    File {
        id: usize,
        file: SendFile,
        name: String,
    },
    Section {
        id: usize,
        name: RwSignal<String>,
        parent: Option<String>,
    },
}

impl BuilderEntry {
    fn id(&self) -> usize {
        match self {
            Self::File { id, .. } => *id,
            Self::Section { id, .. } => *id,
        }
    }

    fn name(&self) -> String {
        match self {
            Self::File { name, .. } => name.clone(),
            Self::Section { name, .. } => name.get(),
        }
    }

    fn is_section(&self) -> bool {
        matches!(self, Self::Section { .. })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct MetaEntry {
    id: usize,
    key: String,
    value: String,
}

#[component]
pub fn Builder() -> impl IntoView {
    let (entries, set_entries) = signal(Vec::<BuilderEntry>::new());
    let (metadata, set_metadata) = signal(Vec::<MetaEntry>::new());
    let (status, set_status) = signal(String::new());

    let (editing_id, set_editing_id) = signal(Option::<usize>::None);
    let (drag_id, set_drag_id) = signal(Option::<usize>::None);

    let next_id = RwSignal::new(0_usize);
    let get_id = move || {
        next_id.update(|n| *n += 1);
        next_id.get_untracked()
    };

    let handle_files = move |ev: web_sys::Event| {
        let target: HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(files) = target.files() {
            let mut new_entries = Vec::new();
            for i in 0..files.length() {
                if let Some(file) = files.get(i) {
                    new_entries.push(BuilderEntry::File {
                        id: get_id(),
                        name: file.name(),
                        file: SendFile(file),
                    });
                }
            }
            set_entries.update(move |e: &mut Vec<BuilderEntry>| e.extend(new_entries));
        }
    };

    let add_section = move |_| {
        let id = get_id();
        set_entries.update(move |e: &mut Vec<BuilderEntry>| {
            e.push(BuilderEntry::Section {
                id,
                name: RwSignal::new("New Section".to_string()),
                parent: None,
            })
        });
    };

    let add_meta = move |_| {
        let id = get_id();
        set_metadata.update(move |m: &mut Vec<MetaEntry>| {
            m.push(MetaEntry {
                id,
                key: "".to_string(),
                value: "".to_string(),
            })
        });
    };

    let remove_entry = move |id: usize| {
        set_entries.update(|e| e.retain(|x| x.id() != id));
    };

    let handle_drag_start = move |id: usize| {
        set_drag_id.set(Some(id));
    };

    let handle_drop = move |target_id: usize| {
        if let Some(dragged) = drag_id.get() {
            if dragged != target_id {
                set_entries.update(|list| {
                    if let Some(from_idx) = list.iter().position(|e| e.id() == dragged) {
                        if let Some(to_idx) = list.iter().position(|e| e.id() == target_id) {
                            let item = list.remove(from_idx);
                            list.insert(to_idx, item);
                        }
                    }
                });
            }
        }
        set_drag_id.set(None);
    };

    let compile = move |_| {
        spawn_local(async move {
            set_status.set("Reading files...".to_string());
            let current_entries = entries.get();
            let current_meta = metadata.get();

            let mut cursor = Cursor::new(Vec::new());

            let mut builder = match BBFBuilder::new(&mut cursor) {
                Ok(b) => b,
                Err(err) => {
                    set_status.set(format!("Error initializing builder: {:?}", err));
                    return;
                }
            };

            let mut page_count = 0;

            for entry in current_entries {
                match entry {
                    BuilderEntry::File { file, name, .. } => match read_file_to_vec(&file).await {
                        Ok(data) => {
                            let ext = std::path::Path::new(&name)
                                .extension()
                                .and_then(|e| e.to_str())
                                .map(|e| format!(".{}", e))
                                .unwrap_or_default();

                            let media_type = BBFMediaType::from_extension(&ext);
                            if let Err(err) = builder.add_page(&data, media_type, 0) {
                                set_status.set(format!("Error adding page: {:?}", err));
                                return;
                            }
                            page_count += 1;
                        }
                        Err(_) => {
                            set_status.set("Failed to read file".to_string());
                            return;
                        }
                    },
                    BuilderEntry::Section { name, .. } => {
                        builder.add_section(&name.get(), page_count, None);
                    }
                }
            }

            for meta in current_meta {
                builder.add_metadata(&meta.key, &meta.value);
            }

            if let Err(err) = builder.finalize() {
                set_status.set(format!("Error finalizing: {:?}", err));
                return;
            }

            set_status.set("Download starting...".to_string());
            let _ = download_blob(
                cursor.get_ref(),
                "web_generated.bbf",
                "application/octet-stream",
            );
            set_status.set("Done!".to_string());
        });
    };

    view! {
        <div class="p-6 h-full overflow-y-auto">
            <h2 class="text-2xl font-bold mb-6 text-indigo-400">"BBF Builder Mode"</h2>

            <div class="bg-slate-900 border border-slate-700 rounded-xl p-6 shadow-xl mb-6">
                <div class="mb-4">
                    <label class="block mb-2 text-sm font-medium text-slate-300">"Add Files"</label>
                    <input
                        type="file"
                        multiple
                        on:change=handle_files
                        class="block w-full text-sm text-slate-400
                            file:mr-4 file:py-2 file:px-4
                            file:rounded-full file:border-0
                            file:text-sm file:font-semibold
                            file:bg-indigo-900 file:text-indigo-300
                            hover:file:bg-indigo-800
                            cursor-pointer" 
                    />
                </div>

                <div class="flex gap-4 mb-4">
                    <button
                        on:click=add_section
                        class="bg-slate-800 border border-slate-600 hover:bg-slate-700 text-indigo-300 px-4 py-2 rounded-lg transition-colors flex items-center gap-2"
                    >
                        <span>"Add Section Marker"</span>
                    </button>
                    <button
                        on:click=add_meta
                        class="bg-slate-800 border border-slate-600 hover:bg-slate-700 text-emerald-400 px-4 py-2 rounded-lg transition-colors flex items-center gap-2"
                    >
                        <span>"Add Metadata"</span>
                    </button>
                </div>
            </div>

            <div class="flex flex-col md:flex-row gap-6">
                <div class="w-full md:w-1/2 bg-slate-900 border border-slate-700 rounded-xl p-4 shadow-xl">
                    <h3 class="font-bold mb-4 text-slate-200 border-b border-slate-700 pb-2">"Content Order"</h3>
                    <div class="space-y-2 max-h-[500px] overflow-y-auto pr-2">
                        <For
                            each=move || entries.get()
                            key=|e| e.id()
                            children=move |e| {
                                let id = e.id();
                                let is_section = e.is_section();

                                let is_editing = move || editing_id.get() == Some(id);
                                let is_dragging = move || drag_id.get() == Some(id);

                                view! {
                                    <div
                                        class="p-3 rounded-lg border flex items-center justify-between group cursor-move transition-all select-none"
                                        class:bg-slate-800=move || !is_dragging()
                                        class:bg-slate-700=move || is_dragging()
                                        class:opacity-50=move || is_dragging()
                                        class:border-indigo-500=move || is_editing()
                                        class:border-slate-700=move || !is_editing()

                                        draggable=move || if is_editing() { "false" } else { "true" }
                                        on:dragstart=move |_| handle_drag_start(id)
                                        on:dragover=move |ev: web_sys::DragEvent| ev.prevent_default()
                                        on:drop=move |ev: web_sys::DragEvent| {
                                            ev.prevent_default();
                                            handle_drop(id);
                                        }
                                        on:dblclick=move |_| {
                                            if is_section {
                                                set_editing_id.set(Some(id));
                                            }
                                        }
                                    >
                                        <div class="flex items-center flex-1 gap-3 min-w-0">
                                            <span class="text-xl flex-shrink-0">
                                                {match e {
                                                    BuilderEntry::File { .. } => "📄",
                                                    BuilderEntry::Section { .. } => "🔖",
                                                }}
                                            </span>

                                            <div class="flex-1 min-w-0">
                                            {move || {
                                                if is_editing() {
                                                    match e {
                                                        BuilderEntry::Section { name, .. } => {
                                                            view! {
                                                                <input
                                                                    type="text"
                                                                    class="bg-slate-900 text-white p-1 border-b border-indigo-500 focus:outline-none w-full"
                                                                    prop:value=move || name.get()
                                                                    autofocus
                                                                    on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                                                    on:keydown=move |ev: KeyboardEvent| {
                                                                        if ev.key() == "Enter" {
                                                                            ev.prevent_default();
                                                                            let val = event_target_value(&ev);
                                                                            name.set(val);
                                                                            set_editing_id.set(None);
                                                                        } else if ev.key() == "Escape" {
                                                                            set_editing_id.set(None);
                                                                        }
                                                                    }
                                                                    on:blur=move |_| set_editing_id.set(None)
                                                                />
                                                            }.into_any()
                                                        },
                                                        _ => view! {}.into_any()
                                                    }
                                                } else {
                                                    let display_name = e.name();
                                                    view! {
                                                        <span class="text-slate-300 block truncate" title=display_name.clone()>
                                                            {display_name.clone()}
                                                        </span>
                                                    }.into_any()
                                                }
                                            }}
                                            </div>
                                        </div>

                                        <button
                                            class="text-slate-500 hover:text-red-400 p-2 ml-2 opacity-0 group-hover:opacity-100 transition-opacity"
                                            title="Remove"
                                            on:click=move |ev: web_sys::MouseEvent| {
                                                ev.stop_propagation();
                                                remove_entry(id);
                                            }
                                        >
                                            "✕"
                                        </button>
                                    </div>
                                }
                            }
                        />
                         <Show when=move || entries.get().is_empty()>
                            <div class="text-slate-500 text-center py-8 italic">"No files added yet."</div>
                         </Show>
                    </div>
                </div>

                <div class="w-full md:w-1/2 bg-slate-900 border border-slate-700 rounded-xl p-4 shadow-xl">
                    <h3 class="font-bold mb-4 text-slate-200 border-b border-slate-700 pb-2">"Metadata"</h3>
                     <div class="space-y-2">
                         <For
                            each=move || metadata.get()
                            key=|m| m.id
                            children=move |m| {
                                view! {
                                    <div class="flex gap-2 group">
                                        <input class="bg-slate-800 border border-slate-600 rounded p-2 w-1/3 text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                               placeholder="Key"
                                               prop:value=m.key
                                               on:input=move |ev| {
                                                   let val = event_target_value(&ev);
                                                   set_metadata.update(|list| {
                                                       if let Some(item) = list.iter_mut().find(|i| i.id == m.id) {
                                                           item.key = val;
                                                       }
                                                   });
                                               }
                                        />
                                        <input class="bg-slate-800 border border-slate-600 rounded p-2 w-full text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                               placeholder="Value"
                                               prop:value=m.value
                                               on:input=move |ev| {
                                                   let val = event_target_value(&ev);
                                                   set_metadata.update(|list| {
                                                       if let Some(item) = list.iter_mut().find(|i| i.id == m.id) {
                                                           item.value = val;
                                                       }
                                                   });
                                               }
                                        />
                                        <button
                                            class="text-slate-500 hover:text-red-400 px-2 opacity-0 group-hover:opacity-100 transition-opacity"
                                            on:click=move |_| set_metadata.update(|list| list.retain(|x| x.id != m.id))
                                        >
                                            "✕"
                                        </button>
                                    </div>
                                }
                            }
                        />
                        <Show when=move || metadata.get().is_empty()>
                            <div class="text-slate-500 text-center py-8 italic">"No metadata."</div>
                         </Show>
                    </div>
                </div>
            </div>

            <div class="mt-8 flex items-center justify-between bg-slate-900 border border-slate-700 rounded-xl p-4">
                <div class="text-indigo-400 font-mono">{status}</div>
                <button
                    on:click=compile
                    class="bg-indigo-600 hover:bg-indigo-500 text-white px-8 py-3 rounded-lg text-lg font-bold shadow-[0_0_20px_rgba(79,70,229,0.4)] hover:shadow-[0_0_30px_rgba(79,70,229,0.6)] transition-all transform hover:-translate-y-0.5"
                >
                    "Compile & Download .bbf"
                </button>
            </div>
        </div>
    }
}
