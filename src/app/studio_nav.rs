//! Painel lateral estilo SetLab StudioSidePanel — faixas arrastáveis sem abrir Library full.

use super::*;
use eframe::egui::{self, pos2, vec2, Align2, FontId, Rect, Sense, Stroke};

const SIDE_ROW_H: f32 = 36.0;

impl App {
    pub(super) fn studio_sidebar_body(&mut self, ui: &mut egui::Ui) {
        if self.shell_view() == super::shell::ShellView::Library {
            self.studio_sidebar_library_mode(ui);
            return;
        }
        self.studio_sidebar_arrange(ui);
    }

    fn studio_sidebar_library_mode(&mut self, ui: &mut egui::Ui) {
        if ui.add(egui::Button::new(egui::RichText::new("← Arrangement").size(12.0))).clicked() {
            self.top = 3;
            self.tab = Tab::Playlist;
        }
        ui.add_space(8.0);
        sidebar_caption(ui, "Full library");
        ui.label(egui::RichText::new(format!("{} indexed tracks", self.lib.index.entries.len())).size(11.0).color(DIM));
        ui.add_space(6.0);
        if ui.button("Add music folder…").clicked() {
            self.pick_folder();
        }
        ui.label(egui::RichText::new("Imports, filters and table stay here.").size(11.0).color(MUTED));
    }

