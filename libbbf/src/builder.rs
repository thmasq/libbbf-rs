#![allow(clippy::cast_possible_truncation, clippy::missing_errors_doc)]

use std::collections::HashMap;
use std::io::{self, Seek, Write};
use xxhash_rust::xxh3::{Xxh3, xxh3_64};
use zerocopy::{FromZeros, IntoBytes};

use crate::format::{
    BBFAssetEntry, BBFFooter, BBFHeader, BBFMediaType, BBFMetadata, BBFPageEntry, BBFSection,
};

pub struct BBFBuilder<W: Write + Seek> {
    writer: W,
    current_offset: u64,

    assets: Vec<BBFAssetEntry>,
    pages: Vec<BBFPageEntry>,
    sections: Vec<BBFSection>,
    metadata: Vec<BBFMetadata>,
    string_pool: Vec<u8>,

    dedupe_map: HashMap<u64, u32>,
    string_map: HashMap<String, u32>,
}

impl<W: Write + Seek> BBFBuilder<W> {
    pub fn new(mut writer: W) -> io::Result<Self> {
        let header = BBFHeader {
            magic: *b"BBF1",
            version: 2,
            flags: 0.into(),
            header_len: (std::mem::size_of::<BBFHeader>() as u16).into(),
            reserved: 0.into(),
        };

        writer.write_all(header.as_bytes())?;
        let current_offset = std::mem::size_of::<BBFHeader>() as u64;

        Ok(Self {
            writer,
            current_offset,
            assets: Vec::new(),
            pages: Vec::new(),
            sections: Vec::new(),
            metadata: Vec::new(),
            string_pool: Vec::new(),
            dedupe_map: HashMap::new(),
            string_map: HashMap::new(),
        })
    }

    fn align_padding(&mut self) -> io::Result<()> {
        let padding = (4096 - (self.current_offset % 4096)) % 4096;
        if padding > 0 {
            let zeroes = vec![0u8; padding as usize];
            self.writer.write_all(&zeroes)?;
            self.current_offset += padding;
        }
        Ok(())
    }

    pub fn add_page(
        &mut self,
        data: &[u8],
        media_type: BBFMediaType,
        flags: u32,
    ) -> io::Result<u32> {
        let hash = xxh3_64(data);
        let asset_index;

        if let Some(&idx) = self.dedupe_map.get(&hash) {
            asset_index = idx;
        } else {
            self.align_padding()?;

            let offset = self.current_offset;
            let length = data.len() as u64;

            self.writer.write_all(data)?;
            self.current_offset += length;

            let entry = BBFAssetEntry {
                offset: offset.into(),
                length: length.into(),
                decoded_length: length.into(),
                xxh3_hash: hash.into(),
                type_: media_type as u8,
                flags: 0,
                padding: [0; 6],
                reserved: [0.into(); 3],
            };

            asset_index = self.assets.len() as u32;
            self.assets.push(entry);
            self.dedupe_map.insert(hash, asset_index);
        }

        self.pages.push(BBFPageEntry {
            asset_index: asset_index.into(),
            flags: flags.into(),
        });

        Ok(asset_index)
    }

    fn get_or_add_str(&mut self, s: &str) -> u32 {
        if let Some(&offset) = self.string_map.get(s) {
            return offset;
        }

        let offset = self.string_pool.len() as u32;
        self.string_pool.extend_from_slice(s.as_bytes());
        self.string_pool.push(0);
        self.string_map.insert(s.to_string(), offset);
        offset
    }

    pub fn add_section(&mut self, title: &str, start_page: u32, parent_idx: Option<u32>) {
        let section = BBFSection {
            section_title_offset: self.get_or_add_str(title).into(),
            section_start_index: start_page.into(),
            parent_section_index: parent_idx.unwrap_or(0xFFFF_FFFF).into(),
        };
        self.sections.push(section);
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        let meta = BBFMetadata {
            key_offset: self.get_or_add_str(key).into(),
            val_offset: self.get_or_add_str(value).into(),
        };
        self.metadata.push(meta);
    }

    pub fn finalize(self) -> io::Result<()> {
        let Self {
            mut writer,
            mut current_offset,
            assets,
            pages,
            sections,
            metadata,
            string_pool,
            ..
        } = self;

        let mut hasher = Xxh3::new();
        let mut footer = BBFFooter::new_zeroed();

        macro_rules! write_hash {
            ($slice:expr) => {
                if !$slice.is_empty() {
                    writer.write_all($slice)?;
                    hasher.update($slice);
                    current_offset += $slice.len() as u64;
                }
            };
        }

        footer.string_pool_offset = current_offset.into();
        write_hash!(&string_pool);

        footer.asset_table_offset = current_offset.into();
        footer.asset_count = (assets.len() as u32).into();
        for asset in &assets {
            write_hash!(asset.as_bytes());
        }

        footer.page_table_offset = current_offset.into();
        footer.page_count = (pages.len() as u32).into();
        for page in &pages {
            write_hash!(page.as_bytes());
        }

        footer.section_table_offset = current_offset.into();
        footer.section_count = (sections.len() as u32).into();
        for section in &sections {
            write_hash!(section.as_bytes());
        }

        footer.meta_table_offset = current_offset.into();
        footer.key_count = (metadata.len() as u32).into();
        for meta in &metadata {
            write_hash!(meta.as_bytes());
        }

        footer.index_hash = hasher.digest().into();
        footer.magic = *b"BBF1";

        writer.write_all(footer.as_bytes())?;

        let _ = current_offset;

        Ok(())
    }
}
