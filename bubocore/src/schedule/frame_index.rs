use crate::{
    clock::{Clock, SyncTime},
    scene::line::Line,
};

pub fn calculate_frame_index(
    clock: &Clock,
    scene_length: usize,
    line: &Line,
    date: SyncTime,
) -> (usize, usize, usize, SyncTime, SyncTime) {
    let effective_loop_length_beats = line.custom_length.unwrap_or(scene_length as f64);

    if effective_loop_length_beats <= 0.0 {
        return (usize::MAX, usize::MAX, 0, SyncTime::MAX, SyncTime::MAX);
    }

    let current_absolute_beat = clock.beat_at_date(date);
    if current_absolute_beat < 0.0 {
        return (usize::MAX, usize::MAX, 0, SyncTime::MAX, SyncTime::MAX);
    }

    let beat_in_effective_loop = current_absolute_beat % effective_loop_length_beats;
    let loop_iteration = current_absolute_beat.div_euclid(effective_loop_length_beats) as usize;

    let effective_start_frame = line.get_effective_start_frame();
    let effective_num_frames = line.get_effective_num_frames();

    if effective_num_frames == 0 {
        return (usize::MAX, loop_iteration, 0, SyncTime::MAX, SyncTime::MAX);
    }

    let mut cumulative_beats_in_line = 0.0;
    for frame_idx_in_range in 0..effective_num_frames {
        let absolute_frame_index = effective_start_frame + frame_idx_in_range;

        let speed_factor = if line.speed_factor == 0.0 {
            1.0
        } else {
            line.speed_factor
        };
        let single_rep_len_beats = line.frame_len(absolute_frame_index) / speed_factor;
        let total_repetitions = line
            .frame_repetitions
            .get(absolute_frame_index)
            .copied()
            .unwrap_or(1)
            .max(1);
        let total_frame_len_beats = single_rep_len_beats * total_repetitions as f64;

        if single_rep_len_beats <= 0.0 {
            continue;
        }

        let frame_end_beat_in_line = cumulative_beats_in_line + total_frame_len_beats;

        if beat_in_effective_loop >= cumulative_beats_in_line
            && beat_in_effective_loop < frame_end_beat_in_line
        {
            let beat_within_frame = beat_in_effective_loop - cumulative_beats_in_line;
            let current_repetition_index =
                (beat_within_frame / single_rep_len_beats).floor().max(0.0) as usize;
            let current_repetition_index = current_repetition_index.min(total_repetitions - 1);

            let absolute_beat_at_loop_start =
                loop_iteration as f64 * effective_loop_length_beats;
            let frame_first_rep_start_beat_absolute =
                absolute_beat_at_loop_start + cumulative_beats_in_line;
            let current_rep_start_beat_absolute = frame_first_rep_start_beat_absolute
                + (current_repetition_index as f64 * single_rep_len_beats);
            let current_repetition_start_date =
                clock.date_at_beat(current_rep_start_beat_absolute);

            let current_rep_end_beat_in_line = cumulative_beats_in_line
                + (single_rep_len_beats * (current_repetition_index + 1) as f64);
            let remaining_beats_in_rep = current_rep_end_beat_in_line - beat_in_effective_loop;
            let remaining_micros_in_rep = clock.beats_to_micros(remaining_beats_in_rep);

            let remaining_beats_in_loop = effective_loop_length_beats - beat_in_effective_loop;
            let remaining_micros_in_loop = clock.beats_to_micros(remaining_beats_in_loop);

            let next_event_delay = remaining_micros_in_rep.min(remaining_micros_in_loop);

            return (
                absolute_frame_index,
                loop_iteration,
                current_repetition_index,
                current_repetition_start_date,
                next_event_delay,
            );
        }

        cumulative_beats_in_line += total_frame_len_beats;
    }

    let remaining_beats_in_loop = effective_loop_length_beats - beat_in_effective_loop;
    let remaining_micros_in_loop = clock.beats_to_micros(remaining_beats_in_loop);
    (
        usize::MAX,
        loop_iteration,
        0,
        SyncTime::MAX,
        remaining_micros_in_loop,
    )
}