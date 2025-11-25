# Getting Started

Welcome to mslicer!

## Setup

In order for the sliced output to be loadable by your printer, you will first need to configure the platform resolution and build volume in the `Slice Config` panel on the left. The defaults are for the [ElEGOO Saturn 3 Ultra](https://us.elegoo.com/products/elegoo-saturn-3-ultra-resin-3d-printer-12k). If these values are wrong, your printer may fail to load the output without even showing an error message.

## Models

mslicer can load `.stl` and `.obj` models. Add one by going to File  Import Model or drag and drop a model file into the workspace. For now, you can load the built-in test model (the [Utah Teapot](https://en.wikipedia.org/wiki/Utah_teapot)) by pressing `Ctrl+T`.

To move around the viewport, scroll to move towards or away from the target point, drag with left click to orbit the target point, and drag with right click to translate the target point.

Each model in your project is listed in the `Models` panel. By clicking the arrow button next to a model, you can access all its properties, including size, position, and rotation, as well as run actions like deleting the model or aligning it to the bed.

## Slicing

After starting a slice operation, a new panel will open showing the operation progress. However, it shouldn't be open for long because (as far as I know) mslicer is the fastest MSLA slicer currently available (:p). You will then be presented with a slice preview. You can drag to pan, scroll to zoom, and scrub through the slider on the left to look through each layer.
