pub fn smooth(buffer: &mut Vec<f32>, source: &[f32], factor: f32) {
    buffer.resize(source.len(), 0.0);
    for (b, &s) in buffer.iter_mut().zip(source) {
        *b = *b * factor + s * (1.0 - factor);
    }
}

/// Peak hold with exponential decay. Snaps up instantly to new peaks, decays
/// multiplicatively otherwise. Operates on linear magnitudes.
pub fn decay_peaks(peaks: &mut Vec<f32>, source: &[f32], decay: f32) {
    peaks.resize(source.len(), 0.0);
    for (p, &s) in peaks.iter_mut().zip(source.iter()) {
        if s >= *p {
            *p = s;
        } else {
            *p *= decay;
        }
    }
}

/// Downsample using Largest-Triangle-Three-Buckets (LTTB).
/// Preserves waveform shape far better than min/max decimation.
pub fn downsample_lttb(output: &mut Vec<f32>, source: &[f32], target_len: usize) {
    output.clear();
    let n = source.len();
    if n == 0 || target_len == 0 {
        return;
    }
    if n <= target_len {
        output.extend_from_slice(source);
        return;
    }
    output.reserve(target_len);

    // Always keep first point
    output.push(source[0]);

    let bucket_count = target_len - 2;
    let bucket_size = (n - 2) as f64 / bucket_count as f64;
    let mut prev_selected = 0usize;

    for i in 0..bucket_count {
        let bucket_start = ((i as f64 * bucket_size) as usize) + 1;
        let bucket_end = (((i + 1) as f64 * bucket_size) as usize + 1).min(n - 1);

        // Average of the *next* bucket (look-ahead for triangle area)
        let next_start = bucket_end;
        let next_end = if i + 2 < bucket_count {
            (((i + 2) as f64 * bucket_size) as usize + 1).min(n - 1)
        } else {
            n - 1
        };
        let next_len = (next_end - next_start + 1).max(1) as f32;
        let avg_next: f32 = source[next_start..=next_end].iter().sum::<f32>() / next_len;
        let avg_next_x = (next_start + next_end) as f32 * 0.5;

        let prev_x = prev_selected as f32;
        let prev_y = source[prev_selected];

        let mut best_idx = bucket_start;
        let mut best_area = -1.0_f32;
        for (j, &sample) in source[bucket_start..=bucket_end].iter().enumerate() {
            let j = j + bucket_start;
            let area = ((prev_x - avg_next_x) * (sample - prev_y)
                - (prev_x - j as f32) * (avg_next - prev_y))
                .abs();
            if area > best_area {
                best_area = area;
                best_idx = j;
            }
        }

        output.push(source[best_idx]);
        prev_selected = best_idx;
    }

    // Always keep last point
    output.push(source[n - 1]);
}

/// Phosphor-style trace: tracks the min/max envelope of where the waveform
/// has been. Instantly expands to new extremes, slowly decays back.
pub fn apply_trace(trace: &mut Vec<(f32, f32)>, current: &[f32], persistence: f32) {
    if trace.len() != current.len() {
        trace.clear();
        trace.extend(current.iter().map(|&v| (v, v)));
        return;
    }

    let keep = persistence.clamp(0.0, 0.98);
    let take = 1.0 - keep;

    for (t, &v) in trace.iter_mut().zip(current) {
        t.0 = if v < t.0 { v } else { t.0 * keep + v * take };
        t.1 = if v > t.1 { v } else { t.1 * keep + v * take };
    }
}

pub fn align_trigger(buffer: &mut Vec<f32>, source: &[f32]) {
    buffer.clear();
    if source.is_empty() {
        return;
    }

    let len = source.len();
    if len < 3 {
        buffer.extend_from_slice(source);
        return;
    }

    let search_start = len / 16;
    let search_end = (len / 3).max(search_start + 1).min(len - 1);
    let mut best_index = None;
    let mut best_score = f32::MIN;

    for i in search_start..search_end {
        let a = source[i];
        let b = source[i + 1];
        if a <= 0.0 && b > 0.0 {
            let slope = b - a;
            let closeness = 1.0 - (a.abs() + b.abs()).min(1.0);
            let score = slope * 2.0 + closeness;
            if score > best_score {
                best_score = score;
                best_index = Some(i);
            }
        }
    }

    if let Some(start) = best_index {
        buffer.extend_from_slice(&source[start..]);
        buffer.extend_from_slice(&source[..start]);
    } else {
        buffer.extend_from_slice(source);
    }
}
