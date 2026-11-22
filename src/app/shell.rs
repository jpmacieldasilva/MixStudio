//! SetLab-inspired application shell: sidebar, toolbar, inspector chrome, transport.

use super::*;
use eframe::egui::{self, vec2, Margin};

const SIDEBAR_W: f32 = 300.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ShellView {
    Arrange,
    Library,
}

struct InspectorLink {
    tab: Tab,
    label: &'static str,
    hint: &'static str,
}

const INSPECTOR_LINKS: [InspectorLink; 9] = [
    InspectorLink { tab: Tab::Playlist, label: "Playlist", hint: "Track list, order and compatibility" },
    InspectorLink { tab: Tab::Transition, label: "Transition", hint: "Presets, stems and auto-align" },
    InspectorLink { tab: Tab::Track, label: "Track", hint: "Source, stretch and deck settings" },
    InspectorLink { tab: Tab::Grid, label: "Beatgrid", hint: "Beat map and timing" },
    InspectorLink { tab: Tab::Compare, label: "Compare", hint: "Side-by-side waveforms and anchors" },
    InspectorLink { tab: Tab::Effects, label: "Effects", hint: "Effect blocks and XY pad" },
    InspectorLink { tab: Tab::Master, label: "Master", hint: "Tempo, export and output level" },
    InspectorLink { tab: Tab::Samples, label: "Samples", hint: "Sample lanes" },
    InspectorLink { tab: Tab::Zoom, label: "Zoom", hint: "Timeline magnification" },
];

impl App {
  pub(super) fn shell_view(&self) -> ShellView {
    if self.top == 2 { ShellView::Library } else { ShellView::Arrange }
  }

