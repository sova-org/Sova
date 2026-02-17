use eframe::egui;

pub struct Waveform<'a> {
    samples: &'a [f32],
    color: egui::Color32,
    fill_alpha: f32,
    stroke_width: f32,
    normalize: bool,
    num_bins: usize,
    cursor_pos: Option<f32>,
}

impl<'a> Waveform<'a> {
    pub fn new(samples: &'a [f32], color: egui::Color32) -> Self {
        Self {
            samples,
            color,
            fill_alpha: 0.35,
            stroke_width: 1.0,
            normalize: false,
            num_bins: 256,
            cursor_pos: None,
        }
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
        self.num_bins = n;
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) -> Option<f32> {
        let avail = ui.available_size();
        let desired = egui::vec2(avail.x, avail.y.max(60.0));
        let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
        let painter = ui.painter_at(rect);

        let center_y = rect.center().y;
        painter.line_segment(
            [
                egui::pos2(rect.left(), center_y),
                egui::pos2(rect.right(), center_y),
            ],
            egui::Stroke::new(0.5, self.color.gamma_multiply(0.2)),
        );

        if self.samples.len() < 2 {
            return self.handle_click(&resp, rect);
        }

        let gain = if self.normalize {
            let peak = self.samples.iter().fold(0.0_f32, |mx, &s| mx.max(s.abs()));
            if peak > 0.0 { 1.0 / peak } else { 1.0 }
        } else {
            1.0
        };

        let num_bins = self.num_bins.min(self.samples.len());
        let bin_size = self.samples.len() / num_bins;
        let peaks: Vec<(f32, f32)> = (0..num_bins)
            .map(|i| {
                let start = i * bin_size;
                let end = (start + bin_size).min(self.samples.len());
                self.samples[start..end]
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(mn, mx), &s| {
                        let s = s * gain;
                        (mn.min(s), mx.max(s))
                    })
            })
            .collect();

        let half_h = rect.height() * 0.5;
        let num_peaks = peaks.len() as f32;
        let width = rect.width();

        let peak_x = |i: usize| rect.left() + (i as f32 / (num_peaks - 1.0)) * width;
        let val_y = |v: f32| center_y - v.clamp(-1.0, 1.0) * half_h;

        let fill_color = self.color.gamma_multiply(self.fill_alpha);
        let mut mesh = egui::Mesh::default();
        for i in 0..peaks.len() - 1 {
            let x0 = peak_x(i);
            let x1 = peak_x(i + 1);
            let (min0, max0) = peaks[i];
            let (min1, max1) = peaks[i + 1];

            let base = mesh.vertices.len() as u32;
            mesh.colored_vertex(egui::pos2(x0, val_y(max0)), fill_color);
            mesh.colored_vertex(egui::pos2(x0, val_y(min0)), fill_color);
            mesh.colored_vertex(egui::pos2(x1, val_y(max1)), fill_color);
            mesh.colored_vertex(egui::pos2(x1, val_y(min1)), fill_color);
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 2, base + 3);
        }
        painter.add(egui::Shape::mesh(mesh));

        let stroke = egui::Stroke::new(self.stroke_width, self.color);
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
        painter.add(egui::Shape::line(top_line, stroke));
        painter.add(egui::Shape::line(bot_line, stroke));

        if let Some(pos) = self.cursor_pos {
            let x = rect.left() + pos.clamp(0.0, 1.0) * rect.width();
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.5, self.color),
            );
        }

        self.handle_click(&resp, rect)
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
