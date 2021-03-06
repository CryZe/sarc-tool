extern crate clap;
extern crate walkdir;
extern crate itertools;
extern crate zip;
extern crate nintendo_sarc as sarc;

use std::io::{Read, Seek, BufReader, BufWriter, Write};
use std::path::Path;
use std::fs::{File, create_dir_all};
use clap::{Arg, App, SubCommand};
use walkdir::WalkDir;
use itertools::Itertools;
use sarc::{Sarc, SarcFile, SarcFolder};
use zip::ZipWriter;
use zip::write::FileOptions;

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
        )
        .subcommand(
            SubCommand::with_name("to-zip")
                .about("Extracts SARC Files to a ZIP File")
                .arg(
                    Arg::with_name("input")
                        .help("The path to the SARC file to extract")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .help("The path to the output zip file")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("pack")
                .about("Packs a folder into a SARC file")
                .arg(
                    Arg::with_name("input")
                        .help("The path folder to pack into the SARC file")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("output")
                        .help(
                            "The path to the SARC file to pack into. This is being overwritten.",
                        )
                        .takes_value(true)
                        .required(true),
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
            println!("Extracting '{}'...", file.name);
            let path = base.join(&file.name);
            create_dir_all(path.parent().unwrap()).unwrap();

            BufWriter::new(File::create(path).unwrap())
                .write_all(&file.data)
                .unwrap();
        }
    } else if let Some(matches) = matches.subcommand_matches("to-zip") {
        let input = matches.value_of("input").unwrap();
        let output = matches.value_of("output").unwrap();

        let file = BufReader::new(File::open(input).unwrap());
        let archive = sarc::parse(file).unwrap().into_folder();

        let mut writer = ZipWriter::new(BufWriter::new(File::create(output).unwrap()));

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
    } else if let Some(matches) = matches.subcommand_matches("pack") {
        let input = matches.value_of("input").unwrap_or_default();
        let output = matches.value_of("output").unwrap();

        let mut archive = Sarc { files: Vec::new() };

        for file in WalkDir::new(input) {
            let file = file.unwrap();
            if file.path().is_file() {
                let path = file.path()
                    .strip_prefix(input)
                    .unwrap()
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .join("/");

                println!("Packing '{}'...", path);

                let mut file = BufReader::new(File::open(file.path()).unwrap());
                let mut data = Vec::new();
                file.read_to_end(&mut data).unwrap();

                archive.files.push(SarcFile { name: path, data });
            }
        }

        let file = BufWriter::new(File::create(output).unwrap());
        sarc::encode(file, &archive).unwrap();
    } else {
        app.print_help().unwrap();
    }
}
