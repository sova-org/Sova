use super::{MenuDef, MenuItemDef, MenuLevelState};

pub(super) enum MnemonicMatch {
    Action(super::MenuAction),
    OpenSubmenu(usize),
}

pub(super) fn next_navigable(items: &[MenuItemDef], current: Option<usize>, dir: i32) -> Option<usize> {
    let n = items.len();
    if n == 0 {
        return None;
    }
    let start = match current {
        None => {
            if dir > 0 {
                0
            } else {
                n - 1
            }
        }
        Some(i) => (i as i64 + dir as i64).rem_euclid(n as i64) as usize,
    };
    for offset in 0..n {
        let idx =
            (start as i64 + offset as i64 * dir.signum() as i64).rem_euclid(n as i64) as usize;
        if items[idx].is_navigable() {
            return Some(idx);
        }
    }
    None
}

pub(super) fn find_mnemonic_match(items: &[MenuItemDef], ch: char) -> Option<MnemonicMatch> {
    let lower = ch.to_ascii_lowercase();
    items.iter().enumerate().find_map(|(idx, item)| {
        if item.mnemonic()?.to_ascii_lowercase() != lower {
            return None;
        }

        match item {
            MenuItemDef::SubMenu { enabled: true, .. } => Some(MnemonicMatch::OpenSubmenu(idx)),
            _ => item.action().map(MnemonicMatch::Action),
        }
    })
}

pub(super) fn items_at_depth<'a>(
    menu: &'a MenuDef,
    levels: &[MenuLevelState],
    depth: usize,
) -> Option<&'a [MenuItemDef]> {
    let mut items = menu.items.as_slice();
    for level in levels.iter().take(depth) {
        let idx = level.item_idx?;
        match items.get(idx)? {
            MenuItemDef::SubMenu { items: child, .. } => {
                items = child.as_slice();
            }
            _ => return None,
        }
    }
    Some(items)
}

pub(super) fn clamp_levels(menu: &MenuDef, levels: &mut Vec<MenuLevelState>) {
    if levels.is_empty() {
        levels.push(MenuLevelState::default());
        return;
    }

    let mut max_depth = 1;
    while max_depth < levels.len() {
        if items_at_depth(menu, levels, max_depth).is_some() {
            max_depth += 1;
        } else {
            break;
        }
    }
    levels.truncate(max_depth);
}

pub(super) fn open_submenu(levels: &mut Vec<MenuLevelState>, depth: usize) {
    if levels.len() == depth + 1 {
        levels.push(MenuLevelState::default());
    }
}

pub(super) fn focus_item(levels: &mut Vec<MenuLevelState>, depth: usize, idx: usize, opens_child: bool) {
    let keep_child = opens_child
        && levels.get(depth).and_then(|level| level.item_idx) == Some(idx)
        && levels.len() > depth + 1;

    levels[depth].item_idx = Some(idx);
    if opens_child {
        if !keep_child {
            levels.truncate(depth + 1);
        }
        open_submenu(levels, depth);
    } else {
        levels.truncate(depth + 1);
    }
}
