use std::fmt::Write;
use std::path::Path;

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let mut out = c.to_uppercase().to_string();
                    out.extend(chars);
                    out
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn generate_demos_for_dir(dir: &Path, const_name: &str, out: &mut String) {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "sova")
        })
        .map(|e| e.path())
        .collect();

    files.sort();

    writeln!(out, "pub const {const_name}: &[(&str, &[u8])] = &[").unwrap();

    let mut prev_group: Option<char> = None;
    for path in &files {
        let stem = path.file_stem().unwrap().to_str().unwrap();

        // Extract numeric prefix and name part
        let (num, name_part) = match stem.find('_') {
            Some(pos) => (&stem[..pos], &stem[pos + 1..]),
            None => (stem, ""),
        };

        // Group separator when first digit changes
        let group = num.chars().next().unwrap_or('0');
        if let Some(prev) = prev_group {
            if group != prev {
                writeln!(out, r#"    ("\x00", &[]),"#).unwrap();
            }
        }
        prev_group = Some(group);

        let display = if name_part.is_empty() {
            num.to_string()
        } else {
            let human = title_case(&name_part.replace('_', " "));
            format!("{num} \u{2014} {human}")
        };

        let abs = path.canonicalize().unwrap();
        let abs_str = abs.display().to_string().replace('\\', "/");
        writeln!(out, r#"    ("{display}", include_bytes!("{abs_str}")),"#).unwrap();
    }

    writeln!(out, "];").unwrap();
}

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/Sova.ico")
            .set("ProductName", "Sova")
            .set("FileDescription", "Sova - Live coding sequencer")
            .set("LegalCopyright", "Copyright (c) 2025 Raphaël Forment");
        res.compile().expect("Failed to compile Windows resources");
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("demos_generated.rs");

    let mut code = String::new();

    let demos_base = manifest_dir.join("assets/demos");

    let cagire_dir = demos_base.join("cagire");
    if cagire_dir.is_dir() {
        generate_demos_for_dir(&cagire_dir, "DEMOS_CAGIRE", &mut code);
        println!("cargo:rerun-if-changed=assets/demos/cagire");
    } else {
        writeln!(code, "pub const DEMOS_CAGIRE: &[(&str, &[u8])] = &[];").unwrap();
    }

    let boinx_dir = demos_base.join("boinx");
    if boinx_dir.is_dir() {
        generate_demos_for_dir(&boinx_dir, "DEMOS_BOINX", &mut code);
        println!("cargo:rerun-if-changed=assets/demos/boinx");
    } else {
        writeln!(code, "pub const DEMOS_BOINX: &[(&str, &[u8])] = &[];").unwrap();
    }

    std::fs::write(&out_path, code).unwrap();
}
