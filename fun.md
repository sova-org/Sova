# BuboCore ↔ Ableton Link Timing Alignment Fixes

## Priority Matrix: Ease vs. Precision Impact

### 🟢 High Impact, Easy Fix

**1. Reduce Scheduled Drift (`schedule.rs:35`)** ✅ DONE
- **Ease**: ⭐⭐⭐⭐⭐ (Single constant change)
- **Precision Impact**: ⭐⭐⭐⭐⭐ (Reduces consistent 30ms phase offset)
- **Fix**: Changed `SCHEDULED_DRIFT` from 30ms to 10ms
- **Risk**: Low - just reduces lookahead buffer

**2. Increase Timebase Calibration Frequency (`world.rs:85`)** ✅ DONE
- **Ease**: ⭐⭐⭐⭐⭐ (Single constant change)
- **Precision Impact**: ⭐⭐⭐⭐ (Reduces drift accumulation)
- **Fix**: Changed calibration interval from 1s to 100ms
- **Risk**: Minimal CPU overhead increase

### 🟡 Medium Impact, Medium Effort

**3. Improve Transport Start Synchronization (`schedule.rs:394-416`)**
- **Ease**: ⭐⭐⭐ (Requires logic modification)
- **Precision Impact**: ⭐⭐⭐⭐ (Better initial alignment)
- **Fix**: Use Link's current phase for smarter start timing
- **Risk**: Medium - affects transport behavior

**4. Add Phase Alignment Diagnostics (`clock.rs`)**
- **Ease**: ⭐⭐⭐⭐ (Non-invasive debug function)
- **Precision Impact**: ⭐⭐⭐ (Enables measurement and tuning)
- **Fix**: Add debug logging to compare BuboCore vs Link phases
- **Risk**: None - debug only

### 🔴 Lower Priority

**5. Frame Index Calculation Optimization (`schedule.rs:148-264`)**
- **Ease**: ⭐⭐ (Complex floating-point refactoring)
- **Precision Impact**: ⭐⭐⭐⭐⭐ (Eliminates cumulative timing drift)
- **Fix**: Fixed-point arithmetic for cumulative calculations, integer beat tracking
- **Risk**: High - affects core scheduling logic
- **Details**: See `timing-precision-analysis.md` for comprehensive analysis

## Recommended Implementation Order

1. ✅ **DONE**: Reduced `SCHEDULED_DRIFT` to 10ms
2. ✅ **DONE**: Increased calibration frequency to 100ms  
3. **Next**: Add phase diagnostics for measurement
4. **Finally**: Improve transport start logic if needed

## Expected Results

- **Immediate**: 20ms closer alignment with Ableton
- **Short-term**: More stable synchronization over time
- **Long-term**: Sub-millisecond precision alignment

## Testing Protocol

```bali
-- Add to your BaLi script for alignment testing
every 4 beats do
  print("Beat: " .. beat() .. " Phase: " .. (beat() % 4))
end
```

Compare BuboCore output with Ableton's beat counter to measure improvement.

## Deep Dive: Floating-Point Precision Issues

See `timing-precision-analysis.md` for a comprehensive analysis of floating-point error accumulation in BuboCore's timing system. The most critical issue is cumulative beat accumulation in `frame_index()` which can cause millisecond-level drift over time.

**Key Finding**: The `cumulative_beats_in_line` variable in `schedule.rs:178` accumulates floating-point errors that compound over thousands of frames, potentially causing the phase misalignment you're experiencing with Ableton.