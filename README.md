# Notes

This is just a place for me to remember and plan features I want to add and bugs to fix.
Feel free to help out :eyes:.

# Bugs

- check if slicing with spacemouse button causes crash
- resizing the window is laggy (bc of all the texture reallocs!)
- fix long file names in remote print causing problems...
- some custom components dont work well on light mode
- if you make changes to a project in the time that it takes for another project to load, your changes will be deleted
  - lock while loading project or saving before close!
- don't allow starting multiple build acc structure tasks!
- image layer: attempt to subtract with overflow
- image load task so you dont just see nothing happen after loading an image
- clicking on tools menu moves it above any floating windows
- trying to load a broken image file crashes the whole program
- limit sliced panel min width

# Features

- support generation
- put all models in the same segments1d to improve slicing times with supports?
- save old versions of config files when being overwritten (invalid config). or
  like just ignore the broken values.
- task cancellation mechanism
- release updated msla_format
- different support colors (also diff default styles)
- sliced post processing
  - remove islands ←
  - exposure remap
- two stage lift config!! (how have i left this unimplemented for so long!)
- slice config presets
- run post processors on tool generated layers
- interpolate remote print eta?
- exposure remapping with eq or lut
