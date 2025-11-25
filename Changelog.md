# Changelog

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
