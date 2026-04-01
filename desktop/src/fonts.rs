use eframe::egui;
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

/// Load a system font by family name into egui for the given font family.
/// Returns `true` if the font was loaded successfully.
pub fn load_system_font(ctx: &egui::Context, family_name: &str, target: egui::FontFamily) -> bool {
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
            return false;
        }
    };

    let font = match handle.load() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to load font '{family_name}': {e}");
            return false;
        }
    };

    let Some(data) = font.copy_font_data() else {
        eprintln!("Failed to read font data for '{family_name}'");
        return false;
    };

    let bytes = Arc::try_unwrap(data).unwrap_or_else(|arc| (*arc).clone());

    ctx.add_font(egui::epaint::text::FontInsert::new(
        family_name,
        egui::FontData {
            font: std::borrow::Cow::Owned(bytes),
            index: 0,
            tweak: egui::FontTweak::default(),
        },
        vec![egui::epaint::text::InsertFontFamily {
            family: target,
            priority: egui::epaint::text::FontPriority::Highest,
        }],
    ));
    true
}

/// Apply custom font settings. Call after the nerd-font fallback is already loaded.
pub fn apply_custom_fonts(ctx: &egui::Context, ui_font: &str, editor_font: &str) {
    if !ui_font.is_empty() {
        load_system_font(ctx, ui_font, egui::FontFamily::Proportional);
    }
    if !editor_font.is_empty() {
        load_system_font(ctx, editor_font, egui::FontFamily::Monospace);
    }
}
