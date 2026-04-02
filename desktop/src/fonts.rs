use eframe::egui;
use font_kit::handle::Handle;
use font_kit::source::SystemSource;
use std::sync::Arc;

/// Cached list of system font family names, sorted alphabetically.
pub fn list_system_fonts() -> Vec<String> {
    let source = SystemSource::new();
    let mut families: Vec<String> = source
        .all_families()
        .unwrap_or_default()
        .into_iter()
        .filter(|name| !name.starts_with('.'))
        .collect();
    families.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    families.dedup();
    families
}

/// Load a system font by family name into memory.
fn load_system_font_data(family_name: &str) -> Option<(Vec<u8>, u32)> {
    let source = SystemSource::new();
    let handle = match source.select_best_match(
        &[font_kit::family_name::FamilyName::Title(
            family_name.to_string(),
        )],
        &font_kit::properties::Properties::new(),
    ) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Font '{family_name}' not found: {e}");
            return None;
        }
    };

    let font_index = match &handle {
        Handle::Path { font_index, .. } | Handle::Memory { font_index, .. } => *font_index,
    };

    let font = match handle.load() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to load font '{family_name}': {e}");
            return None;
        }
    };

    let Some(data) = font.copy_font_data() else {
        eprintln!("Failed to read font data for '{family_name}'");
        return None;
    };

    Some((
        Arc::try_unwrap(data).unwrap_or_else(|arc| (*arc).clone()),
        font_index,
    ))
}

fn insert_system_font(
    fonts: &mut egui::FontDefinitions,
    family_name: &str,
    key: &str,
    target: egui::FontFamily,
) {
    let Some((bytes, font_index)) = load_system_font_data(family_name) else {
        return;
    };

    fonts.font_data.insert(
        key.into(),
        Arc::new(egui::FontData {
            font: std::borrow::Cow::Owned(bytes),
            index: font_index,
            tweak: egui::FontTweak::default(),
        }),
    );

    if let Some(family) = fonts.families.get_mut(&target) {
        let insert_at = family
            .iter()
            .position(|name| name == "phosphor")
            .map(|idx| idx + 1)
            .unwrap_or(0);
        family.insert(insert_at, key.into());
    }
}

/// Rebuild the app font stack and register the icon font as its own family.
pub fn apply_fonts(ctx: &egui::Context, ui_font: &str, editor_font: &str) {
    let mut fonts = egui::FontDefinitions::default();
    crate::icons::install(&mut fonts);

    if !ui_font.is_empty() {
        insert_system_font(
            &mut fonts,
            ui_font,
            &format!("system-ui:{ui_font}"),
            egui::FontFamily::Proportional,
        );
    }
    if !editor_font.is_empty() {
        insert_system_font(
            &mut fonts,
            editor_font,
            &format!("system-editor:{editor_font}"),
            egui::FontFamily::Monospace,
        );
    }

    ctx.set_fonts(fonts);
}
