mod definitions;
mod navigation;
mod rendering;

pub(crate) use definitions::{MenuContext, build_menus};
use navigation::{
    MnemonicMatch, clamp_levels, find_mnemonic_match, focus_item, items_at_depth, next_navigable,
    open_submenu,
};
use rendering::{
    MenuPanelClick, label_job, menu_popup_metrics, show_menu_panel, with_menu_style,
};

use eframe::egui;

use std::path::PathBuf;

use super::widgets::CommandId;

pub(crate) const MENU_MIN_WIDTH: f32 = 180.0;
pub(crate) const MENU_MAX_WIDTH: f32 = 360.0;
pub(crate) const MENU_LABEL_RIGHT_GAP: f32 = 24.0;
pub(crate) const MENU_MAX_HEIGHT_FRACTION: f32 = 0.75;

// ── Actions ───────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) enum MenuAction {
    Command(CommandId),
    LoadSceneAtEnd,
    LoadRecentScene(PathBuf),
    ClearRecentScenes,
    LoadDemo(&'static str, &'static [u8]),
    StartServer,
    StopServer,
    Disconnect,
    BeginRename,
    RestartAudio,
    Exit,
}

// ── Menu definitions ──────────────────────────────────────────────────────────

pub(crate) struct MenuDef {
    pub label: String,
    pub mnemonic: char,
    pub items: Vec<MenuItemDef>,
}

pub(crate) enum MenuItemDef {
    Button {
        label: String,
        mnemonic: char,
        icon: Option<&'static str>,
        enabled: bool,
        action: MenuAction,
    },
    Checkbox {
        label: String,
        mnemonic: char,
        checked: bool,
        shortcut_text: Option<String>,
        enabled: bool,
        action: MenuAction,
    },
    SubMenu {
        label: String,
        mnemonic: char,
        enabled: bool,
        items: Vec<MenuItemDef>,
    },
    Separator,
}

impl MenuItemDef {
    fn is_navigable(&self) -> bool {
        match self {
            Self::Separator => false,
            Self::Button { enabled, .. }
            | Self::Checkbox { enabled, .. }
            | Self::SubMenu { enabled, .. } => *enabled,
        }
    }

    fn mnemonic(&self) -> Option<char> {
        match self {
            Self::Button { mnemonic, .. }
            | Self::Checkbox { mnemonic, .. }
            | Self::SubMenu { mnemonic, .. } => Some(*mnemonic),
            Self::Separator => None,
        }
    }

    fn action(&self) -> Option<MenuAction> {
        match self {
            Self::Button {
                action,
                enabled: true,
                ..
            }
            | Self::Checkbox {
                action,
                enabled: true,
                ..
            } => Some(action.clone()),
            _ => None,
        }
    }
}

// ── State machine ─────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct MenuLevelState {
    item_idx: Option<usize>,
}

