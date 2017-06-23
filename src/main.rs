extern crate byteorder;
#[macro_use]
extern crate quick_error;
extern crate clap;

mod sarc;

use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::fs::{File, create_dir_all};
use clap::{Arg, App, SubCommand};

fn main() {
    let mut app = App::new("sarc-tool")
        .author("Christopher Serr <christopher.serr@gmail.com>")
        .about("Extracts, packs and modifies Nintendo's SARC Files.")
        .subcommand(
            SubCommand::with_name("extract")
                .about("Extracts SARC Files")
                .arg(
                    Arg::with_name("input")
                        .help("The path to the SARC file to extract")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .help("The path to the folder to extract the SARC file into")
                        .takes_value(true)
                        .required(false),
                ),
        );

    let matches = app.clone().get_matches();

    if let Some(matches) = matches.subcommand_matches("extract") {
        let input = matches.value_of("input").unwrap();
        let output = matches.value_of("output").unwrap_or_default();

        let file = BufReader::new(File::open(input).unwrap());
        let archive = sarc::parse(file).unwrap();

        let base = Path::new(output);

        for file in archive.files {
            // TODO Handle empty file paths
            println!("Extracting '{}'...", file.path);
            let path = base.join(&file.path);
            create_dir_all(path.parent().unwrap()).unwrap();

            BufWriter::new(File::create(path).unwrap())
                .write_all(&file.data)
                .unwrap();
        }
    } else {
        app.print_help().unwrap();
    }
}
