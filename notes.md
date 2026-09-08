# Notes

This is just a place for me to remember and plan features I want to add and bugs to fix.
Feel free to help out :eyes:.

## Bugs

- correctly calculate print time with exposure overrides
- incorrect loaded in time when loading sliced file

## Features

- support generation
    - allow interacting with (deleting) supports place manually or automatically
    - for support generation pick less steep angles for the top part of the support
    - support presets, instead of having to mess with all the sliders to change the size
    - dont place supports on points touching the build plate
        - ask you to raise the model before supporting
    - slice supports
    - support placement tool instead of just a checkbox... or maybe a key to hold down while clicking
    - options to only generate some combination of point, edge, and face overhangs
    - transform supports with model
    - don't simplify a cylinder to a segment during intersection testing.
      either use a different algorithm or sample multiple points around the edges.
- put all models in the same segments1d to improve slicing times with supports?
- dont repaint every frame (or at least when unfocused)
- save old versions of config files when being overwritten (invalid config). or like just ignore the broken values.
- mesh export (useful for converting sliced to mesh or other mesh generator tools)
- phonographic record generator (it might be possible :eyes:)
  - apply RAII pre-emphasis
  - allow picking stereo or mono
  - generate async
  - create manifold mesh
- refactor format crates (they're kinda a mess rn)
- add well documented .zip and .bin output formats (closes #29)
- task cancellation mechanism
- release updated msla_format

## Documentation

- getting started video guide? (i do need to update that video in the readme)
- color internal and external links differently
- update getting started guide
  - changed exposure config component
  - change AA config
- document on generating and printing phonograph records

## Maybe

Features that would be cool, but are a bit out of scope for now.

- figure smth out for optimizing orientation
- improved auto layout supporting concave NFPs?
- built-in mesh subdivision
- render units as fractions with custom font
- allow manually editing the pixels in slice preview?
- 3d model packing for sls type printers?
- optimize nanodlp loading with custom png decoder?...
- resample sliced file to different sizes / resolutions?
- multiple workspaces per project
- optimize elephant foot post processing (deprecated post processor)
- split mslicer crate?
  - mslicer (startup + system stuff)
  - mslicer_core (app + project + task?)
  - mslicer_render (render)
  - mslicer_ui (window + ui)
