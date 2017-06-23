extern crate byteorder;
#[macro_use]
extern crate quick_error;

use std::io::{self, Read, Seek, SeekFrom, BufReader, BufWriter, Write};
use std::{mem, result};
use std::path::Path;
use std::fs::{File, create_dir_all};
use byteorder::{ReadBytesExt, BigEndian as BE, LittleEndian as LE, ByteOrder};

pub type Result<T> = result::Result<T, Error>;
pub type ParseNodeResult<T> = result::Result<T, ParseNodeError>;

// TODO Verify File Name Hashes

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        IncorrectMagic(got: [u8; 4], expected: [u8; 4]) {}
        IncorrectHeaderLength(got: u16, expected: u16) {}
        IncorrectBom(bom: [u8; 2]) {}
        ParseNode(err: ParseNodeError) {
            cause(err)
            from()
            description(err.description())
        }
        Io(err: io::Error) {
            cause(err)
            from()
            description(err.description())
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum ParseNodeError {
        Io(err: io::Error) {
            cause(err)
            from()
            description(err.description())
        }
    }
}

const SARC_MAGIC: [u8; 4] = *b"SARC";
const SFAT_MAGIC: [u8; 4] = *b"SFAT";
const SFNT_MAGIC: [u8; 4] = *b"SFNT";
const SARC_HEADER_LENGTH: u16 = 0x14;
const SFAT_HEADER_LENGTH: u16 = 0xc;
const SFNT_HEADER_LENGTH: u16 = 0x8;
const BOM_BE: [u8; 2] = [0xFE, 0xFF];
const BOM_LE: [u8; 2] = [0xFF, 0xFE];
const SFNT_TABLE_OFFSET: u64 = 0xc8;

fn verify_magic<R: Read>(mut reader: R, expected: [u8; 4]) -> Result<()> {
    let mut magic: [u8; 4] = unsafe { mem::uninitialized() };
    reader.read_exact(&mut magic)?;
    if magic == expected {
        Ok(())
    } else {
        Err(Error::IncorrectMagic(magic, expected))
    }
}

fn verify_header_length<B: ByteOrder>(buf: &[u8; 2], expected: u16) -> Result<()> {
    let header_length = B::read_u16(buf);
    if header_length == expected {
        Ok(())
    } else {
        Err(Error::IncorrectHeaderLength(header_length, expected))
    }
}

fn parse_with_bom<B, R>(header_length: &[u8; 2], mut reader: R) -> Result<Vec<SarcFile>>
where
    B: ByteOrder,
    R: Read + Seek,
{
    verify_header_length::<B>(header_length, SARC_HEADER_LENGTH)?;

    let _file_size = reader.read_u32::<B>()?;
    let beginning_of_data_offset = reader.read_u32::<B>()?;
    let _unknown = reader.read_u32::<B>()?;

    let nodes = parse_file_table::<B, _>(&mut reader)?;
    parse_file_name_table::<B, _>(&mut reader)?;

    let mut buf = Vec::new();

    let files = nodes
        .into_iter()
        .map(|node| {
            parse_file::<B, _>(&mut reader, &mut buf, beginning_of_data_offset, node)
        })
        .collect::<Result<_>>()?;

    Ok(files)
}

#[derive(Debug)]
struct SarcFile {
    path: String,
    data: Vec<u8>,
}

fn parse_file<B, R>(
    mut reader: R,
    buf: &mut Vec<u8>,
    data_offset: u32,
    node: Node,
) -> Result<SarcFile>
where
    B: ByteOrder,
    R: Read + Seek,
{
    let path = if node.is_file_name_stored {
        reader.seek(SeekFrom::Start(
            node.file_name_table_entry as u64 * 4 +
                SFNT_TABLE_OFFSET,
        ))?;

        buf.clear();
        let mut byte: [u8; 1] = unsafe { mem::uninitialized() };
        loop {
            reader.read_exact(&mut byte)?;
            if byte[0] == 0 {
                break;
            }
            buf.push(byte[0]);
        }

        String::from_utf8_lossy(&buf).into_owned()
    } else {
        String::new()
    };

    let byte_count = node.end_of_node_file_data - node.beginning_of_node_file_data;
    let mut data = Vec::with_capacity(byte_count as usize);
    unsafe {
        let len = data.capacity();
        data.set_len(len);
    }

    reader.seek(SeekFrom::Start(
        (node.beginning_of_node_file_data + data_offset) as
            u64,
    ))?;
    reader.read_exact(&mut data)?;

    Ok(SarcFile { path, data })
}

