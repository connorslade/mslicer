# Presets

- Consider some way to pull preset values just for individual values. Would
  mainly be useful for exposure remapping. Maybe that should just get its own
  thing?

## Printer

- Unset printer by default to ensure people select their printer
  - They can pick a random or dummy printer for testing
- Show recomended output formats based on printer preset
    - Will require reworking how printer presets work. They should not be identified by the selected build volume / pixel count, they should be stored as an index into the preset table. (String would be fine)

## Slice Config & Tools

- Either project local or global (for installation)
- For Slice Config (maybe rework that name) and tools
- Consider how to reduce or improve the process of updating format versions
