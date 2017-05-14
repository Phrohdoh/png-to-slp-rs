use std::fs::File;

extern crate clap;
use clap::{App, Arg};

extern crate chariot_slp;

extern crate png;
use png::HasParameters;

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
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).expect("Failed to read frame");

    assert_eq!(png::ColorType::Indexed, info.color_type);
}
