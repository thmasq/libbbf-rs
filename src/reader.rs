use std::io::{self, Read, Seek, SeekFrom};
use std::mem::size_of;
use zerocopy::{FromBytes, FromZeros, IntoBytes};

use crate::format::*;

#[derive(Debug, thiserror::Error)]
pub enum BBFError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid BBF Magic")]
    InvalidMagic,
    #[error("File too short or corrupted header")]
    FileTooShort,
    #[error("Table error")]
    TableError,
}

pub struct BBFReader<R: Read + Seek> {
    reader: R,
    pub header: BBFHeader,
    pub footer: BBFFooter,

    pub assets: Vec<BBFAssetEntry>,
    pub pages: Vec<BBFPageEntry>,
    pub sections: Vec<BBFSection>,
    pub metadata: Vec<BBFMetadata>,
    pub string_pool: Vec<u8>,
}

impl<R: Read + Seek> BBFReader<R> {
    pub fn new(mut reader: R) -> Result<Self, BBFError> {
        reader.seek(SeekFrom::Start(0))?;
        let mut header_buf = [0u8; size_of::<BBFHeader>()];
        reader.read_exact(&mut header_buf)?;

        let header =
            BBFHeader::read_from_bytes(&header_buf[..]).map_err(|_| BBFError::FileTooShort)?;

        if &header.magic != b"BBF1" {
            return Err(BBFError::InvalidMagic);
        }

        let footer_size = size_of::<BBFFooter>() as i64;
        let file_len = reader.seek(SeekFrom::End(0))?;
        if file_len < (size_of::<BBFHeader>() as u64 + footer_size as u64) {
            return Err(BBFError::FileTooShort);
        }

        reader.seek(SeekFrom::End(-footer_size))?;
        let mut footer_buf = [0u8; size_of::<BBFFooter>()];
        reader.read_exact(&mut footer_buf)?;

        let footer =
            BBFFooter::read_from_bytes(&footer_buf[..]).map_err(|_| BBFError::FileTooShort)?;

        if &footer.magic != b"BBF1" {
            return Err(BBFError::InvalidMagic);
        }

        let str_pool_len = footer.asset_table_offset - footer.string_pool_offset;
        reader.seek(SeekFrom::Start(footer.string_pool_offset.get()))?;
        let mut string_pool = vec![0u8; str_pool_len.get() as usize];
        reader.read_exact(&mut string_pool)?;

        reader.seek(SeekFrom::Start(footer.asset_table_offset.get()))?;
        let mut assets = vec![BBFAssetEntry::new_zeroed(); footer.asset_count.get() as usize];
        reader.read_exact(assets.as_mut_slice().as_mut_bytes())?;

        reader.seek(SeekFrom::Start(footer.page_table_offset.get()))?;
        let mut pages = vec![BBFPageEntry::new_zeroed(); footer.page_count.get() as usize];
        reader.read_exact(pages.as_mut_slice().as_mut_bytes())?;

        reader.seek(SeekFrom::Start(footer.section_table_offset.get()))?;
        let mut sections = vec![BBFSection::new_zeroed(); footer.section_count.get() as usize];
        reader.read_exact(sections.as_mut_slice().as_mut_bytes())?;

        reader.seek(SeekFrom::Start(footer.meta_table_offset.get()))?;
        let mut metadata = vec![BBFMetadata::new_zeroed(); footer.key_count.get() as usize];
        reader.read_exact(metadata.as_mut_slice().as_mut_bytes())?;

        Ok(Self {
            reader,
            header,
            footer,
            assets,
            pages,
            sections,
            metadata,
            string_pool,
        })
    }

    pub fn get_string(&self, offset: u32) -> Option<&str> {
        let offset = offset as usize;
        if offset >= self.string_pool.len() {
            return None;
        }
        let slice = &self.string_pool[offset..];
        let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
        std::str::from_utf8(&slice[..end]).ok()
    }

    pub fn read_asset(&mut self, asset_index: u32) -> Result<Vec<u8>, BBFError> {
        if asset_index as usize >= self.assets.len() {
            return Err(BBFError::TableError);
        }
        let asset = &self.assets[asset_index as usize];

        let mut buffer = vec![0u8; asset.length.get() as usize];
        self.reader.seek(SeekFrom::Start(asset.offset.get()))?;
        self.reader.read_exact(&mut buffer)?;

        Ok(buffer)
    }
}
