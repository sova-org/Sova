use include_dir::{Dir, include_dir};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

static SAMPLES: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/samples");

/// Extracts embedded samples to the config directory and returns the path.
///
/// Samples are written to `~/.config/sova/samples/default/` (or platform equivalent).
/// Re-extracts when the embedded content changes (tracked via a `.fingerprint` file).
pub fn ensure_default_samples() -> PathBuf {
    let target = dirs::config_dir()
        .expect("no config directory available")
        .join("sova")
        .join("samples")
        .join("default");

    let embedded_fp = fingerprint(&SAMPLES).to_string();
    let fp_file = target.join(".fingerprint");
    let stored_fp = std::fs::read_to_string(&fp_file).unwrap_or_default();

    if embedded_fp == stored_fp {
        return target;
    }

    std::fs::create_dir_all(&target).expect("failed to create default samples directory");
    extract_dir(&SAMPLES, &target);
    std::fs::write(&fp_file, &embedded_fp).ok();
    eprintln!("[sova] extracted default samples to {}", target.display());
    target
}

/// Returns the default samples path if it exists on disk (already extracted).
pub fn default_samples_path() -> Option<PathBuf> {
    let target = dirs::config_dir()?
        .join("sova")
        .join("samples")
        .join("default");
    target.exists().then_some(target)
}

fn fingerprint(dir: &Dir) -> u64 {
    let mut hasher = DefaultHasher::new();
    hash_dir(dir, &mut hasher);
    hasher.finish()
}

fn hash_dir(dir: &Dir, hasher: &mut DefaultHasher) {
    for file in dir.files() {
        file.path().hash(hasher);
        file.contents().len().hash(hasher);
    }
    for sub in dir.dirs() {
        hash_dir(sub, hasher);
    }
}

fn extract_dir(dir: &Dir, target: &std::path::Path) {
    for file in dir.files() {
        let path = target.join(file.path());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&path, file.contents()).ok();
    }
    for sub in dir.dirs() {
        extract_dir(sub, target);
    }
}
