use zerocopy::byteorder::LittleEndian;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};
use zerocopy::{U16, U32, U64};

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
    pub flags: U32<LittleEndian>,
    pub header_len: U16<LittleEndian>,
    pub reserved: U64<LittleEndian>,
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFAssetEntry {
    pub offset: U64<LittleEndian>,
    pub length: U64<LittleEndian>,
    pub decoded_length: U64<LittleEndian>,
    pub xxh3_hash: U64<LittleEndian>,
    pub type_: u8,
    pub flags: u8,
    pub padding: [u8; 6],
    pub reserved: [U64<LittleEndian>; 3],
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFPageEntry {
    pub asset_index: U32<LittleEndian>,
    pub flags: U32<LittleEndian>,
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFSection {
    pub section_title_offset: U32<LittleEndian>,
    pub section_start_index: U32<LittleEndian>,
    pub parent_section_index: U32<LittleEndian>,
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFMetadata {
    pub key_offset: U32<LittleEndian>,
    pub val_offset: U32<LittleEndian>,
}

#[repr(C, packed)]
#[derive(IntoBytes, FromBytes, Immutable, KnownLayout, Unaligned, Debug, Clone, Copy)]
pub struct BBFFooter {
    pub string_pool_offset: U64<LittleEndian>,
    pub asset_table_offset: U64<LittleEndian>,
    pub asset_count: U32<LittleEndian>,

    pub page_table_offset: U64<LittleEndian>,
    pub page_count: U32<LittleEndian>,

    pub section_table_offset: U64<LittleEndian>,
    pub section_count: U32<LittleEndian>,

    pub meta_table_offset: U64<LittleEndian>,
    pub key_count: U32<LittleEndian>,

    pub extra_offset: U64<LittleEndian>,

    pub index_hash: U64<LittleEndian>,
    pub magic: [u8; 4],
}
