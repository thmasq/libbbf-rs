#![allow(
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc,
    clippy::cast_possible_wrap
)]

use std::mem::size_of;
use zerocopy::FromBytes;

use crate::format::{BBFAssetEntry, BBFFooter, BBFHeader, BBFMetadata, BBFPageEntry, BBFSection};

#[derive(Debug, thiserror::Error)]
pub enum BBFError {
    #[error("Invalid BBF Magic")]
    InvalidMagic,
    #[error("File too short or corrupted header")]
    FileTooShort,
    #[error("Table error or invalid offsets")]
    TableError,
    #[error("Index out of bounds")]
    OutOfBounds,
}

pub struct BBFReader<T: AsRef<[u8]>> {
    data: T,
    pub header: BBFHeader,
    pub footer: BBFFooter,
}

impl<T: AsRef<[u8]>> BBFReader<T> {
    pub fn new(data: T) -> Result<Self, BBFError> {
        let slice = data.as_ref();
        let total_len = slice.len() as u64;

        if total_len < (size_of::<BBFHeader>() + size_of::<BBFFooter>()) as u64 {
            return Err(BBFError::FileTooShort);
        }

        let header_slice = &slice[..size_of::<BBFHeader>()];
        let header =
            BBFHeader::read_from_bytes(header_slice).map_err(|_| BBFError::FileTooShort)?;

        if &header.magic != b"BBF1" {
            return Err(BBFError::InvalidMagic);
        }

        let footer_offset = (total_len as usize) - size_of::<BBFFooter>();
        let footer_slice = &slice[footer_offset..];
        let footer =
            BBFFooter::read_from_bytes(footer_slice).map_err(|_| BBFError::FileTooShort)?;

        if &footer.magic != b"BBF1" {
            return Err(BBFError::InvalidMagic);
        }

        let check_range = |offset: u64, count: u32, elem_size: usize| -> Result<(), BBFError> {
            let start = offset;
            let size = u64::from(count)
                .checked_mul(elem_size as u64)
                .ok_or(BBFError::TableError)?;
            let end = start.checked_add(size).ok_or(BBFError::TableError)?;

            if end > total_len {
                return Err(BBFError::FileTooShort);
            }
            Ok(())
        };

        if footer.string_pool_offset.get() > footer.asset_table_offset.get()
            || footer.asset_table_offset.get() > total_len
        {
            return Err(BBFError::TableError);
        }

        check_range(
            footer.asset_table_offset.get(),
            footer.asset_count.get(),
            size_of::<BBFAssetEntry>(),
        )?;
        check_range(
            footer.page_table_offset.get(),
            footer.page_count.get(),
            size_of::<BBFPageEntry>(),
        )?;
        check_range(
            footer.section_table_offset.get(),
            footer.section_count.get(),
            size_of::<BBFSection>(),
        )?;
        check_range(
            footer.meta_table_offset.get(),
            footer.key_count.get(),
            size_of::<BBFMetadata>(),
        )?;

        Ok(Self {
            data,
            header,
            footer,
        })
    }

    fn get_table_slice<U: FromBytes + zerocopy::Immutable>(&self, offset: u64, count: u32) -> &[U] {
        let start = offset as usize;
        let elem_size = size_of::<U>();
        let len = (count as usize) * elem_size;

        let byte_slice = &self.data.as_ref()[start..start + len];

        <[U]>::ref_from_bytes(byte_slice).unwrap_or(&[])
    }

    pub fn assets(&self) -> &[BBFAssetEntry] {
        self.get_table_slice(
            self.footer.asset_table_offset.get(),
            self.footer.asset_count.get(),
        )
    }

    pub fn pages(&self) -> &[BBFPageEntry] {
        self.get_table_slice(
            self.footer.page_table_offset.get(),
            self.footer.page_count.get(),
        )
    }

    pub fn sections(&self) -> &[BBFSection] {
        self.get_table_slice(
            self.footer.section_table_offset.get(),
            self.footer.section_count.get(),
        )
    }

    pub fn metadata(&self) -> &[BBFMetadata] {
        self.get_table_slice(
            self.footer.meta_table_offset.get(),
            self.footer.key_count.get(),
        )
    }

    pub fn get_string(&self, offset: u32) -> Option<&str> {
        let pool_start = self.footer.string_pool_offset.get() as usize;
        let pool_end = self.footer.asset_table_offset.get() as usize;

        let pool_slice = &self.data.as_ref()[pool_start..pool_end];

        let offset = offset as usize;
        if offset >= pool_slice.len() {
            return None;
        }

        let slice_from_offset = &pool_slice[offset..];
        let end = slice_from_offset
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(slice_from_offset.len());

        std::str::from_utf8(&slice_from_offset[..end]).ok()
    }

    pub fn get_asset(&self, asset_index: u32) -> Result<&[u8], BBFError> {
        let assets = self.assets();
        if asset_index as usize >= assets.len() {
            return Err(BBFError::OutOfBounds);
        }

        let asset = &assets[asset_index as usize];
        let offset = asset.offset.get() as usize;
        let length = asset.length.get() as usize;

        let total_slice = self.data.as_ref();

        if offset.checked_add(length).ok_or(BBFError::OutOfBounds)? > total_slice.len() {
            return Err(BBFError::FileTooShort);
        }

        Ok(&total_slice[offset..offset + length])
    }
}
