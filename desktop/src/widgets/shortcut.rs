use eframe::egui;

#[derive(Clone, Copy, Debug)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Space,
    Delete,
    Up,
    Down,
    F(u8),
    Literal(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub struct Shortcut {
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub key: Key,
}

#[derive(Clone, Copy, Debug)]
pub enum Enabled {
    Yes,
    No,
}

impl Shortcut {
    pub const fn plain(key: Key) -> Self {
        Self {
            cmd: false,
            shift: false,
            alt: false,
            ctrl: false,
            key,
        }
    }

    pub const fn cmd(key: Key) -> Self {
        Self {
            cmd: true,
            shift: false,
            alt: false,
            ctrl: false,
            key,
        }
    }

    pub const fn cmd_shift(key: Key) -> Self {
        Self {
            cmd: true,
            shift: true,
            alt: false,
            ctrl: false,
            key,
        }
    }

    pub const fn shift(key: Key) -> Self {
        Self {
            cmd: false,
            shift: true,
            alt: false,
            ctrl: false,
            key,
        }
    }

    pub const fn alt(key: Key) -> Self {
        Self {
            cmd: false,
            shift: false,
            alt: true,
            ctrl: false,
            key,
        }
    }

    pub const fn literal(s: &'static str) -> Self {
        Self {
            cmd: false,
            shift: false,
            alt: false,
            ctrl: false,
            key: Key::Literal(s),
        }
    }

    /// Returns true when the key combo was pressed this frame.
    /// `Key::Literal` shortcuts (display-only) never match.
    pub fn pressed(&self, ctx: &egui::Context) -> bool {
        let Some(key) = self.egui_key() else {
            return false;
        };
        ctx.input(|i| {
            i.key_pressed(key)
                && i.modifiers.command == self.cmd
                && i.modifiers.shift == self.shift
                && i.modifiers.alt == self.alt
        })
    }

    fn egui_key(&self) -> Option<egui::Key> {
        match self.key {
            Key::Char(c) => char_to_egui_key(c),
            Key::Enter => Some(egui::Key::Enter),
            Key::Esc => Some(egui::Key::Escape),
            Key::Space => Some(egui::Key::Space),
            Key::Delete => Some(egui::Key::Delete),
            Key::Up => Some(egui::Key::ArrowUp),
            Key::Down => Some(egui::Key::ArrowDown),
            Key::F(n) => match n {
                1 => Some(egui::Key::F1),
                2 => Some(egui::Key::F2),
                3 => Some(egui::Key::F3),
                4 => Some(egui::Key::F4),
                5 => Some(egui::Key::F5),
                6 => Some(egui::Key::F6),
                7 => Some(egui::Key::F7),
                8 => Some(egui::Key::F8),
                9 => Some(egui::Key::F9),
                10 => Some(egui::Key::F10),
                11 => Some(egui::Key::F11),
                12 => Some(egui::Key::F12),
                _ => None,
            },
            Key::Literal(_) => None,
        }
    }
}

fn char_to_egui_key(c: char) -> Option<egui::Key> {
    let upper = c.to_ascii_uppercase();
    match upper {
        'A' => Some(egui::Key::A),
        'B' => Some(egui::Key::B),
        'C' => Some(egui::Key::C),
        'D' => Some(egui::Key::D),
        'E' => Some(egui::Key::E),
        'F' => Some(egui::Key::F),
        'G' => Some(egui::Key::G),
        'H' => Some(egui::Key::H),
        'I' => Some(egui::Key::I),
        'J' => Some(egui::Key::J),
        'K' => Some(egui::Key::K),
        'L' => Some(egui::Key::L),
        'M' => Some(egui::Key::M),
        'N' => Some(egui::Key::N),
        'O' => Some(egui::Key::O),
        'P' => Some(egui::Key::P),
        'Q' => Some(egui::Key::Q),
        'R' => Some(egui::Key::R),
        'S' => Some(egui::Key::S),
        'T' => Some(egui::Key::T),
        'U' => Some(egui::Key::U),
        'V' => Some(egui::Key::V),
        'W' => Some(egui::Key::W),
        'X' => Some(egui::Key::X),
        'Y' => Some(egui::Key::Y),
        'Z' => Some(egui::Key::Z),
        '0' => Some(egui::Key::Num0),
        '1' => Some(egui::Key::Num1),
        '2' => Some(egui::Key::Num2),
        '3' => Some(egui::Key::Num3),
        '4' => Some(egui::Key::Num4),
        '5' => Some(egui::Key::Num5),
        '6' => Some(egui::Key::Num6),
        '7' => Some(egui::Key::Num7),
        '8' => Some(egui::Key::Num8),
        '9' => Some(egui::Key::Num9),
        ',' => Some(egui::Key::Comma),
        '.' => Some(egui::Key::Period),
        '-' => Some(egui::Key::Minus),
        '=' => Some(egui::Key::Equals),
        '/' => Some(egui::Key::Slash),
        ';' => Some(egui::Key::Semicolon),
        _ => None,
    }
}

fn format_key(key: &Key, mac: bool) -> String {
    match key {
        Key::Char(c) => c.to_ascii_uppercase().to_string(),
        Key::Enter => {
            if mac {
                "⏎".into()
            } else {
                "Enter".into()
            }
        }
        Key::Esc => {
            if mac {
                "⎋".into()
            } else {
                "Esc".into()
            }
        }
        Key::Space => "Space".into(),
        Key::Delete => {
            if mac {
                "⌦".into()
            } else {
                "Del".into()
            }
        }
        Key::Up => "↑".into(),
        Key::Down => "↓".into(),
        Key::F(n) => format!("F{n}"),
        Key::Literal(s) => (*s).to_string(),
    }
}

fn format_glyph(sc: &Shortcut, os: egui::os::OperatingSystem) -> String {
    let mac = os == egui::os::OperatingSystem::Mac;

    if let Key::Literal(s) = sc.key {
        return s.to_string();
    }

    let key_glyph = format_key(&sc.key, mac);

    if mac {
        // Apple HIG modifier order: Control, Option, Shift, Command, then key.
        let mut s = String::new();
        if sc.ctrl {
            s.push('⌃');
        }
        if sc.alt {
            s.push('⌥');
        }
        if sc.shift {
            s.push('⇧');
        }
        if sc.cmd {
            s.push('⌘');
        }
        s.push_str(&key_glyph);
        s
    } else {
        let mut parts: Vec<&str> = Vec::new();
        if sc.cmd || sc.ctrl {
            parts.push("Ctrl");
        }
        if sc.shift {
            parts.push("Shift");
        }
        if sc.alt {
            parts.push("Alt");
        }
        let mut s = parts.join("+");
        if !s.is_empty() {
            s.push('+');
        }
        s.push_str(&key_glyph);
        s
    }
}

fn format_plain(sc: &Shortcut) -> String {
    if let Key::Literal(s) = sc.key {
        return s.to_string();
    }

    let mut parts: Vec<&str> = Vec::new();
    if sc.cmd {
        parts.push("Cmd");
    }
    if sc.ctrl {
        parts.push("Ctrl");
    }
    if sc.shift {
        parts.push("Shift");
    }
    if sc.alt {
        parts.push("Alt");
    }

    let mut s = parts.join("+");
    if !s.is_empty() {
        s.push('+');
    }
    s.push_str(&format_key(&sc.key, false));
    s
}

pub fn format(ctx: &egui::Context, sc: &Shortcut) -> String {
    format_glyph(sc, ctx.os())
}

pub fn format_plain_text(sc: &Shortcut) -> String {
    format_plain(sc)
}

pub fn badge_text(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    bg: egui::Color32,
    fg: egui::Color32,
    font_size: f32,
    pad: egui::Vec2,
) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(text.into(), egui::FontId::monospace(font_size), fg);
    let (rect, resp) = ui.allocate_exact_size(galley.size() + pad * 2.0, egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, bg);
    ui.painter().galley(rect.min + pad, galley, fg);
    resp
}

pub fn labeled(ui: &egui::Ui, label: &str, sc: &Shortcut, state: Enabled) -> egui::WidgetText {
    let style = ui.style();
    let font = egui::TextStyle::Button.resolve(style);
    let label_color = match state {
        Enabled::Yes => style.visuals.text_color(),
        Enabled::No => style.visuals.weak_text_color(),
    };
    let mut job = egui::text::LayoutJob::default();
    job.append(
        label,
        0.0,
        egui::TextFormat {
            font_id: font.clone(),
            color: label_color,
            ..Default::default()
        },
    );
    let glyph = format(ui.ctx(), sc);
    job.append(
        &format!("  {glyph}"),
        0.0,
        egui::TextFormat {
            font_id: font,
            color: style.visuals.weak_text_color(),
            ..Default::default()
        },
    );
    job.into()
}

pub fn grid_row(ui: &mut egui::Ui, label: impl Into<egui::WidgetText>, sc: &Shortcut) {
    ui.label(label);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.monospace(format(ui.ctx(), sc));
    });
    ui.end_row();
}
