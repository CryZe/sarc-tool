#![allow(non_snake_case)]
#![feature(link_args)]
#![no_main]

#[link_args = "-O3 -s TOTAL_MEMORY=33554432 -s ALLOW_MEMORY_GROWTH=1 -s BINARYEN_METHOD='native-wasm'"]
extern "C" {}

extern crate nintendo_sarc as sarc;
extern crate zip;

use sarc::SarcFolder;
use std::io::{Cursor, Write, Seek};
use std::slice::from_raw_parts;
use std::ptr::null_mut;
use zip::ZipWriter;
use zip::write::FileOptions;

pub struct Zip {
    data: Vec<u8>,
}

#[no_mangle]
pub unsafe extern "C" fn Zip_data(this: *const Zip) -> *const u8 {
    (&*this).data.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn Zip_len(this: *const Zip) -> usize {
    (&*this).data.len()
}

#[no_mangle]
pub unsafe extern "C" fn Zip_drop(this: *mut Zip) {
    Box::from_raw(this);
}

#[no_mangle]
pub unsafe extern "C" fn Sarc_to_zip(data: *const u8, len: usize) -> *mut Zip {
    if let Ok(archive) = sarc::parse(Cursor::new(from_raw_parts(data, len))) {
        let archive = archive.into_folder();
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut cursor);

            fn descend<W: Write + Seek>(writer: &mut ZipWriter<W>, folder: &SarcFolder) {
                for file in &folder.files {
                    writer
                        .start_file(
                            format!("{}/{}", folder.full_name, file.name),
                            FileOptions::default(),
                        )
                        .unwrap();
                    writer.write_all(&file.data).unwrap();
                }

                for folder in &folder.folders {
                    writer
                        .add_directory(format!("{}/", folder.full_name), FileOptions::default())
                        .unwrap();
                    descend(writer, folder);
                }
            }
            descend(&mut writer, &archive);
        }
        Box::into_raw(Box::new(Zip { data: cursor.into_inner() }))
    } else {
        null_mut()
    }
}
