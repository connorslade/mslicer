# Better Rendering

- use sep uniform buffers!!!
  - make a resources struct for every pass (FxaaResources) that contains the uniform, or use push constants?
- clear accm buffer (flag to composite) if scene is dirty
- reprojection?
- fix edge flicker
- use better sampling function
- correct mix factor (based on frame)
- allow toggling / configuring TAA settings in UI

---

- reduce power usage (dont always re-render, don't re-do work)
- render scale!
- only render viewport on change (will require `scene_dirty` flag)
- temporal accumulation (w jitter), sucks but ig that's just how you do rendering these days
- can we somehow get the fading effect? idk.
