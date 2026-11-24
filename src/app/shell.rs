//! SetLab-inspired application shell: sidebar, toolbar, inspector chrome, transport.

use super::*;
use eframe::egui::{self, vec2, Margin};

const SIDEBAR_W: f32 = 280.0;

fn autosave_line(p: &persistence::Persistence) -> String {
    if !p.enabled {
        "Autosave disabled".into()
    } else if !p.dirty {
        "No unsaved changes".into()
    } else if p.pending_write {
        "Autosaving…".into()
    } else if let Some(at) = p.last_good {
        format!(
            "Recovery copy: {} seconds ago",
            crate::recovery::now().saturating_sub(at)
        )
    } else {
        "Autosave: checks every second, writes at most every 15 seconds".into()
    }
}

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

  fn shell_editor_tab(&self) -> Option<Tab> {
    if self.top == 2 {
      return None;
    }
    if self.tab == Tab::Compare && self.comparison.incoming.is_some() {
      return Some(Tab::Compare);
    }
    if self.fx_sel.is_some() {
      return Some(Tab::Effects);
    }
    if self.sel.is_some() {
      return Some(Tab::Track);
    }
    if self.sel_in.is_some() {
      return Some(Tab::Transition);
    }
    None
  }

  fn shell_frame_pin_demo(&mut self) {
    let Ok(tab) = std::env::var("MIXSTUDIO_DEMO_TAB") else { return };
    match tab.as_str() {
      "library" => {
        self.top = 2;
        self.tab = Tab::Library;
        self.sel = None;
        self.sel_in = None;
      }
      "track" | "arrangement" => {
        self.top = 3;
        self.library_ui.side_tab = 1;
        if self.mix.tracks.len() > 1 {
          self.sel = Some(1);
        }
        self.sel_in = None;
        self.tab = Tab::Track;
      }
      "transition" => {
        self.top = 3;
        self.library_ui.side_tab = 1;
        self.sel = None;
        if let Some(&(_, i, _, _)) = self.mix.logical_transitions().first() {
          self.sel_in = Some(i);
          self.tab = Tab::Transition;
        }
      }
      "autoorder" => {
        self.top = 3;
        self.library_ui.side_tab = 1;
        self.tab = Tab::Playlist;
        self.library_ui.capture_autoorder_menu = true;
      }
      _ => {}
    }
  }

  pub(super) fn shell_frame(&mut self, ctx: &egui::Context) {
    self.shell_sidebar(ctx);
    self.shell_toolbar(ctx);
    self.shell_status(ctx);
    self.shell_settings_window(ctx);
    self.shell_shortcuts_window(ctx);
    self.shell_frame_pin_demo();
    self.library_import_window(ctx);
    let view = self.shell_view();
    let screen = ctx.screen_rect().height();
    if self.top != 2 {
      if let Some(t) = self.shell_editor_tab() {
        self.tab = t;
      }
    }
    self.shell_transport_bar(ctx);
    let show_inspector = match view {
      ShellView::Library => self.tab != Tab::Library,
      ShellView::Arrange => self.shell_editor_tab().is_some(),
    };
    if show_inspector {
      let bottom_panel = if view == ShellView::Library {
        egui::TopBottomPanel::bottom("inspector")
          .resizable(true)
          .default_height(280.0)
          .height_range(180.0..=screen * 0.55)
      } else if self.tab == Tab::Transition {
        egui::TopBottomPanel::bottom("inspector")
          .resizable(false)
          .exact_height(272.0)
      } else if self.tab == Tab::Track {
        egui::TopBottomPanel::bottom("inspector")
          .resizable(false)
          .exact_height(220.0)
      } else if self.tab == Tab::Compare {
        egui::TopBottomPanel::bottom("inspector")
          .resizable(false)
          .exact_height(380.0)
      } else {
        egui::TopBottomPanel::bottom("inspector")
          .resizable(false)
          .exact_height(176.0)
      };
      bottom_panel
        .frame(
          egui::Frame::none()
            .fill(PANEL)
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .stroke(Stroke::new(1.0_f32, LINE_SOFT)),
        )
        .show(ctx, |ui| self.bottom(ui));
    }
    if self.master_popover {
      self.shell_master_popover(ctx);
    }
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
    let lib_full = self.shell_view() == ShellView::Library && self.tab == Tab::Library;
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
        if lib_full {
          if ui.add(egui::Button::new(egui::RichText::new("← Arrangement").size(12.0))).clicked() {
            self.top = 3;
            self.tab = Tab::Playlist;
          }
          ui.add_space(6.0);
          egui::ScrollArea::vertical()
            .id_salt("lib_sources_sidebar")
            .auto_shrink([false; 2])
            .show(ui, |ui| self.library_sidebar(ui));
        } else {
          self.studio_sidebar_body(ui);
        }
        if self.shell_view() == ShellView::Arrange || lib_full {
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
            ui.menu_button(
              egui::RichText::new(format!("{}{} ", self.mix.name, dirty)).size(15.0).strong().color(TXT),
              |ui| {
                ui.horizontal(|ui| {
                  ui.label("Name:");
                  ui.text_edit_singleline(&mut self.mix.name);
                });
                ui.label(egui::RichText::new(autosave_line(&self.persistence)).size(11.0).color(DIM));
                ui.separator();
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
                if ui.button("Settings…").clicked() {
                  self.settings_open = true;
                  ui.close_menu();
                }
                if ui.button("Keyboard Shortcuts…").clicked() {
                  self.shortcuts_help = true;
                  ui.close_menu();
                }
              },
            );
            let len = self.mix.length_beats();
            let dur = fmt_t(len * 60.0 / self.mix.bpm.max(1.0));
            ui.label(
              egui::RichText::new(format!("{} tracks · {}", self.mix.tracks.len(), dur))
                .size(11.0)
                .color(MUTED),
            );
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
                egui::Button::new(egui::RichText::new("Export").size(13.0).color(TXT))
                  .fill(CTRL_FILL)
                  .stroke(Stroke::new(1.0_f32, LINE_SOFT))
                  .min_size(vec2(78.0, 28.0)),
              )
              .on_hover_text("Export mix")
              .clicked()
            {
              self.dialog(true, 2);
            }
            let ao = ui.menu_button(
              egui::RichText::new("Auto-order").size(13.0),
              |ui| {
                if ui.button("Run Auto-order").clicked() {
                  self.auto_order();
                  ui.close_menu();
                }
                ui.separator();
                ui.label(egui::RichText::new("Order by").strong());
                for m in crate::automix::Mode::ALL {
                  ui.radio_value(&mut self.ao_mode, m, m.name());
                }
                ui.separator();
                ui.label(egui::RichText::new("Energy").strong());
                for c in crate::automix::Curve::ALL {
                  ui.radio_value(&mut self.ao_curve, c, c.name());
                }
                ui.checkbox(&mut self.ao_keep_first, "Keep first track");
                if ui.button("Set All Overlaps to 8 Bars").clicked() {
                  self.snapshot();
                  self.bars = 32;
                  let o = self.mix.main_order();
                  self.mix.relayout(&o, self.bars as f64);
                  ui.close_menu();
                }
                if ui.button("Overlap Outros with Intros").clicked() {
                  self.overlap_outros_with_intros();
                  ui.close_menu();
                }
                if self.mix.has_transition_locks() && ui.button("Unlock all transitions").clicked() {
                  self.snapshot();
                  for t in &mut self.mix.tracks {
                    t.trans.locked = false;
                  }
                  ui.close_menu();
                }
              },
            );
            if self.library_ui.capture_autoorder_menu {
              let id = ao.response.id;
              ui.ctx().memory_mut(|m| m.open_popup(id));
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

  fn shell_transport_bar(&mut self, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("transport_bar")
      .exact_height(56.0)
      .frame(
        egui::Frame::none()
          .fill(PANEL)
          .inner_margin(Margin::symmetric(12.0, 6.0))
          .stroke(Stroke::new(1.0_f32, LINE_SOFT)),
      )
      .show(ctx, |ui| {
        let empty = self.mix.tracks.is_empty();
        let b = self.beat();
        let bpm = self.mix.bpm;
        let bar_n = (b / 4.0).floor() + 1.0;
        let bar_frac = (b % 4.0) + 1.0;
        let total = self.mix.length_beats();
        ui.horizontal(|ui| {
          ui.horizontal(|ui| {
            let prev = ghost_icon_button(ui, "⏮", "Previous track · double-click Return to start");
            if prev.double_clicked() {
              self.seek(0.0);
            } else if prev.clicked() {
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
            ui.add_space(6.0);
            if ui
              .add(
                egui::Button::new(egui::RichText::new("Loop").size(11.0).color(if self.loop_on { TXT } else { DIM }))
                  .fill(if self.loop_on { CTRL_FILL } else { C::TRANSPARENT })
                  .stroke(Stroke::new(if self.loop_on { 1.0 } else { 0.0 }, LINE_SOFT))
                  .min_size(vec2(40.0, 26.0)),
              )
              .clicked()
            {
              self.stop_comparison();
              self.stop_transition_audition();
              self.loop_on = !self.loop_on;
              let s = (b / 4.0).floor() * 4.0;
              self.set_loop(if self.loop_on {
                Some(self.section_range.map(|(_, a, b2)| (a, b2)).unwrap_or((s, s + self.bars as f64 * 4.0)))
              } else {
                None
              });
            }
            egui::ComboBox::from_id_salt("overlap_bars")
              .width(72.0)
              .selected_text(format!("{} bars", self.bars / 4))
              .show_ui(ui, |ui| {
                for bars in [4u32, 8, 16, 32] {
                  ui.selectable_value(&mut self.bars, bars, format!("{} bars", bars / 4));
                }
              });
          });
          ui.add_space(12.0);
          ui.horizontal(|ui| {
            let now_s = fmt_t(self.sec(b));
            let end_s = fmt_t(self.sec(total));
            ui.label(
              egui::RichText::new(if empty {
                "0:00 / 0:00".into()
              } else {
                format!("{now_s} / {end_s}")
              })
              .size(22.0)
              .strong()
              .color(TXT),
            );
            ui.add_space(16.0);
            ui.vertical(|ui| {
              ui.spacing_mut().item_spacing.y = 0.0;
              ui.label(egui::RichText::new("Bar").size(10.0).color(MUTED));
              ui.label(
                egui::RichText::new(if empty {
                  "1.1".into()
                } else {
                  format!("{:.1}", bar_n + (bar_frac - 1.0) / 4.0)
                })
                .size(12.0)
                .color(TXT),
              );
            });
            ui.add_space(12.0);
            let logical = self.mix.logical_transitions();
            let trans = logical
              .iter()
              .enumerate()
              .find(|(_, (_, _, s, e))| b >= *s && b <= *e)
              .or_else(|| logical.iter().enumerate().next());
            ui.vertical(|ui| {
              ui.spacing_mut().item_spacing.y = 0.0;
              if empty {
                ui.label(egui::RichText::new("Track").size(10.0).color(MUTED));
                ui.label(egui::RichText::new("—").size(12.0).color(DIM));
              } else if let Some((k, (o, i, s, e))) = trans {
                let left = ((e - b) / 4.0).max(0.0).ceil() as i64;
                ui.label(
                  egui::RichText::new(format!("Transition {} · {left} bars left", k + 1))
                    .size(10.0)
                    .color(MUTED),
                );
                ui.label(
                  egui::RichText::new(format!("{} -> {}", self.mix.tracks[*o].title, self.mix.tracks[*i].title))
                    .size(11.5)
                    .color(TXT),
                );
                let _ = s;
              } else {
                ui.label(egui::RichText::new("Transition").size(10.0).color(MUTED));
                ui.label(egui::RichText::new("—").size(12.0).color(DIM));
              }
            });
          });
          ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(4.0);
            ui.scope(|ui| {
              ui.set_min_width(40.0);
              self.master_meter(ui);
            });
            if ui
              .add(
                egui::Button::new(egui::RichText::new("Key Lock").size(11.0).color(if self.mix.key_lock { TXT } else { DIM }))
                  .fill(C::TRANSPARENT)
                  .frame(false),
              )
              .on_hover_text("Lock pitch when tempo changes")
              .clicked()
            {
              self.mix.key_lock = !self.mix.key_lock;
            }
            ui.label(egui::RichText::new(format!("{bpm:.1} BPM")).size(12.5).strong().color(TXT));
            let mo = self.mix.metronome;
            let click_btn = ghost_icon_button(
              ui,
              if mo { "♩" } else { "♪" },
              if mo { "Metronome on — click to disable" } else { "Metronome off — click to enable" },
            );
            if click_btn.clicked() {
              self.mix.metronome = !mo;
            }
          });
        });
      });
  }

  fn shell_settings_window(&mut self, ctx: &egui::Context) {
    if !self.settings_open {
      return;
    }
    egui::Window::new("Settings")
      .collapsible(false)
      .resizable(true)
      .default_width(420.0)
      .show(ctx, |ui| {
        ui.checkbox(&mut self.mix.auto_stems, "Separate stems automatically");
        ui.horizontal(|ui| {
          ui.label("Transition stem colours:");
          for (v, n) in [(0u8, "Normal"), (1, "Dimmed"), (2, "Inverted")] {
            ui.radio_value(&mut self.mix.trans_stem_style, v, n);
          }
        });
        if ui.button("Separate stems for all tracks").clicked() {
          for i in 0..self.mix.tracks.len() {
            self.request_stems(i);
          }
        }
        ui.label(
          egui::RichText::new(format!(
            "Stem model: HT-Demucs (MIT), {} MB, downloaded on first use to {}",
            crate::stems::MODEL_SIZE / 1_000_000,
            crate::stems::models_dir().display()
          ))
          .size(11.0)
          .color(DIM),
        );
        if ui.button("Close").clicked() {
          self.settings_open = false;
        }
      });
  }

  fn shell_shortcuts_window(&mut self, ctx: &egui::Context) {
    if !self.shortcuts_help {
      return;
    }
    egui::Window::new("Keyboard Shortcuts")
      .collapsible(false)
      .default_width(360.0)
      .show(ctx, |ui| {
        for line in [
          "Space — Play / Pause",
          "Return — To start",
          "Cmd+Z / Cmd+Shift+Z — Undo / Redo",
          "Cmd+S / Cmd+Shift+S — Save / Save as",
          "Cmd+C/V/D — Copy / Paste / Repeat section",
          "Cmd+E — Split at playhead",
          "Cmd+/ — This list",
          ", / . — Nudge incoming ±1 ms (Shift ±10)",
        ] {
          ui.label(line);
        }
        if ui.button("Close").clicked() {
          self.shortcuts_help = false;
        }
      });
  }

  fn shell_master_popover(&mut self, ctx: &egui::Context) {
    egui::Window::new("Master")
      .collapsible(false)
      .default_width(320.0)
      .show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut self.mix.master, 0.0..=2.0).text("Gain"));
        ui.checkbox(&mut self.mix.output.enabled, "Soft clipper");
        if ui.button("Close").clicked() {
          self.master_popover = false;
        }
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