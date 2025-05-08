use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::lang::variable::Variable;

pub const DEBUG_TIME_STATEMENTS: bool = false;
pub const DEBUG_INSTRUCTIONS: bool = true;
pub const DEFAULT_VELOCITY: i64 = 90;
pub const DEFAULT_CHAN: i64 = 1;
pub const DEFAULT_DEVICE: i64 = 1;
pub const DEFAULT_DURATION: i64 = 1;

lazy_static! {
    pub static ref LOCAL_TARGET_VAR: Variable = Variable::Instance("_local_target".to_owned());
    pub static ref LOCAL_PICK_VAR: Variable = Variable::Instance("_local_pick".to_owned());
    pub static ref LOCAL_ALT_VAR: Variable = Variable::Instance("_local_alt".to_owned());
}

pub fn generate_note_map() -> HashMap<String, i64> {
    let mut m = HashMap::new();
    for midi_val_i64 in 0..=127 {
        let midi_val = midi_val_i64 as i64;
        let octave_num = midi_val / 12 - 2;
        let note_idx = midi_val % 12;

        match note_idx {
            0 => { // C
                m.insert(format!("c{}", octave_num), midi_val);
                if octave_num > -2 { // Excludes C-2 for b#-1 logic
                    let prev_octave_for_sharp = octave_num - 1;
                    m.insert(format!("b#{}" , prev_octave_for_sharp), midi_val);
                    m.insert(format!("b{}#" , prev_octave_for_sharp), midi_val);
                }
            }
            1 => { // C# / Db
                m.insert(format!("c#{}" , octave_num), midi_val);
                m.insert(format!("c{}#" , octave_num), midi_val);
                m.insert(format!("db{}" , octave_num), midi_val);
                m.insert(format!("d{}b" , octave_num), midi_val);
            }
            2 => { // D
                m.insert(format!("d{}", octave_num), midi_val);
            }
            3 => { // D# / Eb
                m.insert(format!("d#{}" , octave_num), midi_val);
                m.insert(format!("d{}#" , octave_num), midi_val);
                m.insert(format!("eb{}" , octave_num), midi_val);
                m.insert(format!("e{}b" , octave_num), midi_val);
            }
            4 => { // E / Fb
                m.insert(format!("e{}", octave_num), midi_val);
                m.insert(format!("fb{}", octave_num), midi_val);
                m.insert(format!("f{}b", octave_num), midi_val);
            }
            5 => { // F / E#
                m.insert(format!("f{}", octave_num), midi_val);
                m.insert(format!("e#{}" , octave_num), midi_val);
                m.insert(format!("e{}#" , octave_num), midi_val);
            }
            6 => { // F# / Gb
                m.insert(format!("f#{}" , octave_num), midi_val);
                m.insert(format!("f{}#" , octave_num), midi_val);
                m.insert(format!("gb{}" , octave_num), midi_val);
                m.insert(format!("g{}b" , octave_num), midi_val);
            }
            7 => { // G
                m.insert(format!("g{}", octave_num), midi_val);
            }
            8 => { // G# / Ab
                m.insert(format!("g#{}" , octave_num), midi_val);
                m.insert(format!("g{}#" , octave_num), midi_val);
                m.insert(format!("ab{}" , octave_num), midi_val);
                m.insert(format!("a{}b" , octave_num), midi_val);
            }
            9 => { // A
                m.insert(format!("a{}", octave_num), midi_val);
            }
            10 => { // A# / Bb
                m.insert(format!("a#{}" , octave_num), midi_val);
                m.insert(format!("a{}#" , octave_num), midi_val);
                m.insert(format!("bb{}" , octave_num), midi_val);
                m.insert(format!("b{}b" , octave_num), midi_val);
            }
            11 => { // B / Cb
                m.insert(format!("b{}", octave_num), midi_val);
                m.insert(format!("cb{}", octave_num + 1), midi_val);
                m.insert(format!("c{}b", octave_num + 1), midi_val);
            }
            _ => unreachable!("Invalid note_idx: must be 0-11"),
        }

        if octave_num == 3 {
            match note_idx {
                0 => { m.insert("c".to_string(), midi_val); }
                1 => { 
                    m.insert("c#".to_string(), midi_val); m.insert("db".to_string(), midi_val);
                }
                2 => { m.insert("d".to_string(), midi_val); }
                3 => {
                    m.insert("d#".to_string(), midi_val); m.insert("eb".to_string(), midi_val);
                }
                4 => {
                    m.insert("e".to_string(), midi_val); m.insert("fb".to_string(), midi_val);
                }
                5 => {
                    m.insert("f".to_string(), midi_val); m.insert("e#".to_string(), midi_val);
                }
                6 => {
                    m.insert("f#".to_string(), midi_val); m.insert("gb".to_string(), midi_val);
                }
                7 => { m.insert("g".to_string(), midi_val); }
                8 => {
                    m.insert("g#".to_string(), midi_val); m.insert("ab".to_string(), midi_val);
                }
                9 => { m.insert("a".to_string(), midi_val); }
                10 => {
                    m.insert("a#".to_string(), midi_val); m.insert("bb".to_string(), midi_val);
                }
                11 => {
                    m.insert("b".to_string(), midi_val); 
                    // cb alias handled below
                }
                _ => {}
            }
        }
    }

    m.insert("cb".to_string(), 59);
    m
}

