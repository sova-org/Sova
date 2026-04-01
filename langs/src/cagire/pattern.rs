use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PatHit {
    pub position: f64,
    pub gate: f64,
    pub alt: Option<Alt>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Alt {
    pub index: u16,
    pub count: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PatAnnotatedHit {
    pub position: f64,
    pub gate: f64,
    pub alt: Option<Alt>,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, Debug)]
struct SourceChar {
    ch: char,
    start: usize,
    end: usize,
}

trait PatternToken {
    fn ch(&self) -> char;
}

impl PatternToken for char {
    fn ch(&self) -> char {
        *self
    }
}

impl PatternToken for SourceChar {
    fn ch(&self) -> char {
        self.ch
    }
}

pub(crate) fn parse_pattern(input: &str) -> Result<Arc<[PatHit]>, String> {
    let chars: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
    let total = count_slots(&chars, 0, chars.len())?;
    if total == 0 {
        return Err("empty pattern".into());
    }
    let mut hits = Vec::new();
    collect_hits(&chars, 0, chars.len(), 0.0, 1.0, total, None, &mut hits)?;
    Ok(hits.into())
}

pub(crate) fn parse_pattern_annotated(input: &str) -> Result<Vec<PatAnnotatedHit>, String> {
    let chars: Vec<SourceChar> = input
        .char_indices()
        .filter_map(|(start, ch)| {
            if ch.is_whitespace() {
                None
            } else {
                Some(SourceChar {
                    ch,
                    start,
                    end: start + ch.len_utf8(),
                })
            }
        })
        .collect();
    let total = count_slots(&chars, 0, chars.len())?;
    if total == 0 {
        return Err("empty pattern".into());
    }
    let mut hits = Vec::new();
    collect_hits_annotated(&chars, 0, chars.len(), 0.0, 1.0, total, None, &mut hits)?;
    Ok(hits)
}

fn count_slots<T: PatternToken>(chars: &[T], start: usize, end: usize) -> Result<usize, String> {
    let mut count = 0usize;
    let mut i = start;
    while i < end {
        match chars[i].ch() {
            'x' | '.' | '-' => {
                let (repeat, consumed) = parse_repeat(chars, i + 1, end);
                count += repeat;
                i += 1 + consumed;
            }
            '[' => {
                let close = find_closing(chars, i)?;
                let (repeat, consumed) = parse_repeat(chars, close + 1, end);
                count += repeat;
                i = close + 1 + consumed;
            }
            '<' => {
                let close = find_closing(chars, i)?;
                let (repeat, consumed) = parse_repeat(chars, close + 1, end);
                count += repeat;
                i = close + 1 + consumed;
            }
            ']' => return Err("unexpected ']'".into()),
            '>' => return Err("unexpected '>'".into()),
            '|' => return Err("'|' outside alternation '<...>'".into()),
            '*' => return Err("'*' without preceding group".into()),
            c => return Err(format!("unknown pattern char '{c}'")),
        }
    }
    Ok(count)
}

fn collect_hits(
    chars: &[char],
    start: usize,
    end: usize,
    offset: f64,
    width: f64,
    total: usize,
    alt: Option<Alt>,
    hits: &mut Vec<PatHit>,
) -> Result<(), String> {
    let slot_width = width / total as f64;
    let mut slot = 0usize;
    let mut i = start;

    while i < end {
        match chars[i] {
            'x' => {
                let (repeat, rep_consumed) = parse_repeat(chars, i + 1, end);
                if rep_consumed > 0 {
                    for _ in 0..repeat {
                        let pos = offset + slot as f64 * slot_width;
                        hits.push(PatHit {
                            position: pos,
                            gate: slot_width,
                            alt: alt.clone(),
                        });
                        slot += 1;
                    }
                    i += 1 + rep_consumed;
                } else {
                    let pos = offset + slot as f64 * slot_width;
                    let dashes = count_ahead(chars, i + 1, end, '-');
                    let gate = slot_width * (1 + dashes) as f64;
                    hits.push(PatHit {
                        position: pos,
                        gate,
                        alt: alt.clone(),
                    });
                    i += 1 + dashes;
                    slot += 1 + dashes;
                }
            }
            '.' => {
                let (repeat, consumed) = parse_repeat(chars, i + 1, end);
                slot += repeat;
                i += 1 + consumed;
            }
            '-' => {
                return Err("'-' without preceding 'x'".into());
            }
            '[' => {
                let close = find_closing(chars, i)?;
                let (repeat, consumed) = parse_repeat(chars, close + 1, end);
                let inner_start = i + 1;
                let inner_end = close;
                if inner_start == inner_end {
                    return Err("empty subdivision '[]'".into());
                }
                let sub_total = count_slots(chars, inner_start, inner_end)?;
                if sub_total == 0 {
                    return Err("empty subdivision '[]'".into());
                }
                for _ in 0..repeat {
                    let sub_offset = offset + slot as f64 * slot_width;
                    collect_hits(
                        chars,
                        inner_start,
                        inner_end,
                        sub_offset,
                        slot_width,
                        sub_total,
                        alt.clone(),
                        hits,
                    )?;
                    slot += 1;
                }
                i = close + 1 + consumed;
            }
            '<' => {
                let close = find_closing(chars, i)?;
                let (repeat, consumed) = parse_repeat(chars, close + 1, end);
                let inner_start = i + 1;
                let inner_end = close;
                let alternatives = split_alternatives(chars, inner_start, inner_end)?;
                if alternatives.is_empty() {
                    return Err("empty alternation '<>'".into());
                }
                let alt_count = alternatives.len() as u16;
                for _ in 0..repeat {
                    let sub_offset = offset + slot as f64 * slot_width;
                    for (alt_idx, &(seg_start, seg_end)) in alternatives.iter().enumerate() {
                        let sub_total = count_slots(chars, seg_start, seg_end)?;
                        if sub_total == 0 {
                            continue;
                        }
                        let a = Alt {
                            index: alt_idx as u16,
                            count: alt_count,
                        };
                        collect_hits(
                            chars,
                            seg_start,
                            seg_end,
                            sub_offset,
                            slot_width,
                            sub_total,
                            Some(a),
                            hits,
                        )?;
                    }
                    slot += 1;
                }
                i = close + 1 + consumed;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn collect_hits_annotated(
    chars: &[SourceChar],
    start: usize,
    end: usize,
    offset: f64,
    width: f64,
    total: usize,
    alt: Option<Alt>,
    hits: &mut Vec<PatAnnotatedHit>,
) -> Result<(), String> {
    let slot_width = width / total as f64;
    let mut slot = 0usize;
    let mut i = start;

    while i < end {
        match chars[i].ch {
            'x' => {
                let (repeat, rep_consumed) = parse_repeat(chars, i + 1, end);
                if rep_consumed > 0 {
                    let token_end = chars[i + rep_consumed].end;
                    for _ in 0..repeat {
                        let pos = offset + slot as f64 * slot_width;
                        hits.push(PatAnnotatedHit {
                            position: pos,
                            gate: slot_width,
                            alt: alt.clone(),
                            start: chars[i].start,
                            end: token_end,
                        });
                        slot += 1;
                    }
                    i += 1 + rep_consumed;
                } else {
                    let pos = offset + slot as f64 * slot_width;
                    let dashes = count_ahead(chars, i + 1, end, '-');
                    let gate = slot_width * (1 + dashes) as f64;
                    hits.push(PatAnnotatedHit {
                        position: pos,
                        gate,
                        alt: alt.clone(),
                        start: chars[i].start,
                        end: chars[i + dashes].end,
                    });
                    i += 1 + dashes;
                    slot += 1 + dashes;
                }
            }
            '.' => {
                let (repeat, consumed) = parse_repeat(chars, i + 1, end);
                slot += repeat;
                i += 1 + consumed;
            }
            '-' => {
                return Err("'-' without preceding 'x'".into());
            }
            '[' => {
                let close = find_closing(chars, i)?;
                let (repeat, consumed) = parse_repeat(chars, close + 1, end);
                let inner_start = i + 1;
                let inner_end = close;
                if inner_start == inner_end {
                    return Err("empty subdivision '[]'".into());
                }
                let sub_total = count_slots(chars, inner_start, inner_end)?;
                if sub_total == 0 {
                    return Err("empty subdivision '[]'".into());
                }
                for _ in 0..repeat {
                    let sub_offset = offset + slot as f64 * slot_width;
                    collect_hits_annotated(
                        chars,
                        inner_start,
                        inner_end,
                        sub_offset,
                        slot_width,
                        sub_total,
                        alt.clone(),
                        hits,
                    )?;
                    slot += 1;
                }
                i = close + 1 + consumed;
            }
            '<' => {
                let close = find_closing(chars, i)?;
                let (repeat, consumed) = parse_repeat(chars, close + 1, end);
                let inner_start = i + 1;
                let inner_end = close;
                let alternatives = split_alternatives(chars, inner_start, inner_end)?;
                if alternatives.is_empty() {
                    return Err("empty alternation '<>'".into());
                }
                let alt_count = alternatives.len() as u16;
                for _ in 0..repeat {
                    let sub_offset = offset + slot as f64 * slot_width;
                    for (alt_idx, &(seg_start, seg_end)) in alternatives.iter().enumerate() {
                        let sub_total = count_slots(chars, seg_start, seg_end)?;
                        if sub_total == 0 {
                            continue;
                        }
                        let a = Alt {
                            index: alt_idx as u16,
                            count: alt_count,
                        };
                        collect_hits_annotated(
                            chars,
                            seg_start,
                            seg_end,
                            sub_offset,
                            slot_width,
                            sub_total,
                            Some(a),
                            hits,
                        )?;
                    }
                    slot += 1;
                }
                i = close + 1 + consumed;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn count_ahead<T: PatternToken>(chars: &[T], from: usize, end: usize, target: char) -> usize {
    let mut n = 0;
    let mut i = from;
    while i < end && chars[i].ch() == target {
        n += 1;
        i += 1;
    }
    n
}

fn find_closing<T: PatternToken>(chars: &[T], open_pos: usize) -> Result<usize, String> {
    let open = chars[open_pos].ch();
    let close = match open {
        '[' => ']',
        '<' => '>',
        _ => unreachable!(),
    };
    let mut depth = 1;
    let mut i = open_pos + 1;
    while i < chars.len() {
        if chars[i].ch() == open {
            depth += 1;
        } else if chars[i].ch() == close {
            depth -= 1;
            if depth == 0 {
                return Ok(i);
            }
        }
        i += 1;
    }
    Err(format!("unmatched '{open}'"))
}

fn split_alternatives<T: PatternToken>(
    chars: &[T],
    start: usize,
    end: usize,
) -> Result<Vec<(usize, usize)>, String> {
    let mut ranges = Vec::new();
    let mut seg_start = start;
    let mut i = start;
    while i < end {
        match chars[i].ch() {
            '|' => {
                ranges.push((seg_start, i));
                seg_start = i + 1;
                i += 1;
            }
            '[' | '<' => {
                i = find_closing(chars, i)? + 1;
            }
            _ => i += 1,
        }
    }
    ranges.push((seg_start, end));
    Ok(ranges)
}

fn parse_repeat<T: PatternToken>(chars: &[T], from: usize, end: usize) -> (usize, usize) {
    if from < end && chars[from].ch() == '*' {
        let mut i = from + 1;
        while i < end && chars[i].ch().is_ascii_digit() {
            i += 1;
        }
        if i > from + 1 {
            let n: usize = chars[from + 1..i]
                .iter()
                .map(PatternToken::ch)
                .collect::<String>()
                .parse()
                .unwrap_or(1);
            return (n.max(1), i - from);
        }
    }
    (1, 0)
}

/// Split a pattern string into top-level slot substrings.
/// Each slot is: a char (+ optional `*N`), or a bracket group (+ optional `*N`).
fn top_level_slots(input: &str) -> Result<Vec<String>, String> {
    let chars: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
    let mut slots = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            'x' | '.' | '-' => {
                let (_, rep_consumed) = parse_repeat(&chars, i + 1, chars.len());
                let end = i + 1 + rep_consumed;
                slots.push(chars[i..end].iter().collect());
                i = end;
            }
            '[' | '<' => {
                let close = find_closing(&chars, i)?;
                let (_, rep_consumed) = parse_repeat(&chars, close + 1, chars.len());
                let end = close + 1 + rep_consumed;
                slots.push(chars[i..end].iter().collect());
                i = end;
            }
            c => return Err(format!("unknown pattern char '{c}'")),
        }
    }
    Ok(slots)
}

pub(crate) fn rotate_pattern(input: &str, n: i64) -> Result<String, String> {
    let mut slots = top_level_slots(input)?;
    if slots.is_empty() {
        return Ok(String::new());
    }
    let len = slots.len() as i64;
    let shift = ((n % len) + len) % len;
    slots.rotate_right(shift as usize);
    Ok(slots.join(""))
}

pub(crate) fn reverse_pattern(input: &str) -> Result<String, String> {
    let mut slots = top_level_slots(input)?;
    slots.reverse();
    Ok(slots.join(""))
}

pub(crate) fn invert_pattern(input: &str) -> Result<String, String> {
    let chars: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
    Ok(chars
        .iter()
        .map(|c| match c {
            'x' => '.',
            '.' => 'x',
            '-' => 'x',
            other => *other,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn assert_hit(hit: &PatHit, pos: f64, gate: f64) {
        assert!(
            approx(hit.position, pos),
            "position: expected {pos}, got {}",
            hit.position
        );
        assert!(
            approx(hit.gate, gate),
            "gate: expected {gate}, got {}",
            hit.gate
        );
    }

    fn assert_hit_alt(hit: &PatHit, pos: f64, gate: f64, index: u16, count: u16) {
        assert_hit(hit, pos, gate);
        let a = hit.alt.as_ref().expect("expected alt");
        assert_eq!(a.index, index, "alt index");
        assert_eq!(a.count, count, "alt count");
    }

    // --- basic (unchanged) ---

    #[test]
    fn simple_four() {
        let hits = parse_pattern("x.x.").unwrap();
        assert_eq!(hits.len(), 2);
        assert_hit(&hits[0], 0.0, 0.25);
        assert_hit(&hits[1], 0.5, 0.25);
    }

    #[test]
    fn all_hits() {
        let hits = parse_pattern("xxxx").unwrap();
        assert_eq!(hits.len(), 4);
        for (i, h) in hits.iter().enumerate() {
            assert_hit(h, i as f64 / 4.0, 0.25);
        }
    }

    #[test]
    fn elongation() {
        let hits = parse_pattern("x--x").unwrap();
        assert_eq!(hits.len(), 2);
        assert_hit(&hits[0], 0.0, 0.75);
        assert_hit(&hits[1], 0.75, 0.25);
    }

    #[test]
    fn subdivision() {
        let hits = parse_pattern("[xx.]").unwrap();
        assert_eq!(hits.len(), 2);
        assert_hit(&hits[0], 0.0, 1.0 / 3.0);
        assert_hit(&hits[1], 1.0 / 3.0, 1.0 / 3.0);
    }

    #[test]
    fn nested() {
        let hits = parse_pattern("x..[xx.]").unwrap();
        assert_eq!(hits.len(), 3);
        assert_hit(&hits[0], 0.0, 0.25);
        let sw = 0.25 / 3.0;
        assert_hit(&hits[1], 0.75, sw);
        assert_hit(&hits[2], 0.75 + sw, sw);
    }

    #[test]
    fn elongation_complex() {
        let hits = parse_pattern("x--x..[xx.]").unwrap();
        assert_eq!(hits.len(), 4);
        assert_hit(&hits[0], 0.0, 3.0 / 7.0);
        assert_hit(&hits[1], 3.0 / 7.0, 1.0 / 7.0);
        let sw = (1.0 / 7.0) / 3.0;
        assert_hit(&hits[2], 6.0 / 7.0, sw);
        assert_hit(&hits[3], 6.0 / 7.0 + sw, sw);
    }

    #[test]
    fn nested_subdivision() {
        let hits = parse_pattern("x[x[xx]]").unwrap();
        assert_eq!(hits.len(), 4);
        assert_hit(&hits[0], 0.0, 0.5);
        assert_hit(&hits[1], 0.5, 0.25);
        assert_hit(&hits[2], 0.75, 0.125);
        assert_hit(&hits[3], 0.875, 0.125);
    }

    #[test]
    fn elongation_in_subdivision() {
        let hits = parse_pattern("[x-.]").unwrap();
        assert_eq!(hits.len(), 1);
        assert_hit(&hits[0], 0.0, 2.0 / 3.0);
    }

    #[test]
    fn all_rests() {
        let hits = parse_pattern("....").unwrap();
        assert_eq!(hits.len(), 0);
    }

    // --- repeat ---

    #[test]
    fn repeat_subdivision() {
        // [x.]*3 = 3 slots, each subdivided into (x .)
        let hits = parse_pattern("[x.]*3").unwrap();
        assert_eq!(hits.len(), 3);
        assert_hit(&hits[0], 0.0, 1.0 / 6.0);
        assert_hit(&hits[1], 1.0 / 3.0, 1.0 / 6.0);
        assert_hit(&hits[2], 2.0 / 3.0, 1.0 / 6.0);
    }

    #[test]
    fn repeat_with_other_slots() {
        // x[x.]*2x = 4 top-level slots
        let hits = parse_pattern("x[x.]*2x").unwrap();
        assert_eq!(hits.len(), 4);
        assert_hit(&hits[0], 0.0, 0.25);
        assert_hit(&hits[1], 0.25, 0.125);
        assert_hit(&hits[2], 0.5, 0.125);
        assert_hit(&hits[3], 0.75, 0.25);
    }

    // --- alternation ---

    #[test]
    fn simple_alternation() {
        // <x|.> = 1 slot, alt 0 has a hit, alt 1 is rest
        let hits = parse_pattern("<x|.>").unwrap();
        assert_eq!(hits.len(), 1);
        assert_hit_alt(&hits[0], 0.0, 1.0, 0, 2);
    }

    #[test]
    fn alternation_with_context() {
        // x<x.|.x>x = 3 top-level slots
        let hits = parse_pattern("x<x.|.x>x").unwrap();
        // slot 0: x (no alt)
        // slot 1 alt 0: x. → hit at 1/3, gate 1/6
        // slot 1 alt 1: .x → hit at 1/3 + 1/6, gate 1/6
        // slot 2: x (no alt)
        assert_eq!(hits.len(), 4);
        assert_hit(&hits[0], 0.0, 1.0 / 3.0);
        assert!(hits[0].alt.is_none());
        assert_hit_alt(&hits[1], 1.0 / 3.0, 1.0 / 6.0, 0, 2);
        assert_hit_alt(&hits[2], 1.0 / 3.0 + 1.0 / 6.0, 1.0 / 6.0, 1, 2);
        assert_hit(&hits[3], 2.0 / 3.0, 1.0 / 3.0);
        assert!(hits[3].alt.is_none());
    }

    #[test]
    fn three_way_alternation() {
        let hits = parse_pattern("<x|.|x->").unwrap();
        // alt 0: x (1 slot) → pos 0, gate 1
        // alt 1: . (1 slot) → no hit
        // alt 2: x- (2 slots) → pos 0, gate 1
        assert_eq!(hits.len(), 2);
        assert_hit_alt(&hits[0], 0.0, 1.0, 0, 3);
        assert_hit_alt(&hits[1], 0.0, 1.0, 2, 3);
    }

    #[test]
    fn alternation_repeated() {
        // <x|.>*2 = 2 top-level slots, both with alternation
        let hits = parse_pattern("<x|.>*2").unwrap();
        assert_eq!(hits.len(), 2);
        assert_hit_alt(&hits[0], 0.0, 0.5, 0, 2);
        assert_hit_alt(&hits[1], 0.5, 0.5, 0, 2);
    }

    #[test]
    fn alternation_with_subdivision() {
        // <[xx]|x> = 1 slot
        // alt 0: [xx] → 2 sub-hits
        // alt 1: x → 1 hit
        let hits = parse_pattern("<[xx]|x>").unwrap();
        assert_eq!(hits.len(), 3);
        assert_hit_alt(&hits[0], 0.0, 0.5, 0, 2);
        assert_hit_alt(&hits[1], 0.5, 0.5, 0, 2);
        assert_hit_alt(&hits[2], 0.0, 1.0, 1, 2);
    }

    // --- char repeat ---

    #[test]
    fn repeat_hit() {
        // x*4 = xxxx
        let hits = parse_pattern("x*4").unwrap();
        assert_eq!(hits.len(), 4);
        for (i, h) in hits.iter().enumerate() {
            assert_hit(h, i as f64 / 4.0, 0.25);
        }
    }

    #[test]
    fn repeat_rest() {
        // .x*3. = 5 slots, 3 hits in the middle
        let hits = parse_pattern(".x*3.").unwrap();
        assert_eq!(hits.len(), 3);
        assert_hit(&hits[0], 0.2, 0.2);
        assert_hit(&hits[1], 0.4, 0.2);
        assert_hit(&hits[2], 0.6, 0.2);
    }

    #[test]
    fn repeat_rest_only() {
        // .*4 = ....
        let hits = parse_pattern(".*4").unwrap();
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn repeat_256_uniform_gate() {
        let hits = parse_pattern("x*256").unwrap();
        assert_eq!(hits.len(), 256);
        let gate = 1.0 / 256.0;
        for (i, h) in hits.iter().enumerate() {
            assert_hit(h, i as f64 / 256.0, gate);
        }
    }

    // --- whitespace ---

    #[test]
    fn whitespace_ignored() {
        let hits = parse_pattern("x . x .").unwrap();
        assert_eq!(hits.len(), 2);
        assert_hit(&hits[0], 0.0, 0.25);
        assert_hit(&hits[1], 0.5, 0.25);
    }

    #[test]
    fn annotated_hit_spans_preserve_source_ranges() {
        let hits = parse_pattern_annotated("x--x..[xx.]").unwrap();
        assert_eq!(hits.len(), 4);
        assert_eq!((hits[0].start, hits[0].end), (0, 3));
        assert_eq!((hits[1].start, hits[1].end), (3, 4));
        assert_eq!((hits[2].start, hits[2].end), (7, 8));
        assert_eq!((hits[3].start, hits[3].end), (8, 9));
    }

    #[test]
    fn annotated_hit_spans_keep_whitespace_offsets() {
        let hits = parse_pattern_annotated("x . x .").unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!((hits[0].start, hits[0].end), (0, 1));
        assert_eq!((hits[1].start, hits[1].end), (4, 5));
    }

    // --- errors ---

    #[test]
    fn error_leading_dash() {
        assert!(parse_pattern("-x").is_err());
    }

    #[test]
    fn error_empty_brackets() {
        assert!(parse_pattern("[]").is_err());
    }

    #[test]
    fn error_unmatched_bracket() {
        assert!(parse_pattern("[xx").is_err());
    }

    #[test]
    fn error_unmatched_angle() {
        assert!(parse_pattern("<x|y").is_err());
    }

    #[test]
    fn error_empty() {
        assert!(parse_pattern("").is_err());
    }

    #[test]
    fn error_pipe_outside() {
        assert!(parse_pattern("x|x").is_err());
    }

    // --- transforms ---

    #[test]
    fn rotate_right() {
        // [x, ., ., x] → rotate_right(1) → [x, x, ., .]
        assert_eq!(rotate_pattern("x..x", 1).unwrap(), "xx..");
    }

    #[test]
    fn rotate_left() {
        // x . . x → rotate left by 1 → . . x x
        assert_eq!(rotate_pattern("x..x", -1).unwrap(), "..xx");
    }

    #[test]
    fn rotate_preserves_brackets() {
        // x [xx] . → rotate right by 1 → . x [xx]
        assert_eq!(rotate_pattern("x[xx].", 1).unwrap(), ".x[xx]");
    }

    #[test]
    fn rotate_with_repeat() {
        // x . *2 → slots: ["x", ".*2"] → rotate right 1 → .*2 x
        assert_eq!(rotate_pattern("x.*2", 1).unwrap(), ".*2x");
    }

    #[test]
    fn rotate_zero() {
        assert_eq!(rotate_pattern("x.x.", 0).unwrap(), "x.x.");
    }

    #[test]
    fn rotate_full_cycle() {
        assert_eq!(rotate_pattern("x.x.", 4).unwrap(), "x.x.");
    }

    #[test]
    fn reverse_simple() {
        assert_eq!(reverse_pattern("x...").unwrap(), "...x");
    }

    #[test]
    fn reverse_with_brackets() {
        assert_eq!(reverse_pattern("x.[xx.]").unwrap(), "[xx.].x");
    }

    #[test]
    fn reverse_preserves_alternation() {
        assert_eq!(reverse_pattern("x<a|b>.").unwrap(), ".<a|b>x");
    }

    #[test]
    fn invert_simple() {
        assert_eq!(invert_pattern("x.x.").unwrap(), ".x.x");
    }

    #[test]
    fn invert_elongation() {
        // x-- becomes .xx (elongation of a now-rest becomes hits)
        assert_eq!(invert_pattern("x--.").unwrap(), ".xxx");
    }

    #[test]
    fn invert_preserves_brackets() {
        assert_eq!(invert_pattern("[x.x]").unwrap(), "[.x.]");
    }
}
