extern crate byteorder;
#[macro_use]
extern crate quick_error;

mod sarc;

use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::fs::{File, create_dir_all};

fn main() {
    let file = BufReader::new(File::open(r"D:\Downloads\Dungeon000.pack").unwrap());
    let archive = sarc::parse(file).unwrap();

    let base = Path::new("extracted");

    for file in archive.files {
        // TODO Handle empty file paths
        println!("Extracting '{}'...", file.path);
        let path = base.join(&file.path);
        create_dir_all(path.parent().unwrap()).unwrap();

        BufWriter::new(File::create(path).unwrap())
            .write_all(&file.data)
            .unwrap();
    }
}
