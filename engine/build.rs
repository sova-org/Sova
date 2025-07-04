use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const WAVETABLE_SIZE: usize = 600;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("wavetables.rs");
    let mut f = File::create(&dest_path).unwrap();

    let static_dir = Path::new("static/tables");

    if !static_dir.exists() {
        panic!(
            "Static tables directory not found: {}",
            static_dir.display()
        );
    }

    let mut entries = fs::read_dir(static_dir)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    writeln!(f, "// Auto-generated wavetables from build.rs").unwrap();
    writeln!(f, "use std::sync::OnceLock;").unwrap();
    writeln!(f, "").unwrap();
    writeln!(f, "pub const WAVETABLE_SIZE: usize = {};", WAVETABLE_SIZE).unwrap();
    writeln!(f, "pub const NUM_WAVETABLES: usize = {};", entries.len()).unwrap();
    writeln!(f, "").unwrap();

    for (index, entry) in entries.iter().enumerate() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("wav") {
            let _file_name = path.file_stem().unwrap().to_str().unwrap();

            match load_wavetable(&path) {
                Ok(samples) => {
                    writeln!(f, "const WAVETABLE_{}: [f32; WAVETABLE_SIZE] = [", index).unwrap();

                    for (i, sample) in samples.iter().enumerate() {
                        if i % 8 == 0 {
                            write!(f, "    ").unwrap();
                        }
                        write!(f, "{:>12.8}", sample).unwrap();
                        if i < samples.len() - 1 {
                            write!(f, ",").unwrap();
                        }
                        if i % 8 == 7 || i == samples.len() - 1 {
                            writeln!(f, "").unwrap();
                        }
                    }
                    writeln!(f, "];").unwrap();
                    writeln!(f, "").unwrap();
                }
                Err(e) => {
                    println!(
                        "cargo:warning=Failed to load wavetable {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }
    }

    writeln!(f, "pub static WAVETABLES: OnceLock<&'static [&'static [f32; WAVETABLE_SIZE]]> = OnceLock::new();").unwrap();
    writeln!(f, "").unwrap();
    writeln!(
        f,
        "pub fn get_wavetables() -> &'static [&'static [f32; WAVETABLE_SIZE]] {{"
    )
    .unwrap();
    writeln!(f, "    WAVETABLES.get_or_init(|| &[").unwrap();

    for i in 0..entries.len() {
        writeln!(f, "        &WAVETABLE_{},", i).unwrap();
    }

    writeln!(f, "    ])").unwrap();
    writeln!(f, "}}").unwrap();
    writeln!(f, "").unwrap();
    writeln!(
        f,
        "pub fn get_wavetable(index: usize) -> &'static [f32; WAVETABLE_SIZE] {{"
    )
    .unwrap();
    writeln!(f, "    let tables = get_wavetables();").unwrap();
    writeln!(f, "    tables[index % tables.len()]").unwrap();
    writeln!(f, "}}").unwrap();

    println!("cargo:rerun-if-changed=static/tables");
    println!("cargo:rerun-if-changed=build.rs");
}

fn load_wavetable(path: &Path) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();

    if spec.channels != 1 {
        return Err(format!("Expected mono audio, got {} channels", spec.channels).into());
    }

    let samples: Result<Vec<f32>, _> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect(),
        hound::SampleFormat::Int => reader
            .samples::<i32>()
            .map(|s| s.map(|sample| sample as f32 / (1i32 << (spec.bits_per_sample - 1)) as f32))
            .collect(),
    };

    let mut samples = samples?;

    if samples.len() != WAVETABLE_SIZE {
        if samples.len() > WAVETABLE_SIZE {
            samples.truncate(WAVETABLE_SIZE);
        } else {
            samples.resize(WAVETABLE_SIZE, 0.0);
        }
    }

    Ok(samples)
}