fn parse_file_table<B, R>(mut reader: R) -> Result<Vec<Node>>
where
    B: ByteOrder,
    R: Read,
{
    verify_magic(&mut reader, SFAT_MAGIC)?;

    let mut header_length: [u8; 2] = unsafe { mem::uninitialized() };
    reader.read_exact(&mut header_length)?;
    verify_header_length::<B>(&header_length, SFAT_HEADER_LENGTH)?;

    let node_count = reader.read_u16::<B>()?;
    let _hash_multiplier = reader.read_u32::<B>()?;

    let nodes = (0..node_count)
        .map(|_| parse_node::<B, _>(&mut reader).map_err(Into::into))
        .collect::<Result<_>>()?;

    Ok(nodes)
}

fn parse_file_name_table<B, R>(mut reader: R) -> Result<()>
where
    B: ByteOrder,
    R: Read,
{
    verify_magic(&mut reader, SFNT_MAGIC)?;

    let mut header_length: [u8; 2] = unsafe { mem::uninitialized() };
    reader.read_exact(&mut header_length)?;
    verify_header_length::<B>(&header_length, SFNT_HEADER_LENGTH)?;

    let _unknown = reader.read_u16::<B>()?;

    Ok(())
}

#[derive(Debug)]
struct Node {
    hash: u32,
    is_file_name_stored: bool,
    file_name_table_entry: u32,
    beginning_of_node_file_data: u32,
    end_of_node_file_data: u32,
}

fn parse_node<B, R>(mut reader: R) -> ParseNodeResult<Node>
where
    B: ByteOrder,
    R: Read,
{
    let hash = reader.read_u32::<B>()?;
    let archive_type = reader.read_u32::<B>()?;
    let beginning_of_node_file_data = reader.read_u32::<B>()?;
    let end_of_node_file_data = reader.read_u32::<B>()?;

    let is_file_name_stored = archive_type >> 24 == 0x01;
    let file_name_table_entry = archive_type & 0xFF_FF_FF;

    Ok(Node {
        hash,
        is_file_name_stored,
        file_name_table_entry,
        beginning_of_node_file_data,
        end_of_node_file_data,
    })
}

fn parse<R>(mut reader: R) -> Result<Vec<SarcFile>>
where
    R: Read + Seek,
{
    verify_magic(&mut reader, SARC_MAGIC)?;

    let mut header_length: [u8; 2] = unsafe { mem::uninitialized() };
    reader.read_exact(&mut header_length)?;

    let mut bom: [u8; 2] = unsafe { mem::uninitialized() };
    reader.read_exact(&mut bom)?;

    if bom == BOM_BE {
        parse_with_bom::<BE, _>(&header_length, reader)
    } else if bom == BOM_LE {
        parse_with_bom::<LE, _>(&header_length, reader)
    } else {
        Err(Error::IncorrectBom(bom))
    }
}

fn main() {
    let file = BufReader::new(File::open(r"D:\Downloads\Dungeon000.pack").unwrap());
    let files = parse(file).unwrap();

    let base = Path::new("extracted");

    for file in files {
        // TODO Handle empty file paths
        println!("Extracting '{}'...", file.path);
        let path = base.join(&file.path);
        create_dir_all(path.parent().unwrap()).unwrap();

        BufWriter::new(File::create(path).unwrap())
            .write_all(&file.data)
            .unwrap();
    }
}