    fn studio_sidebar_arrange(&mut self, ui: &mut egui::Ui) {
        let tab = self.library_ui.side_tab;
        egui::Frame::none()
            .fill(CTRL_FILL)
            .rounding(6.0)
            .inner_margin(2.0)
            .stroke(Stroke::new(1.0_f32, LINE_SOFT))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if theme::seg_button(ui, "Tracks", tab == 0).clicked() {
                        self.library_ui.side_tab = 0;
                    }
                    if theme::seg_button(ui, "Mix", tab == 1).clicked() {
                        self.library_ui.side_tab = 1;
                    }
                });
            });
        ui.add_space(6.0);
        if tab == 0 {
            self.studio_track_picker(ui);
        } else {
            self.studio_mix_nav(ui);
        }
    }

    fn studio_track_picker(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("⌕").color(MUTED));
            let clear = !self.lib_q.is_empty();
            ui.add(
                egui::TextEdit::singleline(&mut self.lib_q)
                    .hint_text("Search title, artist, key…")
                    .desired_width(ui.available_width() - if clear { 22.0 } else { 0.0 }),
            );
            if clear
                && ui
                    .add(egui::Button::new("×").frame(false).small())
                    .on_hover_text("Clear")
                    .clicked()
            {
                self.lib_q.clear();
            }
        });
        ui.horizontal(|ui| {
            toggle_button(ui, &mut self.lib_key, "Harmonic");
            toggle_button(ui, &mut self.lib_bpm, "BPM");
            if self.lib_bpm {
                ui.add(egui::DragValue::new(&mut self.lib_pct).range(1.0..=30.0).speed(0.2).prefix("±").suffix("%"));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("+ Folder").clicked() {
                    self.pick_folder();
                }
            });
        });
        self.refresh_library_view();
        ui.add_space(4.0);
        let avail = ui.available_height() - 52.0;
        let mut listen = None;
        let mut insert = None;
        egui::ScrollArea::vertical()
            .id_salt("studio_nav_tracks")
            .max_height(avail.max(120.0))
            .show(ui, |ui| {
                if self.lib_view.is_empty() {
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new(if self.lib.cfg.folders.is_empty() {
                            "Add a music folder, then drag tracks onto the timeline."
                        } else {
                            "No matches — clear search or turn off Harmonic / BPM filters."
                        })
                        .size(11.5)
                        .color(DIM),
                    );
                    return;
                }
                let rows: Vec<(usize, String, String, String, Option<f64>, Option<String>)> = self
                    .lib_view
                    .iter()
                    .filter_map(|&idx| {
                        self.library_entries().get(idx).map(|e| {
                            (
                                idx,
                                e.path.clone(),
                                e.filename().to_string(),
                                e.artist.clone(),
                                e.bpm,
                                e.key.clone(),
                            )
                        })
                    })
                    .collect();
                for (idx, path, title, artist, bpm, key) in rows {
                    let _ = idx;
                    if self.studio_nav_row(ui, &path, &title, &artist, bpm, key.as_deref(), &mut listen, &mut insert) {
                        break;
                    }
                }
            });
        if let Some((path, title)) = listen {
            self.toggle_listen(path, title, ui.ctx());
        }
        if let Some(path) = insert {
            self.lib_add(vec![path]);
        }
        self.studio_nav_footer(ui);
    }

    fn studio_nav_row(
        &mut self,
        ui: &mut egui::Ui,
        path: &str,
        title: &str,
        artist: &str,
        bpm: Option<f64>,
        key: Option<&str>,
        listen: &mut Option<(String, String)>,
        insert: &mut Option<String>,
    ) -> bool {
        let in_mix = self.mix.tracks.iter().any(|t| t.path == path);
        let preview = self.listening(path);
        let selected = self.lib_sel.contains(path);
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), SIDE_ROW_H), Sense::click_and_drag());
        let p = ui.painter();
        if selected || response.hovered() {
            p.rect_filled(rect, 6.0, if selected { FOCUS_SOFT } else { CTRL_HOVER });
        }
        let key_lbl = key.unwrap_or("—");
        chip_at(p, pos2(rect.left() + 4.0, rect.center().y), key_lbl, key_col(key_lbl));
        let title_rect = Rect::from_min_max(pos2(rect.left() + 52.0, rect.top() + 4.0), pos2(rect.right() - 88.0, rect.center().y));
        p.text(title_rect.left_top(), Align2::LEFT_TOP, title, FontId::proportional(12.0), if in_mix { MUTED } else { TXT });
        if !artist.is_empty() {
            p.text(
                pos2(title_rect.left(), rect.center().y + 2.0),
                Align2::LEFT_TOP,
                artist,
                FontId::proportional(10.5),
                DIM,
            );
        }
        let bpm_lbl = bpm.map(|b| format!("{b:.0}")).unwrap_or_else(|| "—".into());
        p.text(
            pos2(rect.right() - 8.0, rect.center().y),
            Align2::RIGHT_CENTER,
            bpm_lbl,
            FontId::proportional(11.0),
            MUTED,
        );
        if response.hovered() || preview {
            let play_r = Rect::from_center_size(pos2(rect.right() - 28.0, rect.center().y), vec2(22.0, 22.0));
            let add_r = Rect::from_center_size(pos2(rect.right() - 52.0, rect.center().y), vec2(22.0, 22.0));
            if ui.interact(add_r, ui.id().with(("add", path)), Sense::click()).clicked() {
                *insert = Some(path.to_string());
                return true;
            }
            if ui
                .interact(play_r, ui.id().with(("prev", path)), Sense::click())
                .on_hover_text(if preview { "Stop preview" } else { "Preview" })
                .clicked()
            {
                if preview {
                    self.stop_listen();
                } else {
                    *listen = Some((path.to_string(), title.to_string()));
                }
            }
            p.circle_stroke(add_r.center(), 10.0, Stroke::new(1.0_f32, LINE));
            p.text(add_r.center(), Align2::CENTER_CENTER, "+", FontId::proportional(14.0), TXT);
        }
        if response.drag_started() && !self.arrangement_locked() {
            response.dnd_set_drag_payload(LibDrag(vec![path.to_string()]));
        }
        if response.clicked() {
            if ui.input(|i| i.modifiers.command) {
                if selected {
                    self.lib_sel.remove(path);
                } else {
                    self.lib_sel.insert(path.to_string());
                }
            } else {
                self.lib_sel.clear();
                self.lib_sel.insert(path.to_string());
            }
        }
        if response.double_clicked() && !self.arrangement_locked() {
            *insert = Some(path.to_string());
            return true;
        }
        false
    }

    fn studio_mix_nav(&mut self, ui: &mut egui::Ui) {
        let order = self.mix.main_order();
        if order.is_empty() {
            ui.label(egui::RichText::new("Empty mix — drag tracks from Tracks tab.").size(11.5).color(DIM));
            return;
        }
        egui::ScrollArea::vertical().id_salt("studio_mix_rows").show(ui, |ui| {
            for (k, &ti) in order.iter().enumerate() {
                let t = &self.mix.tracks[ti];
                let sel = self.sel == Some(ti);
                let playing = self.playing() && self.sel == Some(ti);
                let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), SIDE_ROW_H), Sense::click());
                if response.clicked() {
                    self.sel = Some(ti);
                    self.sel_in = None;
                    self.tab = Tab::Track;
                }
                let p = ui.painter();
                if sel || response.hovered() {
                    p.rect_filled(rect, 6.0, if playing { ACCENT.gamma_multiply(0.2) } else if sel { FOCUS_SOFT } else { CTRL_HOVER });
                }
                chip_at(p, pos2(rect.left() + 4.0, rect.center().y), &t.key, key_col(&t.key));
                p.text(
                    pos2(rect.left() + 50.0, rect.top() + 5.0),
                    Align2::LEFT_TOP,
                    &t.title,
                    FontId::proportional(12.0),
                    if playing { ACCENT } else { TXT },
                );
                p.text(
                    pos2(rect.left() + 50.0, rect.bottom() - 14.0),
                    Align2::LEFT_BOTTOM,
                    format!("Deck {} · bar {:.0}", t.lane + 1, t.start_beat / 4.0 + 1.0),
                    FontId::proportional(10.0),
                    DIM,
                );
                p.text(
                    pos2(rect.right() - 6.0, rect.center().y),
                    Align2::RIGHT_CENTER,
                    format!("{}.", k + 1),
                    FontId::proportional(11.0),
                    MUTED,
                );
            }
        });
    }

    fn studio_nav_footer(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        egui::Frame::none()
            .fill(ROW)
            .rounding(6.0)
            .inner_margin(8.0)
            .stroke(Stroke::new(1.0_f32, LINE_SOFT))
            .show(ui, |ui| {
                let n = self.lib_view.len();
                let compatible = if self.lib_key { self.lib_view.len() } else { n };
                ui.label(
                    egui::RichText::new(format!("{n} shown · {compatible} for mix"))
                        .size(11.0)
                        .color(DIM),
                );
                if self.lib.scanning {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Scanning…");
                    });
                } else if self.lib.analysing.1 > 0 {
                    let (d, t) = self.lib.analysing;
                    ui.label(egui::RichText::new(format!("Analysing {d}/{t}")).size(11.0).color(DIM));
                }
            });
    }
}

fn sidebar_caption(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text.to_uppercase()).size(10.5).color(MUTED).strong());
}

fn chip_at(p: &egui::Painter, center: egui::Pos2, text: &str, col: C) {
    let galley = p.layout_no_wrap(text.to_string(), FontId::proportional(11.0), col);
    let size = galley.size() + vec2(10.0, 4.0);
    let rect = Rect::from_center_size(center, size);
    p.rect_filled(rect, 4.0, col.gamma_multiply(0.18));
    p.galley(rect.min + vec2(5.0, 2.0), galley, col);
}