use std::io::{self, Write};
use std::result;
use byteorder::{WriteBytesExt, BigEndian as BE};
use super::{Sarc, name_table_data_offset};
use super::consts::*;

pub type Result<T> = result::Result<T, Error>;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: io::Error) {
            cause(err)
            from()
            description(err.description())
        }
    }
}

fn align32(x: u32) -> u32 {
    if x & 3 == 0 { x } else { (x | 3) + 1 }
}

fn file_name_length(s: &str) -> u32 {
    if s.is_empty() {
        0
    } else {
        align32(s.len() as u32 + 1)
    }
}

fn hash(name: &str, multiplier: u32) -> u32 {
    name.bytes().fold(
        0,
        |a, b| (b as u32).wrapping_add(a.wrapping_mul(multiplier)),
    )
}

pub fn encode<W: Write>(mut writer: W, archive: &Sarc) -> Result<()> {
    writer.write_all(&SARC_MAGIC)?;
    writer.write_u16::<BE>(SARC_HEADER_LENGTH)?;
    writer.write_all(&BOM_BE)?;

    let name_table_size: u32 = archive
        .files
        .iter()
        .map(|f| file_name_length(&f.name))
        .sum();

    let padded_file_sizes = archive
        .files
        .iter()
        .take(archive.files.len() - 1)
        .map(|f| align32(f.data.len() as u32))
        .sum::<u32>();

    let unpadded_last_file_size = archive
        .files
        .last()
        .map(|f| f.data.len() as u32)
        .unwrap_or_default();

    let data_size = padded_file_sizes + unpadded_last_file_size;

    let name_table_data_offset = name_table_data_offset(archive.files.len());
    let beginning_of_data_offset = name_table_data_offset as u32 + name_table_size;
    let file_size = beginning_of_data_offset + data_size;

    writer.write_u32::<BE>(file_size)?;
    writer.write_u32::<BE>(beginning_of_data_offset)?;
    writer.write_u32::<BE>(0x01000000)?;

    writer.write_all(&SFAT_MAGIC)?;
    writer.write_u16::<BE>(SFAT_HEADER_LENGTH)?;
    writer.write_u16::<BE>(archive.files.len() as u16)?;
    writer.write_u32::<BE>(HASH_MULTIPLIER)?;

    let mut entry = 0;
    let mut beginning_of_data = 0;

    for file in &archive.files {
        let hash = hash(&file.name, HASH_MULTIPLIER);
        let file_name_length = file_name_length(&file.name);

        let mut encoded_entry = entry;
        if file_name_length > 0 {
            encoded_entry |= 1 << 24;
        }

        writer.write_u32::<BE>(hash)?;
        writer.write_u32::<BE>(encoded_entry)?;
        writer.write_u32::<BE>(beginning_of_data)?;
        beginning_of_data += file.data.len() as u32;
        writer.write_u32::<BE>(beginning_of_data)?;

        beginning_of_data = align32(beginning_of_data);
        entry += file_name_length / 4;
    }

    writer.write_all(&SFNT_MAGIC)?;
    writer.write_u16::<BE>(SFNT_HEADER_LENGTH)?;
    writer.write_u16::<BE>(0x0000)?;

    for file in &archive.files {
        writer.write_all(file.name.as_bytes())?;
        let padding_bytes = file_name_length(&file.name) - file.name.len() as u32;
        for _ in 0..padding_bytes {
            writer.write_all(&[0])?;
        }
    }

    for (i, file) in archive.files.iter().enumerate() {
        writer.write_all(&file.data)?;
        if i + 1 != archive.files.len() {
            let padding_bytes = align32(file.data.len() as u32) - file.data.len() as u32;
            for _ in 0..padding_bytes {
                writer.write_all(&[0])?;
            }
        }
    }

    Ok(())
}
