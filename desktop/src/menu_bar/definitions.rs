use std::path::PathBuf;

use eframe::egui;

use crate::scene_panel::ViewMode;
use crate::widgets::{
    CommandId,
    shortcut::{self, Key, Shortcut},
};

use super::{MenuAction, MenuDef, MenuItemDef};

pub(crate) struct MenuContext<'a> {
    pub connected: bool,
    pub server_running: bool,
    pub scope_open: bool,
    pub spectrum_open: bool,
    pub vu_meter_open: bool,
    pub scope_bar_open: bool,
    pub chat_open: bool,
    pub sample_browser_open: bool,
    pub sample_browser_available: bool,
    pub debug_open: bool,
    pub scene_view_mode: ViewMode,
    pub recent_scenes: &'a [PathBuf],
    pub egui_ctx: &'a egui::Context,
}

pub(crate) fn build_menus(ctx: &MenuContext<'_>) -> Vec<MenuDef> {
    use sova_server::demos::{DEMOS_BOINX, DEMOS_CAGIRE, DEMOS_GENERAL};

    let sc = |sc: &Shortcut| {
        if ctx.egui_ctx.os() == egui::os::OperatingSystem::Mac {
            shortcut::format_plain_text(sc)
        } else {
            shortcut::format(ctx.egui_ctx, sc)
        }
    };

    // ── Scene ──
    let recent_items: Vec<MenuItemDef> = ctx
        .recent_scenes
        .iter()
        .filter(|p| p.exists())
        .map(|p| {
            let label = p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.display().to_string());
            MenuItemDef::Button {
                label,
                mnemonic: '\0',
                icon: None,
                enabled: true,
                action: MenuAction::LoadRecentScene(p.clone()),
            }
        })
        .chain(std::iter::once(MenuItemDef::Separator))
        .chain(std::iter::once(MenuItemDef::Button {
            label: t!("menu.clear").into_owned(),
            mnemonic: 'C',
            icon: Some(crate::icons::TRASH),
            enabled: true,
            action: MenuAction::ClearRecentScenes,
        }))
        .collect();

    let demos_submenu = |label: &str, demos: &'static [(&'static str, &'static [u8])]| {
        let items: Vec<MenuItemDef> = demos
            .iter()
            .map(|(name, bytes)| {
                if *name == "\x00" {
                    MenuItemDef::Separator
                } else {
                    MenuItemDef::Button {
                        label: (*name).to_string(),
                        mnemonic: name.chars().next().unwrap_or('\0'),
                        icon: None,
                        enabled: true,
                        action: MenuAction::LoadDemo(name, bytes),
                    }
                }
            })
            .collect();
        MenuItemDef::SubMenu {
            label: label.to_string(),
            mnemonic: label.chars().next().unwrap_or('\0'),
            enabled: items.iter().any(MenuItemDef::is_navigable),
            items,
        }
    };

    let scene = MenuDef {
        label: format!("{} (F1)", t!("menu.file")),
        mnemonic: 'S',
        items: vec![
            MenuItemDef::Button {
                label: t!("menu.save_scene").into_owned(),
                mnemonic: 'S',
                icon: None,
                enabled: ctx.connected,
                action: MenuAction::Command(CommandId::SaveScene),
            },
            MenuItemDef::Button {
                label: t!("menu.load_scene").into_owned(),
                mnemonic: 'L',
                icon: None,
                enabled: ctx.connected,
                action: MenuAction::Command(CommandId::LoadScene),
            },
            MenuItemDef::Button {
                label: t!("menu.load_scene_at_end").into_owned(),
                mnemonic: 'E',
                icon: None,
                enabled: ctx.connected,
                action: MenuAction::LoadSceneAtEnd,
            },
            MenuItemDef::Button {
                label: t!("menu.reset_scene").into_owned(),
                mnemonic: 'R',
                icon: None,
                enabled: ctx.connected,
                action: MenuAction::Command(CommandId::ResetScene),
            },
            MenuItemDef::SubMenu {
                label: t!("menu.recent").into_owned(),
                mnemonic: 'c',
                enabled: ctx.connected && !ctx.recent_scenes.is_empty(),
                items: recent_items,
            },
            MenuItemDef::SubMenu {
                label: t!("menu.demos").into_owned(),
                mnemonic: 'D',
                enabled: ctx.connected,
                items: vec![
                    demos_submenu("Cagire", DEMOS_CAGIRE),
                    demos_submenu("Boinx", DEMOS_BOINX),
                    demos_submenu("Demos", DEMOS_GENERAL),
                ],
            },
            MenuItemDef::Separator,
            MenuItemDef::Button {
                label: t!("menu.exit").into_owned(),
                mnemonic: 'x',
                icon: None,
                enabled: true,
                action: MenuAction::Exit,
            },
        ],
    };

    // ── Server ──
    let mut server_items = if ctx.server_running {
        vec![MenuItemDef::Button {
            label: t!("menu.stop_server").into_owned(),
            mnemonic: 'S',
            icon: Some(crate::icons::STOP),
            enabled: true,
            action: MenuAction::StopServer,
        }]
    } else {
        vec![MenuItemDef::Button {
            label: t!("menu.start_server").into_owned(),
            mnemonic: 'S',
            icon: Some(crate::icons::PLAY),
            enabled: true,
            action: MenuAction::StartServer,
        }]
    };
    if ctx.connected {
        server_items.push(MenuItemDef::Separator);
        server_items.push(MenuItemDef::Button {
            label: t!("common.disconnect").into_owned(),
            mnemonic: 'd',
            icon: Some(crate::icons::DISCONNECT),
            enabled: true,
            action: MenuAction::Disconnect,
        });
        server_items.push(MenuItemDef::Button {
            label: t!("menu.rename").into_owned(),
            mnemonic: 'R',
            icon: None,
            enabled: true,
            action: MenuAction::BeginRename,
        });
    }
    let server = MenuDef {
        label: format!("{} (F2)", t!("menu.server")),
        mnemonic: 'e',
        items: server_items,
    };

    // ── Engine ──
    let engine = MenuDef {
        label: format!("{} (F3)", t!("menu.engine")),
        mnemonic: 'n',
        items: vec![
            MenuItemDef::Button {
                label: t!("menu.restart_audio").into_owned(),
                mnemonic: 'A',
                icon: None,
                enabled: ctx.connected,
                action: MenuAction::RestartAudio,
            },
            MenuItemDef::Button {
                label: t!("menu.restart_core").into_owned(),
                mnemonic: 'C',
                icon: None,
                enabled: ctx.connected,
                action: MenuAction::Command(CommandId::RestartCore),
            },
        ],
    };

    // ── Audio ──
    let audio = MenuDef {
        label: format!("{} (F4)", t!("menu.audio")),
        mnemonic: 'A',
        items: vec![
            MenuItemDef::Checkbox {
                label: t!("scope.title").into_owned(),
                mnemonic: 'O',
                checked: ctx.scope_open,
                shortcut_text: Some(sc(&Shortcut::cmd_shift(Key::Char('O')))),
                enabled: true,
                action: MenuAction::Command(CommandId::Scope),
            },
            MenuItemDef::Checkbox {
                label: t!("spectrum.title").into_owned(),
                mnemonic: 'P',
                checked: ctx.spectrum_open,
                shortcut_text: Some(sc(&Shortcut::cmd_shift(Key::Char('P')))),
                enabled: true,
                action: MenuAction::Command(CommandId::Spectrum),
            },
            MenuItemDef::Checkbox {
                label: t!("cmd.vu_meter").into_owned(),
                mnemonic: 'U',
                checked: ctx.vu_meter_open,
                shortcut_text: Some(sc(&Shortcut::cmd_shift(Key::Char('U')))),
                enabled: true,
                action: MenuAction::Command(CommandId::VuMeter),
            },
            MenuItemDef::Checkbox {
                label: t!("cmd.scope_bar").into_owned(),
                mnemonic: 'W',
                checked: ctx.scope_bar_open,
                shortcut_text: Some(sc(&Shortcut::cmd_shift(Key::Char('W')))),
                enabled: true,
                action: MenuAction::Command(CommandId::ScopeBar),
            },
        ],
    };

    // ── View ──
    let scene_view_submenu = MenuItemDef::SubMenu {
        label: t!("menu.view.scene_view").into_owned(),
        mnemonic: 'S',
        enabled: true,
        items: vec![
            MenuItemDef::Checkbox {
                label: t!("options.scene_view_mode.sequencer").into_owned(),
                mnemonic: 'Q',
                checked: ctx.scene_view_mode == ViewMode::Sequencer,
                shortcut_text: None,
                enabled: true,
                action: MenuAction::SetSceneViewMode(ViewMode::Sequencer),
            },
            MenuItemDef::Checkbox {
                label: t!("options.scene_view_mode.stack").into_owned(),
                mnemonic: 'T',
                checked: ctx.scene_view_mode == ViewMode::Stack,
                shortcut_text: Some(sc(&Shortcut::plain(Key::Char('V')))),
                enabled: true,
                action: MenuAction::SetSceneViewMode(ViewMode::Stack),
            },
        ],
    };

    let view = MenuDef {
        label: format!("{} (F5)", t!("menu.view")),
        mnemonic: 'V',
        items: vec![
            scene_view_submenu,
            MenuItemDef::Separator,
            MenuItemDef::Checkbox {
                label: t!("chat.title").into_owned(),
                mnemonic: 'h',
                checked: ctx.chat_open,
                shortcut_text: Some(sc(&Shortcut::cmd_shift(Key::Char('C')))),
                enabled: true,
                action: MenuAction::Command(CommandId::Chat),
            },
            MenuItemDef::Checkbox {
                label: t!("sample_browser.title").into_owned(),
                mnemonic: 'B',
                checked: ctx.sample_browser_open,
                shortcut_text: Some(sc(&Shortcut::cmd_shift(Key::Char('E')))),
                enabled: ctx.sample_browser_available,
                action: MenuAction::Command(CommandId::SampleBrowser),
            },
            MenuItemDef::Separator,
            MenuItemDef::Checkbox {
                label: t!("debug.title").into_owned(),
                mnemonic: 'D',
                checked: ctx.debug_open,
                shortcut_text: Some(sc(&Shortcut::cmd_shift(Key::Char('B')))),
                enabled: true,
                action: MenuAction::Command(CommandId::Debug),
            },
            MenuItemDef::Separator,
            MenuItemDef::Button {
                label: t!("menu.keybindings").into_owned(),
                mnemonic: 'K',
                icon: Some(crate::icons::KEYBOARD),
                enabled: true,
                action: MenuAction::Command(CommandId::Keybindings),
            },
        ],
    };

    vec![scene, server, engine, audio, view]
}
