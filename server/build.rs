fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../desktop/assets/Sova.ico")
            .set("ProductName", "Sova Server")
            .set("FileDescription", "Sova Server - Live coding sequencer")
            .set("LegalCopyright", "Copyright (c) 2025 Raphaël Forment");
        res.compile().expect("Failed to compile Windows resources");
    }
}
