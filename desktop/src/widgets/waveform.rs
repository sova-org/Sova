use eframe::egui;

enum WaveformData<'a> {
    Peaks { samples: &'a [f32], num_bins: usize },
    Line(&'a [f32]),
}

pub struct Waveform<'a> {
    data: WaveformData<'a>,
    peaks: Option<&'a [(f32, f32)]>,
    fill_peaks: Option<&'a [(f32, f32)]>,
    trace: Option<&'a [(f32, f32)]>,
    color: egui::Color32,
    fill_alpha: f32,
    stroke_width: f32,
    normalize: bool,
    cursor_pos: Option<f32>,
}

impl<'a> Waveform<'a> {
    pub fn new(samples: &'a [f32], color: egui::Color32) -> Self {
        Self {
            data: WaveformData::Peaks { samples, num_bins: 256 },
            peaks: None,
            fill_peaks: None,
            trace: None,
            color,
            fill_alpha: 0.35,
            stroke_width: 1.0,
            normalize: false,
            cursor_pos: None,
        }
    }

    pub fn from_line(line: &'a [f32], color: egui::Color32) -> Self {
        Self {
            data: WaveformData::Line(line),
            peaks: None,
            fill_peaks: None,
            trace: None,
            color,
            fill_alpha: 0.35,
            stroke_width: 1.0,
            normalize: false,
            cursor_pos: None,
        }
    }

    pub fn with_trace(mut self, trace: &'a [(f32, f32)]) -> Self {
        self.trace = Some(trace);
        self
    }

    pub fn cursor(mut self, pos: Option<f32>) -> Self {
        self.cursor_pos = pos;
        self
    }

    pub fn stroke_width(mut self, w: f32) -> Self {
        self.stroke_width = w;
        self
    }

    pub fn fill_alpha(mut self, a: f32) -> Self {
        self.fill_alpha = a;
        self
    }

    pub fn normalize(mut self, on: bool) -> Self {
        self.normalize = on;
        self
    }

