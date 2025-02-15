# Getting Started

Welcome to mslicer!

Currently, mslicer can only output `.goo` files, which is a format spsific to [ELEGOO](https://www.elegoo.com) resin printers. In the future I plan to add support for other formats, but in the meantime you can use [UVTools](https://github.com/sn4k3/UVtools) to convert between formats.

## Setup

In order for the sliced output to be loadable by your printer, you will first need to configure the platform resolution and build volume in the `Slice Config` panel on the left. The defaults are for the [ElEGOO Saturn 3 Ultra](https://us.elegoo.com/products/elegoo-saturn-3-ultra-resin-3d-printer-12k). If these values are wrong, your printer may fail to load the output without even showing an error message.

## Models

mslicer can load `.stl` and `.obj` models. Add one by going to File  Import Model or drag and drop a model file into the workspace. For now you can load the built in test model (it's the [Utah Teapot](https://en.wikipedia.org/wiki/Utah_teapot)) by pressing `Ctrl+T`.

You can move around the viewport by scrolling on it to move towards or away from the target point, dragging with left click to orbit the target point, and dragging with right click to move the target point.

Each model in your project is listed in the `Models` panel. By clicking the arrow button next to a model, you can access all it's properties, including size, position, rotation as well as run actions like deleting the model, or aligning it to the bed.

If you're unfamiliar with normals, they are vectors at each vertex of a model that are perpendicular to the surface and point outward. Normals are crucial for the slicer to determine whether it is entering or exiting a model, especially when models intersect themselves or other models. Incorrect normals from imported `.stl` or `.obj` files can result in artifacts in the output. For this reason, under the `Normals` action you can either flip or recalculate the normals of a model.

## Slicing

After starting a slice operation, a new panel will open showing the operation progress. It shouldnt be open for long though because  (as far as I know) mslicer is the fastest MSLA slicer currently available. :p You will then be presented with a slice preview, you can drag to pan, scroll to zoom, and scrub through the the slider on the left to look through each layer.
