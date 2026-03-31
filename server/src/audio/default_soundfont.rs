use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

static SOUNDFONT: &[u8] = include_bytes!("../../assets/soundfont/generaluser_gs.sf2");
const FILENAME: &str = "generaluser_gs.sf2";

/// Extracts the embedded soundfont to the config directory and returns the
/// **parent directory** (suitable for `load_soundfont_from_paths`).
///
/// Target: `~/.config/sova/soundfont/generaluser_gs.sf2` (or platform equivalent).
/// Re-extracts only when the embedded content changes.
pub fn ensure_default_soundfont() -> PathBuf {
    let dir = soundfont_dir();
    let fp_file = dir.join(".fingerprint");
    let embedded_fp = fingerprint().to_string();
    let stored_fp = std::fs::read_to_string(&fp_file).unwrap_or_default();

    if embedded_fp == stored_fp {
        return dir;
    }

    std::fs::create_dir_all(&dir).expect("failed to create default soundfont directory");
    std::fs::write(dir.join(FILENAME), SOUNDFONT).expect("failed to write default soundfont");
    std::fs::write(&fp_file, &embedded_fp).ok();
    eprintln!("[sova] extracted default soundfont to {}", dir.display());
    dir
}

/// Returns true if any of the given paths contain a `.sf2` file,
/// indicating the user has provided their own soundfont.
pub fn user_has_soundfont(paths: &[PathBuf]) -> bool {
    for path in paths {
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if has_sf2_extension(&entry.path()) {
                        return true;
                    }
                }
            }
        } else if has_sf2_extension(path) {
            return true;
        }
    }
    false
}

fn soundfont_dir() -> PathBuf {
    dirs::config_dir()
        .expect("no config directory available")
        .join("sova")
        .join("soundfont")
}

fn fingerprint() -> u64 {
    let mut hasher = DefaultHasher::new();
    FILENAME.hash(&mut hasher);
    SOUNDFONT.len().hash(&mut hasher);
    hasher.finish()
}

fn has_sf2_extension(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("sf2"))
}
