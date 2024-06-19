# `goo_format`

Library for encoding and decoding Elegoo's proprietary `.goo` file format.
A binary is also included for inspecting and debugging the sliced goo files, with the option to dump the layers as png images.

If you ended up here trying to learn more about the format, make sure you read the [official format spec](https://github.com/elegooofficial/GOO).
Some things aren't mentioned in the spec like how everything is big-endian, the checksum is just the negated sum of all bytes in the payload, and the image data encoding specification is hard to follow, so look at [my implementation](src/encoded_layer.rs).
Also, if you use [ImHex](https://imhex.werwolv.net), I have created a pattern file ([goo.hexpat](goo.hexpat)) that may be helpful.
