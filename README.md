# `png-to-slp`

 A command-line utility used to convert indexed PNGs to [SLPs](https://github.com/ChariotEngine/Slp)

## License

MIT

## Usage

```sh
$ cargo run -- --png-path /path/to/your.png
```

### SLP format in ASCII form
```
+-----------------------------+
|          SlpHeader          |
+-----------------------------+
|SlpShapeHeader|SlpShapeHeader|
+-----------------------------+
|                             |
| Array of u16 padding pairs  | <-+ Each SlpShapeHeader has a "shape_outline_offset"
|                             |     that points to a pair in this array
+-----------------------------+
|                             |
| Arrays of u32 offsets to    | <-+ Each SlpShapeHeader has a "shape_data_offsets"
|  first command in each row  |     that points to an array
|                             |
+-----------------------------+
|                             |
| Drawing commands used to    |
|  produce indexed image data |
|                             |
+-----------------------------+
```
