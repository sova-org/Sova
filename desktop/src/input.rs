use eframe::egui;

use crate::{
    app_types::InputOwner,
    SovaApp,
};

pub(crate) fn consume_menu_fkey(ctx: &egui::Context) -> Option<usize> {
    ctx.input_mut(|i| {
        if !i.modifiers.is_none() {
            return None;
        }

        [
            egui::Key::F1,
            egui::Key::F2,
            egui::Key::F3,
            egui::Key::F4,
            egui::Key::F5,
        ]
        .into_iter()
        .enumerate()
        .find_map(|(idx, key)| i.consume_key(egui::Modifiers::NONE, key).then_some(idx))
    })
}

impl SovaApp {
    /// Determines which panel owns bare (unmodified) keyboard input this frame.
    /// Called once at the start of each update, before any panel renders.
    pub(crate) fn resolve_input_owner(&mut self, ctx: &egui::Context) {
        if self.menu_bar.is_active() {
            self.input_owner = InputOwner::MenuBar;
            // InputOwner contract: when MenuBar owns input, no other widget may
            // hold egui focus. Without this, a focused TextEdit receives the same
            // Event::Text events the menu bar uses for mnemonic navigation and
            // writes them into the buffer.
            if let Some(id) = ctx.memory(|m| m.focused()) {
                ctx.memory_mut(|m| m.surrender_focus(id));
            }
            return;
        }
        if self.panels.command_palette.is_open() {
            self.input_owner = InputOwner::Palette;
            return;
        }

        // Click-to-focus: pointer press this frame determines ownership.
        let click_pos = ctx.input(|i| {
            i.pointer
                .any_pressed()
                .then(|| i.pointer.interact_pos())
                .flatten()
        });
        if let Some(pos) = click_pos {
            let sample_visible = self.panels.tools.settings.show_sample_browser
                && self.panels.tools.settings.open
                && !self.panels.sample_browser.detached;
            if sample_visible && self.sample_browser_rect.is_some_and(|r| r.contains(pos)) {
                self.input_owner = InputOwner::SampleBrowser;
            } else if self.bridge.is_connected() {
                self.input_owner = InputOwner::Scene;
            }
            return;
        }

        // Sticky: preserve previous owner, but revert if the owning panel closed.
        match self.input_owner {
            InputOwner::MenuBar | InputOwner::Palette => {
                self.input_owner = InputOwner::Scene;
            }
            InputOwner::SampleBrowser => {
                let still_visible = self.panels.tools.settings.show_sample_browser
                    && self.panels.tools.settings.open
                    && !self.panels.sample_browser.detached;
                if !still_visible {
                    self.input_owner = InputOwner::Scene;
                }
            }
            InputOwner::Scene => {}
        }
    }

    pub(crate) fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        self.resolve_input_owner(ctx);

        // Menu bar active: it gets all input priority.
        if self.input_owner == InputOwner::MenuBar {
            if let Some(idx) = consume_menu_fkey(ctx) {
                self.menu_bar.activate(idx);
                return;
            }
            let menus = self.build_menus(ctx);
            self.menu_bar.handle_input(ctx, &menus);
            return;
        }
        // Cmd+K opens the command palette and bypasses focus + connection checks.
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::K)) {
            if !self.panels.command_palette.is_open() {
                self.panels.command_palette.open();
            }
            return;
        }
        if self.input_owner == InputOwner::Palette {
            return;
        }
        // F1-F5 activate the menu bar (fires even when an editor has focus).
        if let Some(idx) = consume_menu_fkey(ctx) {
            self.menu_bar.activate(idx);
            return;
        }
        if ctx.memory(|m| m.focused().is_some()) {
            return;
        }
        // Scene undo/redo has no CommandId equivalent and is connection-gated.
        if self.bridge.is_connected() {
            let (undo, redo) = ctx.input(|i| {
                (
                    i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z),
                    i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z),
                )
            });
            if redo {
                self.bridge.redo();
                return;
            }
            if undo {
                self.bridge.undo();
                return;
            }
        }
        // Walk the palette's shortcut table — single source of truth for the
        // key binding map. dispatch() owns all per-command guards.
        let pressed = self
            .panels
            .command_palette
            .shortcut_table()
            .find_map(|(id, sc)| sc.pressed(ctx).then_some(id));
        if let Some(cmd) = pressed {
            self.dispatch(cmd);
        }
    }
}
