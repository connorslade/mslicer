# Better Rendering

- temporal accumulation (w jitter), sucks but ig that's just how you do rendering these days
  - only render viewport on change (will require `scene_dirty` flag)
  - reprojection?
  - use better sampling function
  - correct mix factor (based on frame)
  - reduce accm overhead when TAA is disabled
- reduce power usage (dont always re-render, don't re-do work)
  - also needs the dirty flag
