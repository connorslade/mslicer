# Features

- support generation
    - support presets, instead of having to mess with all the sliders to change the size
    - dont place supports on points touching the build plate
    - options to only generate some combination of point, edge, and face overhangs 
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
- render scale!
- only render viewport on change (will require `scene_dirty` flag)
- temporal accumulation (w jitter), sucks but ig that's just how you do rendering these days
- set fov in °
- run post processors on tool generated layers
