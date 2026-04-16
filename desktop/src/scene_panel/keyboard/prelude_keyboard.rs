use eframe::egui;
use sova_core::schedule::SchedulerMessage;

use crate::client_bridge::ClientBridge;

impl crate::scene_panel::ScenePanel {
    pub(crate) fn handle_prelude_keyboard(
        &mut self,
        ui: &mut egui::Ui,
        idx: usize,
        scene: &sova_core::scene::Scene,
        bridge: &ClientBridge,
    ) {
        let prelude_len = self.prelude_states.len();
        if prelude_len == 0 {
            self.navigate_to_frame((0, 0), bridge);
            return;
        }
        let idx = idx.min(prelude_len - 1);

        let (left, right, down, key_enter, key_escape, key_delete, cmd_d) = ui.input(|i| {
            let no_mod =
                !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && !i.modifiers.shift;
            (
                i.key_pressed(egui::Key::ArrowLeft) || (no_mod && i.key_pressed(egui::Key::H)),
                i.key_pressed(egui::Key::ArrowRight) || (no_mod && i.key_pressed(egui::Key::L)),
                i.key_pressed(egui::Key::ArrowDown) || (no_mod && i.key_pressed(egui::Key::J)),
                i.key_pressed(egui::Key::Enter) && !i.modifiers.command && !i.modifiers.ctrl,
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::D),
            )
        });

        if key_escape {
            self.enter_prelude_edit(idx);
            return;
        }

        if key_enter && idx < prelude_len {
            self.enter_prelude_edit(idx);
            return;
        }

        if key_delete && prelude_len > 0 {
            let mut scripts: Vec<sova_core::scene::script::Script> = bridge
                .scene()
                .map(|s| s.prelude.clone())
                .unwrap_or_default();
            if idx < scripts.len() {
                scripts.remove(idx);
                bridge.send(sova_server::ClientMessage::SchedulerControl(
                    SchedulerMessage::SetScenePrelude(scripts),
                ));
                if self.prelude_states.len() <= 1 {
                    self.deselect_all();
                } else if idx >= self.prelude_states.len().saturating_sub(1) {
                    self.navigate_to_prelude(idx.saturating_sub(1));
                }
            }
            return;
        }

        if cmd_d && idx < prelude_len {
            let mut scripts: Vec<sova_core::scene::script::Script> = bridge
                .scene()
                .map(|s| s.prelude.clone())
                .unwrap_or_default();
            if idx < scripts.len() {
                let dup = scripts[idx].clone();
                scripts.insert(idx + 1, dup);
                bridge.send(sova_server::ClientMessage::SchedulerControl(
                    SchedulerMessage::SetScenePrelude(scripts),
                ));
                self.navigate_to_prelude(idx + 1);
            }
            return;
        }

        if down && !scene.lines.is_empty() && !scene.lines[0].frames.is_empty() {
            self.navigate_to_frame((0, 0), bridge);
            return;
        }

        if left && idx > 0 {
            self.navigate_to_prelude(idx - 1);
        } else if right && idx + 1 < prelude_len {
            self.navigate_to_prelude(idx + 1);
        }
    }
}
