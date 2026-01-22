use wasm_bindgen::prelude::*;
use web_sys::{Blob, File, FileReader, js_sys};

pub async fn read_file_to_vec(file: &File) -> Result<Vec<u8>, JsValue> {
    let reader = FileReader::new()?;
    let reader_c = reader.clone();

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let onload = Closure::once(Box::new(move || {
            let _ = resolve.call0(&JsValue::NULL);
        }));

        let value = reject.clone();
        let onerror = Closure::once(Box::new(move || {
            let _ = value.call0(&JsValue::NULL);
        }));

        reader_c.set_onload(Some(onload.as_ref().unchecked_ref()));
        reader_c.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        if reader_c.read_as_array_buffer(file).is_err() {
            let _ = reject.call0(&JsValue::NULL);
        }

        onload.forget();
        onerror.forget();
    });

    wasm_bindgen_futures::JsFuture::from(promise).await?;

    let array_buffer = reader.result()?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let mut vec = vec![0; uint8_array.length() as usize];
    uint8_array.copy_to(&mut vec);

    Ok(vec)
}

pub fn download_blob(data: &[u8], filename: &str, mime: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    let uint8arr = js_sys::Uint8Array::from(data);
    let array = js_sys::Array::new();
    array.push(&uint8arr.buffer());

    let bag = web_sys::BlobPropertyBag::new();
    bag.set_type(mime);
    let blob = Blob::new_with_blob_sequence_and_options(&array, &bag)?;

    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    let a = document
        .create_element("a")?
        .dyn_into::<web_sys::HtmlAnchorElement>()?;

    a.set_href(&url);
    a.set_download(filename);
    a.style().set_property("display", "none")?;

    body.append_child(&a)?;
    a.click();
    body.remove_child(&a)?;
    web_sys::Url::revoke_object_url(&url)?;

    Ok(())
}
