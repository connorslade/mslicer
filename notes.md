# Notes

This is just a place for me to remember and plan features I want to add and bugs to fix.
Feel free to help out :eyes:.

## Bugs

- check if slicing with spacemouse button causes crash
- resizing the window is laggy (bc of all the texture reallocs!)
- fix long file names in remote print causing problems...
- some custom components dont work well on light mode
- if you make changes to a project in the time that it takes for another project to load, your changes will be deleted
  - lock while loading project or saving before close!
- don't allow starting multiple build acc structure tasks
- fix weird supports OOB shading

## Features

- support generation
    - support presets, instead of having to mess with all the sliders to change the size
    - dont place supports on points touching the build plate
    - options to only generate some combination of point, edge, and face overhangs 
    - don't simplify a cylinder to a segment during intersection testing. either
      use a different algorithm or sample multiple points around the edges.
- put all models in the same segments1d to improve slicing times with supports?
- save old versions of config files when being overwritten (invalid config). or
  like just ignore the broken values.
- phonographic record generator (it might be possible :eyes:)
  - apply RAII pre-emphasis
  - create manifold mesh
- add well documented .zip and .bin output formats (closes #29)
- task cancellation mechanism
- release updated msla_format
- different support colors (also diff default styles)
- sliced post processing
  - remove islands ←
  - exposure remap
- reduce power usage (dont always re-render, don't re-do work)
- two stage lift config!! (how have i left this unimplemented for so long!)
- slice config presets
- configurable (and viewable) key binds

## Documentation

- getting started video guide? (i do need to update that video in the readme)
- color internal and external links differently
- update getting started guide
  - changed exposure config component
  - change AA config
  - moved update check freq
  - Rename 'First Layers' to 'Bottom Layers'
  - note where to find quick layout spacing
  - just go through the whole thing a lot of its probably outdated now...
- document on generating and printing phonograph records
- add phosphor icons in doc pages (like i did on the pcb photolithography page)
- update mesh repair page (previously repairing non-manifold meshes)
- ultimate slicer benchmark (slice Thingi10K dataset)
- cleanup changelog

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
- slice b-rep?
  - at what point of mesh resolution is polygon intersection slower 
  - plot voxel error across different triangulations
- scripting???
- dont make 'Saturn 3 Ultra' the default profile
  - add an unselected option to force users to pick the right printer?
- custom post processor with dyn libs?
  - https://github.com/rodrimati1992/abi_stable_crates
  - https://github.com/ZettaScaleLabs/stabby
- use convex hull to re-calc bounds after transformation
- switch between multiple sliced files in slice preview?
- optimize island detection
- lower render resolution?
- game style camera (arrow keys to move)

---

- support presets and property configuration
- support serialization...
