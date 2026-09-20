# Notes

This is just a place for me to remember and plan features I want to add and bugs to fix.
Feel free to help out :eyes:.

## Bugs

- open slice preview as tab next to 3d view by default
- check if slicing with spacemouse button causes crash

## Features

- support generation
    - for support generation pick less steep angles for the top part of the support
    - support presets, instead of having to mess with all the sliders to change the size
    - dont place supports on points touching the build plate
        - ask you to raise the model before supporting?
    - options to only generate some combination of point, edge, and face overhangs 
    - don't simplify a cylinder to a segment during intersection testing. either
      use a different algorithm or sample multiple points around the edges.
- put all models in the same segments1d to improve slicing times with supports?
- dont repaint every frame (or at least when unfocused)
- save old versions of config files when being overwritten (invalid config). or
  like just ignore the broken values.
- phonographic record generator (it might be possible :eyes:)
  - apply RAII pre-emphasis
  - generate async
  - create manifold mesh
- add well documented .zip and .bin output formats (closes #29)
- task cancellation mechanism
- release updated msla_format
- game style camera (arrow keys to move)
- organize tools menu with categories?
- plot voxel error across different triangulations
- lower render resolution?
- different support colors (also diff default styles)

## Documentation

- getting started video guide? (i do need to update that video in the readme)
- color internal and external links differently
- update getting started guide
  - changed exposure config component
  - change AA config
  - moved update check freq
  - Rename 'First Layers' to 'Bottom Layers'
  - note where to find quick layout spacing
- document on generating and printing phonograph records
- add phosphor icons in doc pages (like i did on the pcb photolithography page)
- update mesh repair page (previously repairing non-manifold meshes)
- ultimate slicer benchmark (slice Thingi10K dataset)

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
- slice b-rep?
  - at what point of mesh resolution is polygon intersection slower 

## Defective Meshes

- [x] non-welded verts
- [x] holes
  - find edge loop and triangulate (somehow...)
- [ ] inconstant winding order
- [ ] non-manifold (not sure how to fix this thb)
- [ ] repeated faces?