  pub(super) fn shell_frame(&mut self, ctx: &egui::Context) {
    self.shell_sidebar(ctx);
    self.shell_toolbar(ctx);
    self.shell_status(ctx);
    let view = self.shell_view();
    let screen = ctx.screen_rect().height();
    let show_inspector = view == ShellView::Arrange
      || (view == ShellView::Library && self.tab != Tab::Library);
    if show_inspector {
      let bottom_panel = if self.tab == Tab::Compare {
        egui::TopBottomPanel::bottom("inspector")
          .resizable(true)
          .default_height(360.0)
          .height_range(220.0..=screen * 0.72)
      } else if view == ShellView::Library {
        egui::TopBottomPanel::bottom("inspector")
          .resizable(true)
          .default_height(280.0)
          .height_range(180.0..=screen * 0.55)
      } else {
        egui::TopBottomPanel::bottom("inspector")
          .resizable(true)
          .default_height(248.0)
          .height_range(168.0..=screen * 0.62)
      };
      bottom_panel
        .frame(
          egui::Frame::none()
            .fill(PANEL)
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .stroke(Stroke::new(1.0_f32, LINE_SOFT)),
        )
        .show(ctx, |ui| {
          self.shell_inspector_header(ui);
          ui.add_space(4.0);
          ui.separator();
          ui.add_space(4.0);
          self.bottom(ui);
        });
    }
    self.shell_control_bar(ctx);
    egui::SidePanel::right("master_meter")
      .exact_width(44.0)
      .resizable(false)
      .frame(egui::Frame::none().fill(PANEL).stroke(Stroke::new(1.0_f32, LINE_SOFT)))
      .show(ctx, |ui| self.master_meter(ui));
    if view == ShellView::Library && self.tab == Tab::Library {
      egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(BG).inner_margin(Margin::symmetric(12.0, 8.0)))
        .show(ctx, |ui| self.library_tab(ui));
      self.timeline_rect = egui::Rect::NOTHING;
    } else {
      self.timeline_rect = egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(BG_WELL))
        .show(ctx, |ui| self.timeline(ui))
        .response
        .rect;
    }
  }

  fn shell_sidebar(&mut self, ctx: &egui::Context) {
    if !self.sidebar_open {
      return;
    }
    egui::SidePanel::left("setlab_sidebar")
      .exact_width(SIDEBAR_W)
      .resizable(false)
      .frame(
        egui::Frame::none()
          .fill(C::from_rgb(30, 30, 34))
          .inner_margin(Margin::symmetric(8.0, 10.0))
          .stroke(Stroke::new(1.0_f32, LINE_SOFT)),
      )
      .show(ctx, |ui| {
        ui.add_space(4.0);
        self.studio_sidebar_body(ui);
        if self.shell_view() == ShellView::Arrange {
          ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            self.shell_sidebar_status(ui);
          });
        }
      });
  }

  fn shell_sidebar_status(&mut self, ui: &mut egui::Ui) {
    egui::Frame::none()
      .fill(ROW)
      .rounding(6.0)
      .inner_margin(8.0)
      .stroke(Stroke::new(1.0_f32, LINE_SOFT))
      .show(ui, |ui| {
        if self.lib.scanning {
          ui.horizontal(|ui| {
            ui.spinner();
            ui.label(egui::RichText::new("Reading folders…").size(11.0).color(DIM));
          });
        } else if self.lib.analysing.1 > 0 {
          let (done, total) = self.lib.analysing;
          ui.add(egui::ProgressBar::new(done as f32 / total.max(1) as f32).desired_height(6.0));
          ui.label(egui::RichText::new(format!("Analysing BPM/key {done}/{total}")).size(11.0).color(DIM));
        } else if self.loading > 0 {
          ui.horizontal(|ui| {
            ui.spinner();
            ui.label(egui::RichText::new(format!("Analysing {}/…", self.loading)).size(11.0).color(DIM));
          });
        } else {
          let note = if self.status.is_empty() {
            "Ready".to_string()
          } else {
            self.status.clone()
          };
          ui.label(egui::RichText::new(note).size(11.0).color(DIM));
        }
      });
    ui.add_space(8.0);
  }

  pub(super) fn shell_toolbar(&mut self, ctx: &egui::Context) {
    egui::TopBottomPanel::top("toolbar")
      .exact_height(52.0)
      .frame(
        egui::Frame::none()
          .fill(PANEL)
          .inner_margin(Margin::symmetric(12.0, 8.0))
          .stroke(Stroke::new(1.0_f32, LINE_SOFT)),
      )
      .show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
          if ghost_icon_button(ui, if self.sidebar_open { "◧" } else { "☰" }, "Show or hide sidebar").clicked() {
            self.sidebar_open = !self.sidebar_open;
          }
          ui.add_space(8.0);
          ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let dirty = if self.persistence.error.is_some() {
              " ⚠"
            } else if self.persistence.dirty {
              " •"
            } else {
              ""
            };
            ui.label(egui::RichText::new(format!("{}{}", self.mix.name, dirty)).size(15.0).strong().color(TXT));
            ui.label(
              egui::RichText::new(format!(
                "{} tracks · {:.1} BPM · {}",
                self.mix.tracks.len(),
                self.mix.bpm,
                fmt_t(self.mix.length_beats() * 60.0 / self.mix.bpm.max(1.0))
              ))
              .size(11.0)
              .color(MUTED),
            );
          });
          ui.menu_button(egui::RichText::new("Mix").size(13.0), |ui| {
            ui.label(egui::RichText::new("Project").strong());
            ui.horizontal(|ui| {
              ui.label("Name:");
              ui.text_edit_singleline(&mut self.mix.name);
            });
            if ui.button("New mix").clicked() {
              self.new_project();
              ui.close_menu();
            }
            if ui.button("Open mix…").clicked() {
              self.dialog(false, 0);
              ui.close_menu();
            }
            if ui.button("Save").clicked() {
              match self.file.clone() {
                Some(p) => self.save(p),
                None => self.dialog(true, 1),
              }
              ui.close_menu();
            }
            if ui.button("Save as…").clicked() {
              self.dialog(true, 1);
              ui.close_menu();
            }
            ui.separator();
            if ui.button("Recover unsaved projects…").clicked() {
              self.persistence.show_scan = true;
              let _ = self.persistence.worker.tx.send(crate::recovery::Job::Scan);
              ui.close_menu();
            }
            let missing = self.missing_sources().len();
            if ui.button(format!("Locate missing audio… ({missing})")).clicked() {
              self.persistence.relink_window = true;
              ui.close_menu();
            }
            ui.separator();
            ui.label(egui::RichText::new("Edit").strong());
            for m in crate::automix::Mode::ALL {
              ui.radio_value(&mut self.ao_mode, m, m.name());
            }
            for c in crate::automix::Curve::ALL {
              ui.radio_value(&mut self.ao_curve, c, c.name());
            }
            ui.checkbox(&mut self.ao_keep_first, "Keep first track");
            if ui.add_enabled(!self.undo.is_empty(), egui::Button::new("Undo")).clicked() {
              self.do_undo();
              ui.close_menu();
            }
            if ui.add_enabled(!self.redo.is_empty(), egui::Button::new("Redo")).clicked() {
              self.do_redo();
              ui.close_menu();
            }
            if ui.button("Add tracks from disk…").clicked() {
              self.pick_files(None);
              ui.close_menu();
            }
            if ui.button("Remove selected track").clicked() {
              self.remove_sel();
              ui.close_menu();
            }
            if ui.button(format!("Set overlaps to {} bars", self.bars)).clicked() {
              self.snapshot();
              let o = self.mix.main_order();
              self.mix.relayout(&o, self.bars as f64);
              ui.close_menu();
            }
            ui.separator();
            ui.label(egui::RichText::new("Library & settings").strong());
            if ui.button("Manage library & sources…").clicked() {
              self.top = 2;
              self.tab = Tab::Library;
              ui.close_menu();
            }
            if ui.button("Add music folder…").clicked() {
              self.pick_folder();
              ui.close_menu();
            }
            ui.checkbox(&mut self.mix.auto_stems, "Separate stems automatically");
            if ui.button("Separate stems for all tracks").clicked() {
              for i in 0..self.mix.tracks.len() {
                self.request_stems(i);
              }
              ui.close_menu();
            }
          });
          let mid = ui.max_rect().center().x;
          let cur = ui.cursor().min.x;
          ui.add_space((mid - 150.0 - cur).max(12.0));
          egui::Frame::none()
            .fill(CTRL_FILL)
            .rounding(7.0)
            .inner_margin(2.0)
            .stroke(Stroke::new(1.0_f32, LINE_SOFT))
            .show(ui, |ui| {
              ui.horizontal(|ui| {
                let arr = self.shell_view() == ShellView::Arrange;
                let lib = self.shell_view() == ShellView::Library;
                if theme::seg_button(ui, "Arrangement", arr).clicked() {
                  self.top = 3;
                  if self.tab == Tab::Library {
                    self.tab = Tab::Playlist;
                  }
                }
                if theme::seg_button(ui, "Library", lib).on_hover_text("Folders, imports and full table").clicked() {
                  self.top = 2;
                  self.tab = Tab::Library;
                }
              });
            });
          ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
              .add(
                egui::Button::new(egui::RichText::new("Export").size(13.0))
                  .fill(FOCUS)
                  .stroke(Stroke::NONE)
                  .min_size(vec2(78.0, 28.0)),
              )
              .on_hover_text("Export mix (format in Master)")
              .clicked()
            {
              self.dialog(true, 2);
            }
            if ui
              .add_enabled(!self.mix.tracks.is_empty(), egui::Button::new(egui::RichText::new("Auto-order").size(13.0)).min_size(vec2(92.0, 28.0)))
              .on_hover_text("Reorder for harmonic and tempo flow")
              .clicked()
            {
              self.auto_order();
            }
          });
        });
      });
  }

  pub(super) fn shell_status(&mut self, ctx: &egui::Context) {
    let only_loading = self.loading > 0 && self.status.is_empty();
    if only_loading && self.sidebar_open && self.shell_view() == ShellView::Arrange {
      return;
    }
    if self.loading == 0 && self.status.is_empty() {
      return;
    }
    egui::TopBottomPanel::top("flash_status")
      .exact_height(24.0)
      .frame(egui::Frame::none().fill(ROW).inner_margin(Margin::symmetric(12.0, 4.0)))
      .show(ctx, |ui| {
        ui.horizontal(|ui| {
          if self.loading > 0 {
            ui.spinner();
          }
          let text = if self.loading > 0 {
            format!("Analysing {}…", self.loading)
          } else {
            self.status.clone()
          };
          ui.add(egui::Label::new(egui::RichText::new(text).size(11.5).color(DIM)).truncate());
        });
      });
  }

  fn shell_inspector_header(&mut self, ui: &mut egui::Ui) {
    let panel = match self.tab {
      Tab::Playlist => "Playlist",
      Tab::Library => "Library",
      Tab::Transition => "Transition",
      Tab::Track => "Track",
      Tab::Grid => "Beatgrid",
      Tab::Compare => "Compare",
      Tab::Zoom => "Zoom",
      Tab::Effects => "Effects",
      Tab::Master => "Master",
      Tab::Samples => "Samples",
    };
    ui.horizontal(|ui| {
      ui.label(egui::RichText::new("Inspector").size(10.5).color(MUTED).strong());
      ui.label(egui::RichText::new(format!("· {panel}")).size(11.0).color(DIM));
      ui.add_space(4.0);
      if let Some(i) = self.sel.or(self.sel_in).filter(|&i| i < self.mix.tracks.len()) {
        let t = &self.mix.tracks[i];
        ui.label(egui::RichText::new(format!("· {} — {}", t.title, t.key)).size(11.5).color(DIM));
      }
      ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        egui::ScrollArea::horizontal()
          .id_salt("inspector_tabs")
          .max_width(ui.available_width().min(520.0))
          .show(ui, |ui| {
            ui.horizontal(|ui| {
              for link in INSPECTOR_LINKS {
                if link.tab == Tab::Library {
                  continue;
                }
                if theme::seg_button(ui, link.label, self.tab == link.tab)
                  .on_hover_text(link.hint)
                  .clicked()
                {
                  self.tab = link.tab;
                }
              }
            });
          });
      });
    });
  }

  fn shell_control_bar(&mut self, ctx: &egui::Context) {
    if self.shell_view() != ShellView::Arrange {
      return;
    }
    egui::TopBottomPanel::bottom("control_bar")
      .exact_height(60.0)
      .frame(
        egui::Frame::none()
          .fill(PANEL)
          .inner_margin(Margin::symmetric(12.0, 6.0))
          .stroke(Stroke::new(1.0_f32, LINE_SOFT)),
      )
      .show(ctx, |ui| {
        let b = self.beat();
        let bpm = self.mix.bpm;
        let bar = (b / 4.0).floor() as i64 + 1;
        ui.horizontal(|ui| {
          ui.set_max_width(360.0);
          self.control_bar_edit(ui);
          ui.add_space(12.0);
          ui.separator();
          ui.add_space(8.0);
          ui.horizontal(|ui| {
            if ghost_icon_button(ui, "⏮", "Previous track").clicked() {
              let st: Vec<f64> = self
                .mix
                .main_order()
                .iter()
                .map(|&i| self.mix.tracks[i].vis_start(bpm))
                .filter(|&s| s < b - 2.0)
                .collect();
              self.seek(st.last().copied().unwrap_or(0.0));
            }
            let pl = self.playing();
            if ui
              .add(
                egui::Button::new(egui::RichText::new(if pl { "⏸" } else { "▶" }).color(ON_LIVE).size(16.0))
                  .fill(ACCENT)
                  .rounding(18.0)
                  .min_size(vec2(34.0, 34.0)),
              )
              .on_hover_text("Play / pause (Space)")
              .clicked()
            {
              self.toggle_playback();
            }
            if ghost_icon_button(ui, "⏭", "Next track").clicked() {
              if let Some(s) = self
                .mix
                .main_order()
                .iter()
                .map(|&i| self.mix.tracks[i].vis_start(bpm))
                .find(|&s| s > b + 0.5)
              {
                self.seek(s);
              }
            }
          });
          ui.add_space(10.0);
          let now = self
            .sel
            .and_then(|i| self.mix.tracks.get(i))
            .map(|t| t.title.as_str())
            .unwrap_or("—");
          ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(
              egui::RichText::new(format!("{} · bar {bar} · {bpm:.1} BPM", fmt_t(self.sec(b))))
                .size(13.5)
                .color(TXT)
                .strong(),
            );
            ui.label(
              egui::RichText::new(now)
                .size(11.0)
                .color(if self.playing() { ACCENT } else { MUTED }),
            );
          });
          ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let lb = egui::Button::new(egui::RichText::new("Loop").size(11.0).color(if self.loop_on { ACCENT } else { DIM }))
              .fill(if self.loop_on { FOCUS_SOFT } else { C::TRANSPARENT })
              .stroke(if self.loop_on {
                Stroke::new(1.0_f32, FOCUS)
              } else {
                Stroke::NONE
              })
              .min_size(vec2(46.0, 26.0));
            if ui.add(lb).on_hover_text("Loop selection or bar length").clicked() {
              self.stop_comparison();
              self.stop_transition_audition();
              self.loop_on = !self.loop_on;
              let s = (b / 4.0).floor() * 4.0;
              self.set_loop(if self.loop_on {
                Some(self.section_range.map(|(_, a, b)| (a, b)).unwrap_or((s, s + self.bars as f64 * 4.0)))
              } else {
                None
              });
            }
            egui::ComboBox::from_id_salt("overlap_bars")
              .width(52.0)
              .selected_text(format!("{}b", self.bars))
              .show_ui(ui, |ui| {
                for bars in [4u32, 8, 16, 32] {
                  ui.selectable_value(&mut self.bars, bars, format!("{bars} beats overlap"));
                }
              })
              .response
              .on_hover_text("Default transition overlap");
            let mo = self.mix.metronome;
            let met = egui::Button::new(egui::RichText::new("Click").size(11.0).color(if mo { ON_LIVE } else { DIM }))
              .fill(if mo { ACCENT.gamma_multiply(0.85) } else { CTRL_FILL })
              .min_size(vec2(52.0, 26.0));
            if ui.add(met).on_hover_text("Metronome").clicked() {
              self.mix.metronome = !mo;
            }
            if ghost_icon_button(ui, if self.lock { "🔒" } else { "🔓" }, "Lock arrangement").clicked() {
              self.lock = !self.lock;
            }
            if ghost_icon_button(ui, if self.follow { "◎" } else { "○" }, "Follow playhead while playing").clicked() {
              self.follow = !self.follow;
            }
            let snap = match self.snapping {
              Snapping::Off => "Off",
              Snapping::Free => "Beat",
              Snapping::Half => "½",
              Snapping::Full => "Bar",
            };
            egui::ComboBox::from_id_salt("snap_shell")
              .width(58.0)
              .selected_text(snap)
              .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.snapping, Snapping::Off, "Snap off");
                ui.selectable_value(&mut self.snapping, Snapping::Free, "Beat");
                ui.selectable_value(&mut self.snapping, Snapping::Half, "Half bar");
                ui.selectable_value(&mut self.snapping, Snapping::Full, "Bar");
              });
          });
        });
      });
  }
}

fn ghost_icon_button(ui: &mut egui::Ui, icon: &str, tip: &str) -> egui::Response {
  ui.add(
    egui::Button::new(egui::RichText::new(icon).size(15.0).color(DIM))
      .frame(false)
      .min_size(vec2(28.0, 28.0)),
  )
  .on_hover_text(tip)
}