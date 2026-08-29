# Notes

This is just a place for me to remember and plan features I want to add and bugs to fix.

## Bugs

- fix auto-layout handling of rotated and scaled models
- don't crash when interacting with remote print after the printer has disconnected
- reference count BVH and half_edge meshes

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
- figure smth out for optimizing orientation
- proper morphological aa
- allow changing broadcast address in UI
- put all models in the same segments1d to improve slicing times with supports?
- multiple workspaces per project
- built-in mesh subdivision
- optimize elephant foot post processing
- allow manually editing the pixels in slice preview?