enum MenuBarMode {
    Closed,
    Active {
        menu_idx: usize,
        levels: Vec<MenuLevelState>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuInteractionMode {
    Mouse,
    Keyboard,
}

pub(crate) struct MenuBarState {
    mode: MenuBarMode,
    pending_action: Option<MenuAction>,
    button_rects: Vec<egui::Rect>,
    interaction_mode: MenuInteractionMode,
}

impl MenuBarState {
    pub(crate) fn new() -> Self {
        Self {
            mode: MenuBarMode::Closed,
            pending_action: None,
            button_rects: Vec::new(),
            interaction_mode: MenuInteractionMode::Mouse,
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        matches!(self.mode, MenuBarMode::Active { .. })
    }

    pub(crate) fn activate(&mut self, menu_idx: usize) {
        self.set_active_menu(menu_idx, MenuInteractionMode::Keyboard);
    }

    fn set_active_menu(&mut self, menu_idx: usize, interaction_mode: MenuInteractionMode) {
        self.mode = MenuBarMode::Active {
            menu_idx,
            levels: vec![MenuLevelState::default()],
        };
        self.interaction_mode = interaction_mode;
    }

    pub(crate) fn take_action(&mut self) -> Option<MenuAction> {
        self.pending_action.take()
    }

    /// Handle keyboard navigation. Must be called before show() each frame.
    pub(crate) fn handle_input(&mut self, ctx: &egui::Context, menus: &[MenuDef]) {
        if menus.is_empty() {
            self.mode = MenuBarMode::Closed;
            return;
        }

        let mut top_level_switch = None;
        let keyboard_used;
        {
            let MenuBarMode::Active { menu_idx, levels } = &mut self.mode else {
                return;
            };
            let menu = &menus[*menu_idx];
            clamp_levels(menu, levels);

            // All reads use consume_key / drain events so no other component
            // (e.g. a focused TextEdit) receives the same events this frame.
            let (esc, enter, left, right, up, down) = ctx.input_mut(|i| {
                (
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                )
            });
            let char_pressed: Option<char> = ctx.input_mut(|i| {
                let ch = i.events.iter().find_map(|e| {
                    if let egui::Event::Text(s) = e {
                        s.chars().next()
                    } else {
                        None
                    }
                });
                if ch.is_some() {
                    i.events.retain(|e| !matches!(e, egui::Event::Text(_)));
                }
                ch
            });

            let n_menus = menus.len();
            let depth = levels.len().saturating_sub(1);
            let Some(items) = items_at_depth(menu, levels, depth) else {
                self.mode = MenuBarMode::Closed;
                return;
            };
            keyboard_used = esc || enter || left || right || up || down || char_pressed.is_some();

            if esc {
                if levels.len() > 1 {
                    levels.pop();
                } else {
                    self.mode = MenuBarMode::Closed;
                }
            } else if left {
                if levels.len() > 1 {
                    levels.pop();
                } else {
                    top_level_switch = Some(menu_idx.checked_sub(1).unwrap_or(n_menus - 1));
                }
            } else if right {
                let selected = levels[depth].item_idx.and_then(|idx| items.get(idx));
                if matches!(selected, Some(MenuItemDef::SubMenu { enabled: true, .. })) {
                    open_submenu(levels, depth);
                } else if depth == 0 {
                    top_level_switch = Some((*menu_idx + 1) % n_menus);
                }
            } else if up {
                levels[depth].item_idx = next_navigable(items, levels[depth].item_idx, -1);
            } else if down {
                levels[depth].item_idx = next_navigable(items, levels[depth].item_idx, 1);
            } else if enter {
                if let Some(idx) = levels[depth].item_idx {
                    if matches!(items[idx], MenuItemDef::SubMenu { enabled: true, .. }) {
                        open_submenu(levels, depth);
                    } else if let Some(action) = items[idx].action() {
                        self.pending_action = Some(action);
                        self.mode = MenuBarMode::Closed;
                    }
                }
            } else if let Some(ch) = char_pressed {
                match find_mnemonic_match(items, ch) {
                    Some(MnemonicMatch::Action(action)) => {
                        self.pending_action = Some(action);
                        self.mode = MenuBarMode::Closed;
                    }
                    Some(MnemonicMatch::OpenSubmenu(idx)) => {
                        focus_item(levels, depth, idx, true);
                    }
                    None => {}
                }
            }
        }

        if let Some(menu_idx) = top_level_switch {
            self.set_active_menu(menu_idx, MenuInteractionMode::Keyboard);
        } else if keyboard_used {
            self.interaction_mode = MenuInteractionMode::Keyboard;
        }
    }

    /// Render the menu bar buttons and dropdown. Call take_action() after to retrieve any triggered action.
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, menus: &[MenuDef]) {
        let n = menus.len();
        self.button_rects.resize(n, egui::Rect::NOTHING);
        let mouse_reasserted =
            ui.input(|i| i.pointer.is_moving() || i.pointer.any_pressed() || i.pointer.any_click());
        if mouse_reasserted {
            self.interaction_mode = MenuInteractionMode::Mouse;
        }
        let follow_mouse = self.interaction_mode == MenuInteractionMode::Mouse;

        let is_active = self.is_active();
        let active_menu_idx = match &self.mode {
            MenuBarMode::Active { menu_idx, .. } => *menu_idx,
            MenuBarMode::Closed => usize::MAX,
        };

        // Render top-level buttons
        let mut clicked_btn: Option<usize> = None;
        let mut hovered_btn: Option<usize> = None;

        with_menu_style(ui, |ui| {
            ui.set_min_height(ui.spacing().interact_size.y);

            for (i, menu) in menus.iter().enumerate() {
                let is_this_active = is_active && i == active_menu_idx;
                let fg = if is_this_active {
                    ui.visuals().widgets.open.fg_stroke.color
                } else {
                    ui.visuals().widgets.inactive.fg_stroke.color
                };
                let job = label_job(ui, &menu.label, menu.mnemonic, false, fg);
                let r = ui.selectable_label(is_this_active, job);
                self.button_rects[i] = r.rect;
                if r.clicked() {
                    clicked_btn = Some(i);
                }
                if follow_mouse && r.hovered() {
                    hovered_btn = Some(i);
                }
            }
        });

        // Handle click on top-level button
        if let Some(i) = clicked_btn {
            if is_active && i == active_menu_idx {
                self.mode = MenuBarMode::Closed;
            } else {
                self.set_active_menu(i, MenuInteractionMode::Mouse);
            }
        }
        // Hover-switch when a menu is already open
        if let Some(i) = hovered_btn
            && is_active
            && i != active_menu_idx
        {
            self.set_active_menu(i, MenuInteractionMode::Mouse);
        }

        // Show dropdown and any nested submenus if active
        if let MenuBarMode::Active {
            menu_idx,
            ref mut levels,
        } = self.mode
        {
            let menu = &menus[menu_idx];
            clamp_levels(menu, levels);
            let btn_rect = self
                .button_rects
                .get(menu_idx)
                .copied()
                .unwrap_or(egui::Rect::NOTHING);
            let dropdown_pos = egui::pos2(btn_rect.left(), btn_rect.bottom());
            let (submenu_gap, submenu_y_offset) = menu_popup_metrics(ui.style());
            let mut clicked_action: Option<MenuAction> = None;
            let mut menu_rects = Vec::new();
            let mut parent_item_rects: Option<Vec<egui::Rect>> = None;
            let mut depth = 0;

            while depth < levels.len() {
                let Some(items) = items_at_depth(menu, levels, depth) else {
                    levels.truncate(depth.max(1));
                    break;
                };
                let pos = if depth == 0 {
                    dropdown_pos
                } else {
                    let Some(parent_rects) = &parent_item_rects else {
                        break;
                    };
                    let Some(parent_idx) = levels[depth - 1].item_idx else {
                        break;
                    };
                    let Some(parent_rect) = parent_rects.get(parent_idx).copied() else {
                        break;
                    };
                    egui::pos2(
                        parent_rect.right() + submenu_gap,
                        parent_rect.top() - submenu_y_offset,
                    )
                };

                let is_deepest = depth + 1 == levels.len();
                let open_parent_idx = levels.get(depth).and_then(|level| level.item_idx);
                let panel = show_menu_panel(
                    ui.ctx(),
                    pos,
                    egui::Id::new(("menu_bar_panel", menu_idx, depth)),
                    items,
                    ui.style().as_ref(),
                    levels[depth].item_idx,
                    is_deepest,
                    follow_mouse,
                    |idx| depth + 1 < levels.len() && open_parent_idx == Some(idx),
                );
                menu_rects.push(panel.rect);
                parent_item_rects = Some(panel.item_rects);

                if let Some(idx) = panel.hovered_idx {
                    let opens_child =
                        matches!(items[idx], MenuItemDef::SubMenu { enabled: true, .. });
                    focus_item(levels, depth, idx, opens_child);
                }

                if let Some(clicked) = panel.clicked {
                    match clicked {
                        MenuPanelClick::Action(action) => {
                            clicked_action = Some(action);
                            break;
                        }
                        MenuPanelClick::OpenSubmenu(idx) => {
                            focus_item(levels, depth, idx, true);
                        }
                    }
                }

                depth += 1;
            }

            let clicked_outside = ui.input(|i| i.pointer.any_pressed())
                && ui.input(|i| {
                    i.pointer.interact_pos().is_some_and(|p| {
                        !self.button_rects.iter().any(|rect| rect.contains(p))
                            && !menu_rects.iter().any(|rect| rect.contains(p))
                    })
                });
            if clicked_outside {
                self.mode = MenuBarMode::Closed;
                return;
            }

            if let Some(action) = clicked_action {
                self.pending_action = Some(action);
                self.mode = MenuBarMode::Closed;
            }
        }
    }
}