    pub fn num_bins(mut self, n: usize) -> Self {
        if let WaveformData::Peaks { ref mut num_bins, .. } = self.data {
            *num_bins = n;
        }
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) -> Option<f32> {
        let avail = ui.available_size();
        let desired = egui::vec2(avail.x, avail.y.max(60.0));
        let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
        let painter = ui.painter_at(rect);

        let center_y = rect.center().y;

        match &self.data {
            WaveformData::Line(line) => {
                if line.len() < 2 {
                    return self.handle_click(&resp, rect);
                }
                self.draw_line(&painter, rect, center_y, line);
            }
            WaveformData::Peaks { samples, num_bins } => {
                if samples.len() < 2 && self.peaks.is_none_or(|p| p.len() < 2) {
                    return self.handle_click(&resp, rect);
                }
                self.draw_peaks(&painter, rect, center_y, samples, *num_bins);
            }
        }

        if let Some(pos) = self.cursor_pos {
            let x = rect.left() + pos.clamp(0.0, 1.0) * rect.width();
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.5, self.color),
            );
        }

        self.handle_click(&resp, rect)
    }

    fn draw_line(&self, painter: &egui::Painter, rect: egui::Rect, center_y: f32, line: &[f32]) {
        let half_h = rect.height() * 0.5;
        let width = rect.width();
        let len = line.len() as f32;

        let x_of = |i: usize| rect.left() + (i as f32 / (len - 1.0)) * width;
        let y_of = |v: f32| center_y - v.clamp(-1.0, 1.0) * half_h;

        // Trace envelope: fading ghost of where the waveform has been
        if let Some(trace) = self.trace {
            let trace_color = self.color.gamma_multiply(self.fill_alpha * 0.6);
            let mut mesh = egui::Mesh::default();
            for i in 0..trace.len() - 1 {
                let x0 = x_of(i);
                let x1 = x_of(i + 1);
                let base = mesh.vertices.len() as u32;
                mesh.colored_vertex(egui::pos2(x0, y_of(trace[i].1)), trace_color);
                mesh.colored_vertex(egui::pos2(x0, y_of(trace[i].0)), trace_color);
                mesh.colored_vertex(egui::pos2(x1, y_of(trace[i + 1].1)), trace_color);
                mesh.colored_vertex(egui::pos2(x1, y_of(trace[i + 1].0)), trace_color);
                mesh.add_triangle(base, base + 1, base + 2);
                mesh.add_triangle(base + 1, base + 2, base + 3);
            }
            painter.add(egui::Shape::mesh(mesh));
        }

        // Fill from waveform to center line
        if self.fill_alpha > 0.01 {
            let fill_color = self.color.gamma_multiply(self.fill_alpha * 0.4);
            let mut mesh = egui::Mesh::default();
            for i in 0..line.len() - 1 {
                let x0 = x_of(i);
                let x1 = x_of(i + 1);
                let base = mesh.vertices.len() as u32;
                mesh.colored_vertex(egui::pos2(x0, y_of(line[i])), fill_color);
                mesh.colored_vertex(egui::pos2(x0, center_y), egui::Color32::TRANSPARENT);
                mesh.colored_vertex(egui::pos2(x1, y_of(line[i + 1])), fill_color);
                mesh.colored_vertex(egui::pos2(x1, center_y), egui::Color32::TRANSPARENT);
                mesh.add_triangle(base, base + 1, base + 2);
                mesh.add_triangle(base + 1, base + 2, base + 3);
            }
            painter.add(egui::Shape::mesh(mesh));
        }

        let points: Vec<egui::Pos2> = line
            .iter()
            .enumerate()
            .map(|(i, &v)| egui::pos2(x_of(i), y_of(v)))
            .collect();
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(self.stroke_width, self.color),
        ));
    }

    fn draw_peaks(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        center_y: f32,
        samples: &[f32],
        num_bins: usize,
    ) {
        let owned_peaks;
        let peaks = if let Some(peaks) = self.peaks {
            peaks
        } else {
            let gain = if self.normalize {
                let peak = samples.iter().fold(0.0_f32, |mx, &s| mx.max(s.abs()));
                if peak > 0.0 { 1.0 / peak } else { 1.0 }
            } else {
                1.0
            };

            let num_bins = num_bins.min(samples.len());
            let len = samples.len() as f32;
            let bins = num_bins as f32;
            owned_peaks = (0..num_bins)
                .map(|i| {
                    let start = (i as f32 * len / bins) as usize;
                    let end = ((i as f32 + 1.0) * len / bins) as usize;
                    samples[start..end]
                        .iter()
                        .fold((f32::MAX, f32::MIN), |(mn, mx), &s| {
                            let s = s * gain;
                            (mn.min(s), mx.max(s))
                        })
                })
                .collect::<Vec<_>>();
            &owned_peaks
        };
        let fill_peaks = self.fill_peaks.unwrap_or(peaks);

        let half_h = rect.height() * 0.5;
        let num_peaks = peaks.len() as f32;
        let width = rect.width();

        let peak_x = |i: usize| rect.left() + (i as f32 / (num_peaks - 1.0)) * width;
        let val_y = |v: f32| center_y - v.clamp(-1.0, 1.0) * half_h;

        let fill_top = self.color.gamma_multiply(self.fill_alpha * 0.8);
        let fill_bottom = self.color.gamma_multiply(self.fill_alpha * 0.3);
        let mut mesh = egui::Mesh::default();
        for i in 0..fill_peaks.len() - 1 {
            let x0 = peak_x(i);
            let x1 = peak_x(i + 1);
            let (min0, max0) = fill_peaks[i];
            let (min1, max1) = fill_peaks[i + 1];

            let base = mesh.vertices.len() as u32;
            mesh.colored_vertex(egui::pos2(x0, val_y(max0)), fill_top);
            mesh.colored_vertex(egui::pos2(x0, val_y(min0)), fill_bottom);
            mesh.colored_vertex(egui::pos2(x1, val_y(max1)), fill_top);
            mesh.colored_vertex(egui::pos2(x1, val_y(min1)), fill_bottom);
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 2, base + 3);
        }
        painter.add(egui::Shape::mesh(mesh));

        let top_line: Vec<egui::Pos2> = peaks
            .iter()
            .enumerate()
            .map(|(i, &(_, max))| egui::pos2(peak_x(i), val_y(max)))
            .collect();
        let bot_line: Vec<egui::Pos2> = peaks
            .iter()
            .enumerate()
            .map(|(i, &(min, _))| egui::pos2(peak_x(i), val_y(min)))
            .collect();

        let stroke = egui::Stroke::new(self.stroke_width, self.color);
        painter.add(egui::Shape::line(top_line, stroke));
        painter.add(egui::Shape::line(bot_line, stroke));
    }

    fn handle_click(&self, resp: &egui::Response, rect: egui::Rect) -> Option<f32> {
        if resp.clicked() {
            resp.interact_pointer_pos()
                .map(|pos| ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0))
        } else {
            None
        }
    }
}
