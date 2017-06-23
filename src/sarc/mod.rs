mod parse;

mod consts {
    pub const SARC_MAGIC: [u8; 4] = *b"SARC";
    pub const SFAT_MAGIC: [u8; 4] = *b"SFAT";
    pub const SFNT_MAGIC: [u8; 4] = *b"SFNT";
    pub const SARC_HEADER_LENGTH: u16 = 0x14;
    pub const SFAT_HEADER_LENGTH: u16 = 0xc;
    pub const SFNT_HEADER_LENGTH: u16 = 0x8;
    pub const BOM_BE: [u8; 2] = [0xFE, 0xFF];
    pub const BOM_LE: [u8; 2] = [0xFF, 0xFE];
    pub const SFNT_TABLE_OFFSET: u64 = 0xc8;
}

pub struct Sarc {
    pub files: Vec<SarcFile>,
}

#[derive(Debug)]
pub struct SarcFile {
    pub path: String,
    pub data: Vec<u8>,
}

#[derive(Debug)]
struct Node {
    hash: u32,
    is_file_name_stored: bool,
    file_name_table_entry: u32,
    beginning_of_node_file_data: u32,
    end_of_node_file_data: u32,
}

pub use self::parse::{parse, Error as ParseError, Result as ParseResult};
