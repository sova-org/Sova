use eframe::egui;
use std::sync::Arc;

pub const FAMILY_NAME: &str = "phosphor";

pub fn install(fonts: &mut egui::FontDefinitions) {
    fonts.font_data.insert(
        FAMILY_NAME.into(),
        Arc::new(egui_phosphor::Variant::Fill.font_data()),
    );
    fonts.families.insert(
        egui::FontFamily::Name(FAMILY_NAME.into()),
        vec![FAMILY_NAME.into()],
    );
}

pub fn family() -> egui::FontFamily {
    egui::FontFamily::Name(FAMILY_NAME.into())
}

pub fn rich(icon: &str) -> egui::RichText {
    egui::RichText::new(icon).family(family())
}

pub fn small(icon: &str) -> egui::RichText {
    rich(icon).small()
}

fn text_format_with_color(
    ui: &egui::Ui,
    text_style: egui::TextStyle,
    color: egui::Color32,
) -> egui::TextFormat {
    egui::TextFormat {
        font_id: text_style.resolve(ui.style()),
        color,
        ..Default::default()
    }
}

fn icon_format_with_color(
    ui: &egui::Ui,
    text_style: egui::TextStyle,
    color: egui::Color32,
) -> egui::TextFormat {
    let font_id = text_style.resolve(ui.style());
    egui::TextFormat {
        font_id: egui::FontId::new(font_id.size, family()),
        color,
        ..Default::default()
    }
}

pub fn text(
    ui: &egui::Ui,
    text_style: egui::TextStyle,
    icon: &str,
    label: impl AsRef<str>,
) -> egui::WidgetText {
    text_colored(
        ui,
        text_style,
        icon,
        label,
        ui.visuals().widgets.inactive.text_color(),
    )
}

pub fn text_colored(
    ui: &egui::Ui,
    text_style: egui::TextStyle,
    icon: &str,
    label: impl AsRef<str>,
    color: egui::Color32,
) -> egui::WidgetText {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        icon,
        0.0,
        icon_format_with_color(ui, text_style.clone(), color),
    );
    job.append(
        " ",
        0.0,
        text_format_with_color(ui, text_style.clone(), color),
    );
    job.append(
        label.as_ref(),
        0.0,
        text_format_with_color(ui, text_style, color),
    );
    job.into()
}

pub fn button_text(ui: &egui::Ui, icon: &str, label: impl AsRef<str>) -> egui::WidgetText {
    text(ui, egui::TextStyle::Button, icon, label)
}

pub fn trailing_text(ui: &egui::Ui, label: impl AsRef<str>, icon: &str) -> egui::WidgetText {
    let mut job = egui::text::LayoutJob::default();
    let color = ui.visuals().widgets.inactive.text_color();
    job.append(
        label.as_ref(),
        0.0,
        text_format_with_color(ui, egui::TextStyle::Button, color),
    );
    job.append(
        " ",
        0.0,
        text_format_with_color(ui, egui::TextStyle::Button, color),
    );
    job.append(
        icon,
        0.0,
        icon_format_with_color(ui, egui::TextStyle::Button, color),
    );
    job.into()
}

// Transport
pub use egui_phosphor::fill::PAUSE;
pub use egui_phosphor::fill::PLAY;
pub use egui_phosphor::fill::STOP;

// Chevrons
pub use egui_phosphor::fill::CARET_DOWN as CHEVRON_DOWN;
pub use egui_phosphor::fill::CARET_LEFT as CHEVRON_LEFT;
pub use egui_phosphor::fill::CARET_RIGHT as CHEVRON_RIGHT;

// Window management
pub use egui_phosphor::fill::ARROW_SQUARE_OUT as POPOUT;
pub use egui_phosphor::fill::ROWS as DOCK;

// Scene grid indicators
pub use egui_phosphor::fill::ARROWS_CLOCKWISE as LOOPING;
pub use egui_phosphor::fill::PUSH_PIN as MANUAL;
pub use egui_phosphor::fill::TEXT_INDENT as TRAILING;

// Status
pub use egui_phosphor::fill::CIRCLE as CIRCLE_FILLED;
pub const CIRCLE_LARGE_FILLED: &str = egui_phosphor::fill::CIRCLE;
pub use egui_phosphor::fill::CHECK;
pub use egui_phosphor::fill::CIRCLE_DASHED as CIRCLE_LARGE_OUTLINE;
pub use egui_phosphor::fill::CPU;

// Dialog / links
pub use egui_phosphor::fill::X as CLOSE;
pub const LINK_EXTERNAL: &str = egui_phosphor::fill::ARROW_SQUARE_OUT;

// Settings
pub use egui_phosphor::fill::GEAR;
pub use egui_phosphor::fill::PALETTE;

// Editor
pub use egui_phosphor::fill::CODE;
pub use egui_phosphor::fill::PENCIL_SIMPLE as MODIFIED;

// Log
pub use egui_phosphor::fill::FILE_TEXT;
pub use egui_phosphor::fill::TRASH;

// Doc panel
pub use egui_phosphor::fill::BOOK_OPEN as BOOK;
pub use egui_phosphor::fill::SWAP;

// Volume
pub use egui_phosphor::fill::SPEAKER_HIGH as UNMUTE;
pub use egui_phosphor::fill::SPEAKER_SLASH as MUTE;

// Actions
pub use egui_phosphor::fill::PLUGS_CONNECTED;
pub use egui_phosphor::fill::PLUS as ADD;

// Transport actions
pub const HUSH: &str = egui_phosphor::fill::SPEAKER_SLASH;
pub use egui_phosphor::fill::WARNING as PANIC;

// Network
pub use egui_phosphor::fill::LINK as CONNECT;
pub use egui_phosphor::fill::LINK_BREAK as DISCONNECT;

// Focus mode
pub use egui_phosphor::fill::ARROWS_IN as UNFOCUS;
pub use egui_phosphor::fill::ARROWS_OUT as FOCUS;

// New icons for text-only buttons
pub use egui_phosphor::fill::ARROW_COUNTER_CLOCKWISE as REFRESH;
pub use egui_phosphor::fill::CHAT_CIRCLE_DOTS as CHAT;
pub use egui_phosphor::fill::KEYBOARD;
pub use egui_phosphor::fill::MUSIC_NOTE;
pub use egui_phosphor::fill::PAPER_PLANE_TILT as SEND;
pub use egui_phosphor::fill::WAVE_SINE;
