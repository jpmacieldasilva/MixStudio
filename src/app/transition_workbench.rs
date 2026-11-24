//! Processed transition audition: temporary snapshot filtering, never project edits.
use super::*;

pub(super) struct Audition {
    incoming: usize,
    side: u8,
    from: f64,
    to: f64,
    repeat: bool,
    stamp: Vec<u64>,
    restore_loop: Option<(f64, f64)>,
    restore_loop_on: bool,
}
impl App {
    pub(super) fn arrangement_locked(&self) -> bool {
        self.lock || self.mix.has_transition_locks()
    }
    pub(super) fn set_transition_protection(&mut self, i: usize, locked: bool) {
        if self.mix.tracks.get(i).is_none_or(|t| t.trans.locked == locked) {
            return;
        }
        self.snapshot();
        self.mix.tracks[i].trans.locked = locked;
        self.mix.sync_transition(i);
        self.drag = None;
        self.tl_drag = None;
        self.grid_drag = None;
        self.bm_drag = None;
        self.tc_drag = None;
        self.status = if locked {
            "Transition protected. Unlock protected transitions to move, cut or reorder the arrangement.".into()
        } else {
            "Transition unlocked. Undo: Ctrl+Z.".into()
        };
    }
    pub(super) fn transition_source_ranges(&self, incoming: usize, side: usize) -> Vec<(f64, f64, f64, f64)> {
        let Some((_, _, s, e)) = self.mix.logical_transitions().into_iter().find(|x| x.1 == incoming) else {
            return vec![];
        };
        let Some(members) = self.transition_members(incoming) else {
            return vec![];
        };
        let mut ranges = vec![];
        for &i in &members[side.min(1)] {
            let t = &self.mix.tracks[i];
            let g = t.grid();
            let lo = s.max(t.vis_start(0.0));
            let hi = e.min(t.end(0.0));
            if hi <= lo {
                continue;
            }
            let mut points = vec![lo];
            points.extend(g.cuts().filter(|b| *b > lo && *b < hi));
            points.push(hi);
            for w in points.windows(2) {
                let at = (w[0] + w[1]) * 0.5;
                ranges.push((w[0], w[1], g.region_sec(w[0], at), g.region_sec(w[1], at)));
            }
        }
        ranges.sort_by(|a, b| a.0.total_cmp(&b.0));
        ranges
    }
    pub(super) fn start_transition_audition(&mut self, i: usize, side: u8, repeat: bool) {
        let Some((_, _, s, e)) = self.mix.logical_transitions().into_iter().find(|x| x.1 == i) else {
            return;
        };
        let Some(stamp) = self.alignment_stamp(i) else {
            self.status = "Load both source tracks before listening.".into();
            return;
        };
        if self
            .transition_members(i)
            .is_none_or(|g| g.iter().flatten().any(|k| !self.store.contains_key(&self.mix.tracks[*k].path)))
        {
            self.status = "Load every source fragment before listening.".into();
            return;
        }
        self.stop_listen();
        self.stop_comparison();
        self.stop_transition_audition();
        let from = (s - self.transition_pre as f64 * 4.0).max(0.0);
        let to = (e + self.transition_post as f64 * 4.0).min(self.mix.length_beats());
        if to <= from {
            return;
        }
        self.transition_audition = Some(Audition {
            incoming: i,
            side: side.clamp(1, 3),
            from,
            to,
            repeat,
            stamp,
            restore_loop: self.active_loop,
            restore_loop_on: self.loop_on,
        });
        self.loop_on = repeat;
        self.set_loop(if repeat { Some((from, to)) } else { None });
        let _ = self.out.snapshot(Arc::new(self.build_playback_snap()));
        self.seek(from);
        let _ = self.out.tx.send(Cmd::Play(true));
    }
    pub(super) fn switch_transition_side(&mut self, side: u8) {
        if let Some(a) = &mut self.transition_audition {
            a.side = side.clamp(1, 3);
            let _ = self.out.snapshot(Arc::new(self.build_playback_snap()));
        }
    }
    pub(super) fn stop_transition_audition(&mut self) {
        if let Some(a) = self.transition_audition.take() {
            let _ = self.out.tx.send(Cmd::Play(false));
            self.loop_on = a.restore_loop_on;
            self.set_loop(a.restore_loop);
            let _ = self.out.snapshot(Arc::new(self.build_snap()));
        }
    }
    pub(super) fn validate_transition_audition(&mut self) {
        if let Some(a) = &self.transition_audition {
            if self.tab != Tab::Transition
                || self.sel_in != Some(a.incoming)
                || self.alignment_stamp(a.incoming).as_ref() != Some(&a.stamp)
                || (!a.repeat && self.beat() >= a.to)
                || self.beat() < a.from - 0.001
            {
                self.stop_transition_audition();
            }
        }
    }
    pub(super) fn filter_transition_audition(&self, snap: &mut Snap) {
        let Some(a) = &self.transition_audition else { return; };
        if self.tab != Tab::Transition || self.alignment_stamp(a.incoming).as_ref() != Some(&a.stamp) {
            return;
        }
        let Some(members) = self.transition_members(a.incoming) else { return; };
        let mut loaded = 0;
        for (i, t) in self.mix.tracks.iter().enumerate() {
            if !self.store.contains_key(&t.path) {
                continue;
            }
            if !members.iter().flatten().any(|k| *k == i) {
                snap.tracks[loaded].vis = snap.tracks[loaded].end;
            }
            loaded += 1;
        }
        for d in 0..4 {
            let allowed = members.iter().enumerate().any(|(side, group)| {
                (a.side == 3 || a.side == side as u8 + 1) && group.iter().any(|i| self.mix.tracks[*i].lane == d)
            });
            snap.decks[d].mute = !allowed;
            snap.decks[d].solo = false;
        }
    }
    fn transition_header(
        &mut self,
        ui: &mut egui::Ui,
        choice: usize,
        transitions: &[(usize, usize, f64, f64)],
        o: usize,
        i: usize,
        s: f64,
        e: f64,
    ) {
        let ot_key = self.mix.tracks[o].key.clone();
        let ot_title = self.mix.tracks[o].title.clone();
        let ot_bpm = self.mix.tracks[o].bpm;
        let it_key = self.mix.tracks[i].key.clone();
        let it_title = self.mix.tracks[i].title.clone();
        let it_bpm = self.mix.tracks[i].bpm;
        let (sc, col, lab) = crate::automix::rate(&self.ao_item(o), &self.ao_item(i));
        let pct = (it_bpm / ot_bpm - 1.0) * 100.0;
        let bar_lo = (s / 4.0).floor() + 1.0;
        let bar_hi = (e / 4.0).ceil();
        let dur_s = self.sec(e) - self.sec(s);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("Transition {} of {}", choice + 1, transitions.len())).size(13.0).strong());
            if ui.add_enabled(choice > 0, egui::Button::new("‹").small()).clicked() {
                self.sel_in = Some(transitions[choice - 1].1);
            }
            if ui
                .add_enabled(choice + 1 < transitions.len(), egui::Button::new("›").small())
                .clicked()
            {
                self.sel_in = Some(transitions[choice + 1].1);
            }
            ui.add_space(6.0);
            chip(ui, ot_key.clone(), key_col(&ot_key));
            ui.label(egui::RichText::new(&ot_title).size(12.0).strong());
            ui.label(egui::RichText::new("->").color(DIM));
            chip(ui, it_key.clone(), key_col(&it_key));
            ui.label(egui::RichText::new(&it_title).size(12.0).strong());
            let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 3.5, compat_col(col));
            ui.label(
                egui::RichText::new(format!(
                    "{lab} {:+.1}% · Bars {:.0}–{:.0} · {}",
                    pct,
                    bar_lo,
                    bar_hi,
                    fmt_t(dur_s.max(0.0))
                ))
                .size(11.5)
                .color(DIM),
            );
            let _ = sc;
            ui.add_space(8.0);
            let locked = self.mix.tracks[i].trans.locked;
            if ui
                .add(
                    egui::Button::new(egui::RichText::new(if locked { "🔒" } else { "🔓" }).size(13.0))
                        .fill(C::TRANSPARENT)
                        .frame(false),
                )
                .on_hover_text("Lock transition from arrangement edits")
                .clicked()
            {
                self.set_transition_protection(i, !locked);
            }
            for (side, label) in [(1, "A"), (3, "Both"), (2, "B")] {
                let on = self.transition_audition.as_ref().is_some_and(|a| a.incoming == i && a.side == side);
                if theme::seg_button(ui, label, on)
                    .on_hover_text("Monitor processed transition on this side")
                    .clicked()
                {
                    if self.transition_audition.is_some() {
                        self.switch_transition_side(side);
                    } else {
                        self.start_transition_audition(i, side, self.transition_repeat);
                    }
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Play Transition").color(TXT))
                            .fill(FOCUS)
                            .min_size(egui::vec2(128.0, 28.0)),
                    )
                    .on_hover_text("Audition the processed overlap")
                    .clicked()
                {
                    self.start_transition_audition(i, 3, self.transition_repeat);
                }
                if self.transition_audition.is_some() && ui.button("Stop").clicked() {
                    self.stop_transition_audition();
                }
                if ui
                    .add(egui::Button::new(egui::RichText::new("Auto-align").size(11.0)).frame(false))
                    .on_hover_text("Measure and align incoming grid in the overlap")
                    .clicked()
                {
                    self.request_beat_check(i, ui.ctx(), true);
                }
            });
        });
        ui.add_space(4.0);
    }
    pub(super) fn transition_tab(&mut self, ui: &mut egui::Ui) {
        let transitions = self.mix.logical_transitions();
        if transitions.is_empty() {
            ui.label("No transition yet. Add two overlapping tracks.");
            if self.mix.has_transition_locks() && ui.button("Unlock all transitions").clicked() {
                self.snapshot();
                for t in &mut self.mix.tracks {
                    t.trans.locked = false;
                }
            }
            return;
        }
        let choice = self.sel_in.and_then(|i| transitions.iter().position(|x| x.1 == i)).unwrap_or(0);
        let (o, i, s, e) = transitions[choice];
        if self.transition_audition.as_ref().is_some_and(|a| a.incoming != i) {
            self.stop_transition_audition();
        }
        self.sel_in = Some(i);
        self.transition_header(ui, choice, &transitions, o, i, s, e);
        let editable = !self.lock && !self.mix.tracks[i].trans.locked;
        ui.columns(4, |cols| {
            cols[0].vertical(|ui| {
                ui.label(egui::RichText::new("PRESET").size(10.0).color(MUTED).strong());
                ui.add_space(2.0);
                ui.add_enabled_ui(editable, |ui| {
                    let preset = self.mix.tracks[i].trans.preset;
                    for p in Preset::ALL {
                        if ui.selectable_label(preset == p, p.name()).clicked() && preset != p {
                            self.snapshot();
                            let t = &mut self.mix.tracks[i].trans;
                            t.preset = p;
                            t.stems = match p {
                                Preset::BassSwap => [StemMode::Crossfade, StemMode::Swap, StemMode::Crossfade, StemMode::Crossfade],
                                Preset::EchoOut => [StemMode::Swap; 4],
                                _ => [StemMode::Crossfade; 4],
                            };
                        }
                    }
                    ui.add_space(8.0);
                    if ui.link("Reset Curves").on_hover_text("Clear custom volume curves for this transition").clicked() {
                        self.snapshot();
                        let t = &mut self.mix.tracks[i].trans;
                        t.curve_out.clear();
                        t.curve_in.clear();
                    }
                    if ui.link("Volume curves…").on_hover_text("Edit outgoing/incoming volume automation").clicked() {
                        ui.data_mut(|d| d.insert_temp(egui::Id::new("trans_curves_open"), true));
                    }
                });
            });
            cols[1].vertical(|ui| {
                ui.label(egui::RichText::new("STEMS").size(10.0).color(MUTED).strong());
                ui.add_space(2.0);
                ui.add_enabled_ui(editable, |ui| {
                    let mut stems = self.mix.tracks[i].trans.stems;
                    for k in 0..4 {
                        ui.horizontal(|ui| {
                            ui.set_width(58.0);
                            ui.label(egui::RichText::new(STEM_N[k]).size(11.0).color(DIM));
                            for mode in StemMode::ALL {
                                if theme::seg_button(ui, mode.name(), stems[k] == mode).clicked() && stems[k] != mode {
                                    self.snapshot();
                                    stems[k] = mode;
                                }
                            }
                        });
                        ui.add_space(2.0);
                    }
                    if stems != self.mix.tracks[i].trans.stems {
                        self.mix.tracks[i].trans.stems = stems;
                    }
                });
                let stems_ready = [o, i].iter().all(|ti| self.stems.contains_key(&self.mix.tracks[*ti].path));
                ui.label(
                    egui::RichText::new(if stems_ready {
                        "Ready"
                    } else {
                        "Separate stems to edit lanes"
                    })
                    .size(11.0)
                    .color(if stems_ready { OK } else { DIM }),
                );
            });
            cols[2].vertical(|ui| {
                ui.label(egui::RichText::new("OVERLAP").size(10.0).color(MUTED).strong());
                ui.add_space(2.0);
                let bars = ((e - s) / 4.0).round() as u32;
                ui.add_enabled_ui(!self.arrangement_locked(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Length").size(10.5).color(DIM));
                        egui::ComboBox::from_id_salt("trans_overlap_len")
                            .width(88.0)
                            .selected_text(format!("{bars} bars"))
                            .show_ui(ui, |ui| {
                                for b in [4u32, 8, 16, 32] {
                                    if ui.selectable_label(bars == b, format!("{b} bars")).clicked() {
                                        self.snapshot();
                                        self.mix.shift_logical_transition(i, (e - s) - b as f64 * 4.0, true);
                                    }
                                }
                            });
                    });
                });
                let mut ratio = self.mix.tracks[i].trans.ratio;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Beat ratio").size(10.5).color(DIM));
                    egui::ComboBox::from_id_salt("beat_ratio")
                        .width(ui.available_width())
                        .selected_text(crate::model::RATIOS[ratio as usize].0)
                        .show_ui(ui, |ui| {
                            for (k, (name, _)) in crate::model::RATIOS.iter().enumerate() {
                                ui.selectable_value(&mut ratio, k as u8, *name);
                            }
                        });
                });
                if ratio != self.mix.tracks[i].trans.ratio {
                    self.snapshot();
                    self.mix.tracks[i].trans.ratio = ratio;
                    self.mix.sync_transition(i);
                    self.mix.resolve_ratios();
                }
                self.song_gain_control(ui, o, "Gain A");
                self.song_gain_control(ui, i, "Gain B");
                let mut prot = self.mix.tracks[i].trans.locked;
                if ui
                    .checkbox(&mut prot, "Protect")
                    .on_hover_text("Lock transition from arrangement edits")
                    .changed()
                {
                    self.set_transition_protection(i, prot);
                }
            });
            cols[3].vertical(|ui| {
                ui.label(egui::RichText::new("BEATMATCH").size(10.0).color(MUTED).strong());
                ui.add_space(2.0);
                self.beatmatch_ui(ui, false);
                ui.horizontal(|ui| {
                    if ui.link("Compare").clicked() {
                        self.open_comparison(i);
                    }
                    if ui.link("Beatgrid B").clicked() {
                        self.sel = Some(i);
                        self.tab = Tab::Grid;
                    }
                    if ui.link("Zoom to Overlap").clicked() {
                        self.scroll = s - 4.0;
                        self.ppb = (self.timeline_rect.width() / (e - s + 8.0) as f32).clamp(2.0, 32.0);
                        self.follow = false;
                        self.seek((s + e) * 0.5);
                    }
                });
            });
        });
        ui.add_space(6.0);
        self.harmony_ui(ui, o, i);
        self.mix.sync_transition(i);
        let show_curves = ui.data_mut(|d| d.get_temp::<bool>(egui::Id::new("trans_curves_open")).unwrap_or(false));
        if show_curves {
            ui.add_enabled_ui(editable, |ui| {
                if ui.small_button("Close curves").clicked() {
                    ui.data_mut(|d| d.insert_temp(egui::Id::new("trans_curves_open"), false));
                }
                self.trans_curves_ui(ui, i);
            });
        }
    }
}