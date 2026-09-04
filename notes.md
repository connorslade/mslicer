# Notes

This is just a place for me to remember and plan features I want to add and bugs to fix.

## Bugs

- don't crash when interacting with remote print after the printer has disconnected (not confirmed after refactor)
- fix gpu decompress failing:
  `Each current dispatch group size dimension ([159078, 1, 1]) must be less or equal to 65535`

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
- multiple workspaces per project
- optimize elephant foot post processing
- allow manually editing the pixels in slice preview?
- dont repaint every frame (or at least when unfocused)
- speed up compilation
- save old versions of config files when being overwritten (invalid config). or like just ignore the broken values.

## Documentation

- getting started video guide? (i do need to update that video in the readme)
- color internal and external links differently
- update getting started guide
  - changed exposure config component
  - change AA config

## Maybe

Features that would be cool, but are a bit out of scope for now.

- figure smth out for optimizing orientation
- improved auto layout supporting concave NFPs?
- built-in mesh subdivision
