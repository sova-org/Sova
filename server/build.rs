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
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sova"))
        .map(|e| e.path())
        .collect();

    files.sort();

    writeln!(out, "pub const {const_name}: &[(&str, &[u8])] = &[").unwrap();

    let mut prev_group: Option<char> = None;
    for path in &files {
        let stem = path.file_stem().unwrap().to_str().unwrap();

        let (num, name_part) = match stem.find('_') {
            Some(pos) => (&stem[..pos], &stem[pos + 1..]),
            None => (stem, ""),
        };

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
        let abs_str = abs.display();
        writeln!(out, r#"    ("{display}", include_bytes!(r"{abs_str}")),"#).unwrap();
    }

    writeln!(out, "];").unwrap();
}

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("demos_generated.rs");

    let mut code = String::new();
    let demos_base = manifest_dir.join("assets/demos");

    for (dir_name, const_name) in [("cagire", "DEMOS_CAGIRE"), ("boinx", "DEMOS_BOINX")] {
        let dir = demos_base.join(dir_name);
        if dir.is_dir() {
            generate_demos_for_dir(&dir, const_name, &mut code);
            println!("cargo:rerun-if-changed=assets/demos/{dir_name}");
        } else {
            writeln!(code, "pub const {const_name}: &[(&str, &[u8])] = &[];").unwrap();
        }
    }

    println!("cargo:rerun-if-changed=assets/demos/general");

    std::fs::write(&out_path, code).unwrap();
}
