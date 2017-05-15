use std::fs::File;

extern crate clap;
use clap::{App, Arg};

extern crate chariot_slp;

extern crate png;
use png::HasParameters;

extern crate itertools;
use itertools::Itertools;

#[derive(Debug)]
struct Run {
    pub len: u8,
    pub val: u8,
}

fn main() {
    let matches = App::new("png-to-slp")
        .version("0.1.0")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .about("Convert a PNG file to an SLP")
        .arg(Arg::with_name("png-path")
            .long("png-path")
            .value_name("/path/to/indexed.png")
            .help("Filepath to the PNG to convert to an SLP")
            .required(true)
            .takes_value(true))
        .get_matches();

    let png_path = matches.value_of("png-path").unwrap();
    let f = File::open(png_path).expect(&format!("Failed to open {}", &png_path));

    let mut decoder = png::Decoder::new(f);
    // Do what is _expected_ instead of trying to be clever.
    decoder.set(png::TRANSFORM_IDENTITY);

    let (info, mut reader) = decoder.read_info().expect("Failed to 'read_info' ???");
    assert_eq!(png::ColorType::Indexed, info.color_type);

    let mut rows_of_runs = Vec::new();

    while let Ok(Some(bytes)) = reader.next_row() {
        let mut runs = Vec::new();

        for (val, group) in &bytes.into_iter().group_by(|idx| *idx) {
            let len = group.into_iter().count() as u8;
            let val = *val;
            runs.push(Run { len, val });
        }

        rows_of_runs.push(runs);
    }

    println!("{:?}", rows_of_runs);
    return;

    let mut slp_header = chariot_slp::SlpHeader::new();
    slp_header.file_version = *b"2.0N";
    slp_header.shape_count = 1;
    slp_header.comment = *b"Chariot/png-to-slp\0\0\0\0\0\0";

    let mut slp = chariot_slp::SlpFile::new(1u8);
    slp.header = slp_header;

    let mut output = {
        let raw_bytes = Vec::new();
        std::io::Cursor::new(raw_bytes)
    };

    slp.header.write_to(&mut output).expect("Failed to write SlpHeader");

    for i in 0..slp.header.shape_count {
        let shape_header = chariot_slp::SlpShapeHeader {
            shape_data_offsets: 32 + 32 * slp.header.shape_count,
            shape_outline_offset: 0u32,
            palette_offset: 0u32,
            properties: 0u32,
            width: info.width,
            height: info.height,
            center_x: (info.width / 2) as i32,
            center_y: (info.height / 2) as i32,
        };

        shape_header.write_to(&mut output).expect(&format!("Failed to write out SlpShapeHeader {}: {:?}", i, shape_header));
    }

    println!("{:?}", output.into_inner());
}
