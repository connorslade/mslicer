# Getting Started

Welcome to mslicer!

Currently, mslicer can only output `.goo` files, which is a format spsific to [ELEGOO](https://www.elegoo.com) resin printers. In the future I plan to add support for other formats, but in the meantime you can use [UVTools](https://github.com/sn4k3/UVtools) to convert between formats.

If you are using mslicer for the first time, in order for the sliced results to be loadable by your printer, you will first need to configure the platform resolution and build volume in the `Slice Config` panel on the left. The defaults are for the ElEGOO [Saturn 3 Ultra](https://us.elegoo.com/products/elegoo-saturn-3-ultra-resin-3d-printer-12k). If these values are wrong, your printer may fail to load the sliced .goo file without even showing an error message.

mslicer can load `.stl` and `.obj` models, either by finding the `Import Model` button under the File menu, or by dragging the files onto the window. For now you can load the built in test model (it's the [Utah Teapot](https://en.wikipedia.org/wiki/Utah_teapot)) by pressing `Ctrl+T`.

To be continued
