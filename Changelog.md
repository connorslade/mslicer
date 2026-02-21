# Changelog

## v0.5.0 &mdash; February 21st, 2026

- Detect and show warnings for
  - Non-manifold meshes
  - Meshes that extend outside the printable volume
- Support generation progress
  - Visualize overhanging faces and vertices
  - Basic (unfinished) manually placed support routing
- Use async tasks for more operations
  - Loading meshes
  - Saving and loading projects
  - Building acceleration structures
  - File pick/save dialogs
  - Exporting sliced files (mostly needed for compressing .nanodlp)
- Color backfaces grey in rendered view
- Don't show multiple copies of layer in slice preview when panning far to the right
- Printer presets (print volume, resolution)
- Reduce memory usage of loaded models
- Reuse GPU buffers when possible (instead of reallocating)
- Don't duplicate mesh data in project save file
- Custom project and mesh (de)serialization with progress monitoring
- Added task panel to view the progress of running async tasks
- Run post processing operations asynchronously with their own progress bars
- Added support for the NanoDLP format (.nanodlp)
- Allow setting/updating default slice config ([#5](https://github.com/connorslade/mslicer/issues/5))
- Remember the file that was opened for future save operations
- Track changes to settings for undo and redo (under edit in the top bar)
- Use metric prefixes for slice config draggers
- Allow using any supported output format with the CLI slicer
- More robust slicing (include scanline boundary hits)
- Show resin volume usage and estimated print time under slice preview
- Island detection task
- Fix invalid unit and color space transformations with type safe systems
- Use the 'windows' subsystem on Windows
- Remove unfinished support for translucent models
- Remove inconsistent slice preview pixel outlines (for now)

## v0.4.0 &mdash; November 25th, 2025

- Added support for the encrypted Chitu format (.ctb)

## v0.3.0 &mdash; November 15th, 2025

- Added support for SVGs as an output format
- Don't show normals for hidden models
- Always open Viewport window on startup
- Show outlines around the pixels when zoomed into the slice preview
- Recompute mesh normals by default (also removed the normal operations button)
- Generated meshes (supports) now have correct normals and face winding order

## v0.2.2 &mdash; June 14th, 2025

- Clamp grid size
- Fix intermittent crash due to mismatched texture formats between egui and renderer pipelines

## v0.2.1 &mdash; April 13, 2025

- Don't produce invalid results when models extend beyond build volume
- Made the slicer system usable as a standalone CLI application

## v0.2.0 &mdash; Feb 19, 2025

- Convert slice operation window to a dockable panel
- Render parts of models that go beyond the print volume red
- Remove the Stats panel and merge it into the Workspace panel
- Add documentation into the About panel
- Add random triangle color mesh render mode
- Persist panel layout between sessions. I also added a button to reset the UI layout in the Workspace panel.

## v0.1.0 &mdash; Feb 12, 2025

First release!

I haven't really changed much in the past like six months, but now mslicer is on Flathub.
