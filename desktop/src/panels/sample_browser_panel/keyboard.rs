use eframe::egui;

use crate::InputOwner;
use crate::sample_browser::SampleBrowserState;

pub(super) fn handle_keyboard(
    ui: &mut egui::Ui,
    state: &mut SampleBrowserState,
    search_focused: bool,
    input_owner: InputOwner,
) -> (bool, bool) {
    let search_id = ui.id().with("sample_search");
    let other_focus = ui.memory(|m| m.focused().is_some_and(|id| id != search_id));
    if other_focus {
        return (false, false);
    }

    let mut activate = false;
    let mut focus_search = false;

    ui.input(|i| {
        if search_focused {
            if i.key_pressed(egui::Key::Escape) {
                state.clear_search();
                ui.memory_mut(|m| m.surrender_focus(search_id));
            }
            return;
        }

        if input_owner != InputOwner::SampleBrowser {
            return;
        }

        let ctrl = i.modifiers.command;

        if i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::K) {
            let n = if ctrl { 10 } else { 1 };
            state.move_up(n);
        }
        if i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::J) {
            let n = if ctrl { 10 } else { 1 };
            state.move_down(n, 30);
        }
        if i.key_pressed(egui::Key::PageUp) {
            state.move_up(20);
        }
        if i.key_pressed(egui::Key::PageDown) {
            state.move_down(20, 30);
        }
        if i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::ArrowRight) {
            activate = true;
        }
        if i.key_pressed(egui::Key::ArrowLeft) {
            state.collapse_at_cursor();
        }
        if i.key_pressed(egui::Key::Slash) {
            focus_search = true;
        }
        if i.key_pressed(egui::Key::Escape) && !state.search_query.is_empty() {
            state.clear_filter();
        }
    });

    (activate, focus_search)
}
