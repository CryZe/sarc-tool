extern crate byteorder;
#[macro_use]
extern crate quick_error;

mod encode;
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
    pub const SFAT_DATA_OFFSET: u32 = SARC_HEADER_LENGTH as u32 + SFAT_HEADER_LENGTH as u32;
    pub const NODE_SIZE: u32 = 0x10;
    pub const HASH_MULTIPLIER: u32 = 0x00000065;
}

pub struct Sarc {
    pub files: Vec<SarcFile>,
}

#[derive(Debug)]
pub struct SarcFile {
    pub name: String,
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

fn name_table_header_offset(node_count: usize) -> u32 {
    node_count as u32 * consts::NODE_SIZE + consts::SFAT_DATA_OFFSET
}

fn name_table_data_offset(node_count: usize) -> u32 {
    name_table_header_offset(node_count) + consts::SFNT_HEADER_LENGTH as u32
}

pub use self::parse::{parse, Error as ParseError, Result as ParseResult};
pub use self::encode::{encode, Error as EncodeError, Result as EncodeResult};

#[derive(Default)]
pub struct SarcFolder {
    pub name: String,
    pub full_name: String,
    pub folders: Vec<Box<SarcFolder>>,
    pub files: Vec<SarcFile>,
}

impl SarcFolder {
    pub fn push(&mut self, mut file: SarcFile) {
        if file.name.contains('/') {
            let (folder_name, file_name) = {
                let mut splits = file.name.splitn(2, '/');
                (
                    splits.next().unwrap().to_owned(),
                    splits.next().unwrap().to_owned(),
                )
            };
            file.name = file_name;
            if let Some(folder) = self.folders.iter_mut().find(|f| f.name == folder_name) {
                folder.push(file);
                return;
            }

            let mut full_name = self.full_name.clone();
            if !full_name.is_empty() {
                full_name.push_str("/");
            }
            full_name.push_str(&folder_name);

            let mut folder = SarcFolder {
                name: folder_name,
                full_name,
                ..Default::default()
            };
            folder.push(file);
            self.folders.push(Box::new(folder));
        } else {
            self.files.push(file);
        }
    }
}

impl Sarc {
    pub fn into_folder(self) -> SarcFolder {
        let mut folder = SarcFolder::default();
        for file in self.files {
            folder.push(file);
        }
        folder
    }
}
