# Multi Stage Lift

- [ ] Figure out UI (see below)

## Figureing

### Current

```plain
Exposure  3.0s  @ 100% □ 3.0s
Lift      5.0mm @ 330.0 mm/min ↑
                  330.0 mm/min ↓
```

### Ideas

```plain
Exposure  3.0s  @ 100%
Delay     Pre   @ 5.0s
          Post  @ 1.0s
          Layer @ 0.0s

Lift      Lower @ 1.0mm 330.0 mm/min
          Upper @ 5.0mm 330.0 mm/min
Retract   Lower @ 1.0mm 330.0 mm/min
          Upper @ 5.0mm 330.0 mm/min
```

- Is the 'Layer Delay' useful for anything? If not omit it.
- For the lift config, the lower/upper or detach/refill modement stage distances must sum to be equal.
  - Perhapse a percentage would be easier to work with?
  - Should it be 'Direction › Stage' or 'Stage › Direction'?
