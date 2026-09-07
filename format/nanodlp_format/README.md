# `nanodlp_format`

Implementation of the `.nanodlp` format.
With a custom PNG encoder (deflate implementation) optimized for writing run length encoded layer data.

## Layer Encoding

The NanoDLP docs don't really go into how the actual layers are encoded, and it's kinda weird so I'll describe it here.
Each layer is a three channel (RGB8) PNG image where every three pixels in the grayscale layer are encoded as a single RGB color value.
For example if some row of a layer PNG was `[(0, 0, 255), (255, 255, 0)]`, it would just map to `[0, 0, 255, 255, 255, 0]`.
Genuinely no idea why it's done like this since its undefined in the case that the horizontal resolution is not divisible by three.

The NanoDLP slicer itself just crashes saying `image width should be divisible by 3 to avoid skewed distortions, off-by-one panics, and other side effects.` and UVTools also fails to load.
I don't know of any printers with horizontal resolutions that aren't divisible by three, so I guess that's fine?

If you know the reasoning behind this weird decision please let me know, I'm very curious.

## References

- <https://docs.nanodlp.com/manual/format>
- <https://docs.nano3dtech.com/manual/code>
