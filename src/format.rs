use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BBFMediaType {
    Unknown = 0x00,
    Avif = 0x01,
    Png = 0x02,
    Webp = 0x03,
    Jxl = 0x04,
    Bmp = 0x05,
    Gif = 0x07,
    Tiff = 0x08,
    Jpg = 0x09,
}

impl From<u8> for BBFMediaType {
    fn from(v: u8) -> Self {
        match v {
            0x01 => Self::Avif,
            0x02 => Self::Png,
            0x03 => Self::Webp,
            0x04 => Self::Jxl,
            0x05 => Self::Bmp,
            0x07 => Self::Gif,
            0x08 => Self::Tiff,
            0x09 => Self::Jpg,
            _ => Self::Unknown,
        }
    }
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFHeader {
    pub magic: [u8; 4], // "BBF1"
    pub version: u8,    // 2
    pub flags: u32,
    pub header_len: u16,
    pub reserved: u64,
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFAssetEntry {
    pub offset: u64,
    pub length: u64,
    pub decoded_length: u64,
    pub xxh3_hash: u64,
    pub type_: u8,
    pub flags: u8,
    pub padding: [u8; 6],
    pub reserved: [u64; 3],
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFPageEntry {
    pub asset_index: u32,
    pub flags: u32,
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFSection {
    pub section_title_offset: u32,
    pub section_start_index: u32,
    pub parent_section_index: u32,
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFMetadata {
    pub key_offset: u32,
    pub val_offset: u32,
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFFooter {
    pub string_pool_offset: u64,
    pub asset_table_offset: u64,
    pub asset_count: u32,

    pub page_table_offset: u64,
    pub page_count: u32,

    pub section_table_offset: u64,
    pub section_count: u32,

    pub meta_table_offset: u64,
    pub key_count: u32,

    pub extra_offset: u64,

    pub index_hash: u64,
    pub magic: [u8; 4],
}
