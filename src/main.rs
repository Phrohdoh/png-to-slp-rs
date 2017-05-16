use std::fs::{File, OpenOptions};
use std::io::Write;

extern crate clap;
use clap::{App, Arg};

extern crate chariot_slp;

extern crate chariot_io_tools;
use chariot_io_tools::WriteExt;

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

    let (info, mut reader) = decoder.read_info().expect("Failed to 'read_info'");
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

    // Write out the SlpShapeHeaders
    for i in 0..slp.header.shape_count {
        let outline_array_offset = (32 * (i+1)) + 32 * slp.header.shape_count;

        let shape_header = chariot_slp::SlpShapeHeader {
            shape_data_offsets: 0u32, // This has to be calculated and set at a later time.
            shape_outline_offset: outline_array_offset,
            palette_offset: 0u32,
            properties: 0u32,
            width: info.width,
            height: info.height,
            center_x: (info.width / 2) as i32,
            center_y: (info.height / 2) as i32,
        };

        println!("TODO(shape_data_offsets): {}", shape_header.shape_data_offsets);
        println!("shape_outline_offset: {}", shape_header.shape_outline_offset);

        shape_header.write_to(&mut output)
            .expect(&format!("Failed to write out SlpShapeHeader {}: {:?}", i, shape_header));
    }

    // Write out the u16 padding pairs
    for i in 0..slp.header.shape_count {
        output.write_u16(0u16); // left
        output.write_u16(0u16); // right
    }

    let data_offsets_start_pos = output.position();
    println!("data_offsets_start_pos: {}", data_offsets_start_pos);

    let mut row_start_cmd_offsets = Vec::new();
    for (row_idx, runs) in rows_of_runs.iter().enumerate() {
        let row_cmd_start_pos = output.position() as u32;
        row_start_cmd_offsets.push(row_cmd_start_pos);

        let last_run_idx = runs.len() - 1;
        for (run_idx, run) in runs.iter().enumerate() {
            let is_last_run = run_idx == last_run_idx;

            // TODO: How do I correctly determine cmd_byte?
            // cmd_byte must satisfy these conditions:
            // * cmd_byte << 2 == run.len
            // * cmd_byte & 0x0F == (0x00 || 0x04 || 0x08 || 0x0C)
            // For now we'll always use the `block copy` command.
            let cmd_byte = run.len << 2;
            output.write_u8(cmd_byte);

            for _ in 0..run.len {
                output.write_u8(run.val);
            }

            if is_last_run {
                output.write_u8(0x0F);
            }
        }

        let pos = output.position();
        output.set_position(32 + (32 * row_idx) as u64);
        output.write_u32(row_cmd_start_pos);
        output.set_position(pos);
    }

    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .open("output.slp")
        .expect("Failed to prepare 'output.slp'");

    f.write_all(&output.into_inner()).expect("Failed to write to 'output.slp'");
}
