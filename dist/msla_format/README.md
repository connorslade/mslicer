# `msla_format` [![Build](https://github.com/connorslade/mslicer/actions/workflows/build.yml/badge.svg)](https://github.com/connorslade/mslicer/actions/workflows/build.yml) [![Latest Version](https://img.shields.io/crates/v/msla_format)](https://crates.io/crates/msla_format) [![Downloads](https://img.shields.io/crates/d/msla_format?label=Downloads)](https://crates.io/crates/msla_format)

Library for encoding and decoding common masked stereolithography (MSLA) file formats.
This crate is a collection of format implementation extracted from my [mslicer](https://github.com/connorslade/mslicer) project, an open source slicer for MSLA printers.

## Supported Formats

- [ChituBox v5 (`.ctb`)](ctb)
- [Elegoo v3.0 (`.goo`)](goo)
- [NanoDLP (`.nanodlp`)](nanodlp)

## Run Length Encoding

Because resin printers often have very high resolution displays/masks it would be impractical to store layer data uncompressed, so for this reason all of the supported formats make use of some for of run length encoding (RLE).
This is why the interface for all the layer encoders lets you add runs of values.

It is important to note that you must define a value for every pixel.
This is because (on my printer at least) the buffer that each layer is decoded into is initially uninitialized.
So if the last run doesn’t fill the buffer, the printer will just print whatever was in the buffer before which just makes a huge mess (theoretically of course).

## Examples

For some real world examples, check out the following links to my mslicer project source code:

- [Slicing a triangular mesh into a layer](https://github.com/connorslade/mslicer/blob/5b4401a550dcc5cea8094d28cefdff45355aa39b/slicer/src/slicer/slice_raster.rs#L17)
- [Decoding a sliced file for a layer preview](https://github.com/connorslade/mslicer/blob/5b4401a550dcc5cea8094d28cefdff45355aa39b/mslicer/src/windows/slice_operation.rs#L199)

### Decode

Decode a `.ctb` file from disk and saves all of its layers as PNGs (using the built-in RLE optimized PNG encoder).

```rust
use std::fs;
use msla_format::{
    container::rle::png::{ColorType, PngEncoder},
    ctb,
    serde::{DynamicSerializer, SliceDeserializer},
};

let bytes = fs::read("out.ctb").unwrap();
let mut des = SliceDeserializer::new(&bytes);
let file = ctb::File::deserialize(&mut des).unwrap();
println!("{file:?}");

for (i, layer) in file.layers.iter().enumerate() {
    let decoder = ctb::LayerDecoder::new(&layer.data);
    
    let mut ser = DynamicSerializer::new();
    let mut png = PngEncoder::new(&mut ser, ColorType::Grayscale, file.resolution);
    png.write_image_data(decoder.collect());
    png.write_end();
    
    fs::write(format!("layer_{i}.png"), ser.into_inner()).unwrap();
}
```

### Encode

Encode a blank layer with default settings to a `.goo` file.

```rust
use std::fs::File;
use msla_format::{
    EncodableLayer, goo,
    serde::WriterSerializer,
    slice::{SliceConfig, SliceResult},
};

let config = SliceConfig::default();
let pixels = config.platform_resolution.x * config.platform_resolution.y;

let mut layer = goo::LayerEncoder::new();
layer.add_run(pixels as u64, 0);
let layer = EncodableLayer::finish(layer, 0, &config);

let file = goo::File::from_slice_result(SliceResult {
    layers: vec![layer],
    voxels: 0,
    slice_config: &config,
});

let mut ser = WriterSerializer::new(File::create("out.goo").unwrap());
file.serialize(&mut ser);
```
