fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/Sova.ico")
            .set("ProductName", "Sova")
            .set("FileDescription", "Sova - Live coding sequencer")
            .set("LegalCopyright", "Copyright (c) 2025 Raphaël Forment");
        res.compile().expect("Failed to compile Windows resources");
    }
}