lazy_static! {
    pub static ref NOTE_MAP: HashMap<String, i64> = generate_note_map();
}

#[cfg(test)]
mod tests {
    use super::NOTE_MAP;

    #[test]
    fn test_specific_notes() {
        let generated_map = NOTE_MAP.clone();

        // Test fundamental and boundary notes
        assert_eq!(generated_map.get("c-2"), Some(&0), "Test failed for c-2");
        assert_eq!(generated_map.get("g8"), Some(&127), "Test failed for g8");
        
        // Tests: sharps, flats, octaves
        assert_eq!(generated_map.get("c#3"), Some(&61), "Test failed for c#3");
        assert_eq!(generated_map.get("db3"), Some(&61), "Test failed for db3");
        assert_eq!(generated_map.get("c3#"), Some(&61), "Test failed for c3#");
        assert_eq!(generated_map.get("d3b"), Some(&61), "Test failed for d3b");
        assert_eq!(generated_map.get("f#-1"), Some(&18), "Test failed for f#-1");
        assert_eq!(generated_map.get("gb-1"), Some(&18), "Test failed for gb-1");
        assert_eq!(generated_map.get("e4"), Some(&76), "Test failed for e4");
        assert_eq!(generated_map.get("b0"), Some(&35), "Test failed for b0");

        // Alias for octave 3
        assert_eq!(generated_map.get("c"), Some(&60), "Test failed for alias c");
        assert_eq!(generated_map.get("c#"), Some(&61), "Test failed for alias c#");
        assert_eq!(generated_map.get("db"), Some(&61), "Test failed for alias db");
        assert_eq!(generated_map.get("b"), Some(&71), "Test failed for alias b");  
        
        // Special enharmonic cases
        assert_eq!(generated_map.get("b#-1"), Some(&24), "Test failed for b#-1 (should be c0)");
        assert_eq!(generated_map.get("cb1"), Some(&35), "Test failed for cb1 (should be b0)");
        assert_eq!(generated_map.get("e#2"), Some(&53), "Test failed for e#2 (should be f2)"); 
        assert_eq!(generated_map.get("fb2"), Some(&52), "Test failed for fb2 (should be e2)");

        // Alias cb
        assert_eq!(generated_map.get("cb"), Some(&59), "Test failed for alias cb (B2/Cb3)");

        // Check a non-existent note
        assert_eq!(generated_map.get("z99"), None, "Test failed for non-existent note z99");
        assert_eq!(generated_map.get("frenchtoast"), None, "Test failed for non-existent note frenchtoast");

        // Test a note with all its English forms for C#3 (MIDI 61)
        let c_sharp_3_midi = 61;
        assert_eq!(generated_map.get("c#3").unwrap_or(&-1), &c_sharp_3_midi, "Missing c#3");
        assert_eq!(generated_map.get("c3#").unwrap_or(&-1), &c_sharp_3_midi, "Missing c3#");
        assert_eq!(generated_map.get("db3").unwrap_or(&-1), &c_sharp_3_midi, "Missing db3");
        assert_eq!(generated_map.get("d3b").unwrap_or(&-1), &c_sharp_3_midi, "Missing d3b");
    }
}
