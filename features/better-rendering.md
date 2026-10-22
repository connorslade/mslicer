# Better Rendering

- reduce power usage (dont always re-render, don't re-do work)
- render scale!
- only render viewport on change (will require `scene_dirty` flag)
- temporal accumulation (w jitter), sucks but ig that's just how you do rendering these days
