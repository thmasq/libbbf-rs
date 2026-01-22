use crate::builder::BBFBuilder;
use crate::format::BBFMediaType;
use crate::reader::BBFReader;
use std::fs::File;
use std::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum BbfError {
    #[error("IO Error: {0}")]
    Io(String),
    #[error("Parse Error: {0}")]
    Parse(String),
    #[error("Object already finalized")]
    AlreadyFinalized,
}

impl From<crate::reader::BBFError> for BbfError {
    fn from(e: crate::reader::BBFError) -> Self {
        Self::Parse(e.to_string())
    }
}

pub struct BbfBuilder {
    inner: Mutex<Option<BBFBuilder<File>>>,
}

impl BbfBuilder {
    pub fn new(path: String) -> Result<Self, BbfError> {
        let file = File::create(path).map_err(|e| BbfError::Io(e.to_string()))?;
        let builder = BBFBuilder::new(file).map_err(|e| BbfError::Io(e.to_string()))?;
        Ok(Self {
            inner: Mutex::new(Some(builder)),
        })
    }

    pub fn add_page(
        &self,
        data: Vec<u8>,
        media_type: MediaType,
        flags: u32,
    ) -> Result<u32, BbfError> {
        let mut guard = self.inner.lock().unwrap();
        if let Some(builder) = guard.as_mut() {
            let mt = media_type.into();
            builder
                .add_page(&data, mt, flags)
                .map_err(|e| BbfError::Io(e.to_string()))
        } else {
            Err(BbfError::AlreadyFinalized)
        }
    }

    pub fn finalize(&self) -> Result<(), BbfError> {
        let mut guard = self.inner.lock().unwrap();
        if let Some(builder) = guard.take() {
            builder.finalize().map_err(|e| BbfError::Io(e.to_string()))
        } else {
            Err(BbfError::AlreadyFinalized)
        }
    }
}

pub struct BbfReader {
    inner: BBFReader<Vec<u8>>,
}

impl BbfReader {
    pub fn new(data: Vec<u8>) -> Result<Self, BbfError> {
        let reader = BBFReader::new(data)?;
        Ok(Self { inner: reader })
    }

    pub fn get_page_count(&self) -> u32 {
        self.inner.footer.page_count.get()
    }

    pub fn get_page(&self, page_index: u32) -> Result<Vec<u8>, BbfError> {
        self.inner
            .get_asset(self.inner.pages()[page_index as usize].asset_index.get())
            .map(|slice| slice.to_vec())
            .map_err(BbfError::from)
    }
}

pub enum MediaType {
    Unknown,
    Avif,
    Png,
    Webp,
    Jxl,
    Bmp,
    Gif,
    Tiff,
    Jpg,
}

impl From<MediaType> for BBFMediaType {
    fn from(val: MediaType) -> Self {
        match val {
            MediaType::Unknown => BBFMediaType::Unknown,
            MediaType::Avif => BBFMediaType::Avif,
            MediaType::Png => BBFMediaType::Png,
            MediaType::Webp => BBFMediaType::Webp,
            MediaType::Jxl => BBFMediaType::Jxl,
            MediaType::Bmp => BBFMediaType::Bmp,
            MediaType::Gif => BBFMediaType::Gif,
            MediaType::Tiff => BBFMediaType::Tiff,
            MediaType::Jpg => BBFMediaType::Jpg,
        }
    }
}
