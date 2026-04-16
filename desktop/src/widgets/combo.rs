use eframe::egui;

pub const COMBO_WIDTH: f32 = 160.0;
const SEARCH_LIST_MAX_HEIGHT: f32 = 200.0;

/// ComboBox for selecting a `String` from a list of names, with an optional
/// "default" sentinel that maps to the empty string.
///
/// Returns `true` when the selection changes.
pub fn string_list(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    selected: &mut String,
    default_label: Option<&str>,
    items: impl IntoIterator<Item = impl AsRef<str>>,
) -> bool {
    let display: String = if selected.is_empty() {
        default_label.unwrap_or("").to_owned()
    } else {
        selected.clone()
    };
    let mut changed = false;
    egui::ComboBox::from_id_salt(id)
        .selected_text(display)
        .width(COMBO_WIDTH)
        .show_ui(ui, |ui| {
            if let Some(label) = default_label
                && ui
                    .selectable_value(selected, String::new(), label)
                    .changed()
            {
                changed = true;
            }
            for item in items {
                let s = item.as_ref();
                if ui.selectable_value(selected, s.to_owned(), s).changed() {
                    changed = true;
                }
            }
        });
    changed
}

/// Like [`string_list`] but with a text-filter input at the top of the popup.
/// Designed for long lists (e.g. system fonts). Uses fuzzy matching from
/// [`super::fuzzy_score`] when a filter is active.
///
/// `items` must be a slice so we can filter and sort without consuming an iterator.
pub fn searchable_string_list(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    selected: &mut String,
    default_label: Option<&str>,
    items: &[String],
) -> bool {
    let display: String = if selected.is_empty() {
        default_label.unwrap_or("").to_owned()
    } else {
        selected.clone()
    };

    let combo_id = egui::Id::new(id);
    let filter_id = combo_id.with("_filter");

    let mut changed = false;

    let mut combo = egui::ComboBox::from_id_salt(combo_id)
        .selected_text(display)
        .width(COMBO_WIDTH);
    combo = combo.height(SEARCH_LIST_MAX_HEIGHT);

    let resp = combo.show_ui(ui, |ui| {
        let mut filter: String = ui.data(|d| d.get_temp(filter_id).unwrap_or_default());

        let te = egui::TextEdit::singleline(&mut filter)
            .hint_text(t!("common.search"))
            .desired_width(COMBO_WIDTH);
        let te_resp = ui.add(te);

        // Keep focus on the text input while the popup is open.
        te_resp.request_focus();

        ui.data_mut(|d| d.insert_temp(filter_id, filter.clone()));

        ui.separator();

        if filter.is_empty() {
            // No filter: show default + all items.
            if let Some(label) = default_label
                && ui
                    .selectable_value(selected, String::new(), label)
                    .changed()
            {
                changed = true;
            }
            for item in items {
                if ui
                    .selectable_value(selected, item.clone(), item.as_str())
                    .changed()
                {
                    changed = true;
                }
            }
        } else {
            // Fuzzy-filter and sort by score.
            let mut scored: Vec<(i32, usize)> = items
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    super::fuzzy_score(&filter, name).map(|(score, _)| (score, i))
                })
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0));

            // Also match default label if present.
            if let Some(label) = default_label
                && super::fuzzy_score(&filter, label).is_some()
                && ui
                    .selectable_value(selected, String::new(), label)
                    .changed()
            {
                changed = true;
            }

            for &(_, idx) in &scored {
                let name = &items[idx];
                if ui
                    .selectable_value(selected, name.clone(), name.as_str())
                    .changed()
                {
                    changed = true;
                }
            }
        }
    });

    // Clear filter when popup closes.
    if resp.inner.is_none() {
        ui.data_mut(|d| d.insert_temp(filter_id, String::new()));
    }

    // Clear filter after a selection so reopening starts fresh.
    if changed {
        ui.data_mut(|d| d.insert_temp(filter_id, String::new()));
    }

    changed
}
