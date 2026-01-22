mod app;
mod builder;
mod reader;
mod utils;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    leptos_styling::init();
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <app::App /> })
}
