# `slicer`

This crate contains the types and algorithms to efficiently slice a mesh and some other stuff for post processing and support generation.

## Command Line interface

This crate also exposes a CLI for slicing models, open the dropdown below to view the help page.

Multiple meshes can be added by using the `--mesh` argument more than once.
If you want to change any properties of the mesh like position, rotation, or scale, you can use the flag followed by a 3D vector (`x,y,z`).
These flags will modify the mesh defined most recently.
See the example below.

```bash
$ slicer --mesh teapot.stl --position 0,0,-0.05 --scale 2,2,2 --mesh frog.stl --position 100,0,0 output.goo
```

<details>
  <summary>CLI Help</summary>

```plain
mslicer command line interface

Usage: slicer [OPTIONS] <--mesh <MESH>|--position <POSITION>|--rotation <ROTATION>|--scale <SCALE>> <OUTPUT>

Arguments:
  <OUTPUT>  File to save sliced result to. Currently only .goo files can be generated

Options:
      --platform-resolution <PLATFORM_RESOLUTION>
          Resolution of the printer mask display in pixels [default: "11520, 5120"]
      --platform-size <PLATFORM_SIZE>
          Size of the printer display / platform in mm [default: "218.88, 122.904, 260.0"]
      --layer-height <LAYER_HEIGHT>
          Layer height in mm [default: 0.05]
      --first-layers <FIRST_LAYERS>
          Number of 'first layers'. These are layers that obey the --first- exposure config flags [default: 3]
      --transition-layers <TRANSITION_LAYERS>
          Number of transition layers. These are layers that interpolate from the first layer config to the default config [default: 10]
      --exposure-time <EXPOSURE_TIME>
          Layer exposure time in seconds [default: 3]
      --lift-distance <LIFT_DISTANCE>
          Distance to lift the platform after exposing each regular layer, in mm [default: 5]
      --lift-speed <LIFT_SPEED>
          The speed to lift the platform after exposing each regular layer, in mm/min [default: 65]
      --retract-speed <RETRACT_SPEED>
          The speed to retract (move down) the platform after exposing each regular layer, in mm/min [default: 150]
      --first-exposure-time <FIRST_EXPOSURE_TIME>
          First layer exposure time in seconds [default: 30]
      --first-lift-distance <FIRST_LIFT_DISTANCE>
          Distance to lift the platform after exposing each first layer, in mm [default: 5]
      --first-lift-speed <FIRST_LIFT_SPEED>
          The speed to lift the platform after exposing each first layer, in mm/min [default: 65]
      --first-retract-speed <FIRST_RETRACT_SPEED>
          The speed to retract (move down) the platform after exposing each first layer, in mm/min [default: 150]
      --preview <PREVIEW>
          Path to a preview image, will be scaled as needed
      --mesh <MESH>
          Path to a .stl or .obj file
      --position <POSITION>
          Location of the bottom center of model bounding box. The origin is the center of the build plate
      --rotation <ROTATION>
          Rotation of the model in degrees, roll, pitch, yaw
      --scale <SCALE>
          Scale of the model along the X, Y, and Z axes
  -h, --help
          Print help
```

</details>
