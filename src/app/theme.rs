//! Visual tokens aligned with [SetLab](https://github.com/jpmacieldasilva/setlab) (`tokens.css`).

use eframe::egui::{self, Color32 as C, Rounding, Stroke, Vec2};

// Surfaces
pub const BG_WELL: C = C::from_rgb(12, 12, 14);
pub const BG: C = C::from_rgb(20, 20, 22);
pub const PANEL: C = C::from_rgb(27, 27, 30);
pub const ROW: C = C::from_rgb(24, 24, 27);
pub const SEG_INNER: C = C::from_rgb(46, 46, 50);
pub const CTRL_FILL: C = C::from_rgb(43, 43, 47);
pub const CTRL_HOVER: C = C::from_rgb(50, 50, 55);

// Text
pub const TXT: C = C::from_rgb(234, 234, 235);
pub const DIM: C = C::from_rgb(153, 153, 155);
pub const MUTED: C = C::from_rgb(128, 128, 130);

// Lines
pub const LINE: C = C::from_rgb(66, 66, 70);
pub const LINE_SOFT: C = C::from_rgb(38, 38, 41);

// Accents
pub const FOCUS: C = C::from_rgb(10, 132, 255);
pub const FOCUS_SOFT: C = C::from_rgb(28, 58, 95);
pub const ACCENT: C = C::from_rgb(229, 240, 0);
pub const ON_LIVE: C = C::from_rgb(21, 23, 10);
pub const BLUE: C = FOCUS;

pub const STEM: [C; 4] = [
    C::from_rgb(72, 188, 118),
    C::from_rgb(208, 168, 88),
    C::from_rgb(198, 82, 74),
    C::from_rgb(82, 118, 212),
];

pub const WARN: C = C::from_rgb(255, 214, 10);
pub const OK: C = C::from_rgb(48, 209, 88);
pub const DANGER: C = C::from_rgb(255, 69, 58);

pub fn apply(ctx: &egui::Context) {
    let mut st = egui::Style::default();
    st.visuals = egui::Visuals::dark();
    st.visuals.dark_mode = true;
    st.visuals.override_text_color = Some(TXT);
    st.visuals.panel_fill = PANEL;
    st.visuals.window_fill = PANEL;
    st.visuals.extreme_bg_color = BG;
    st.visuals.faint_bg_color = ROW;
    st.visuals.selection.bg_fill = FOCUS_SOFT;
    st.visuals.selection.stroke = Stroke::new(1.0_f32, FOCUS);
    st.visuals.widgets.noninteractive.bg_fill = ROW;
    st.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, LINE_SOFT);
    st.visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
    st.visuals.widgets.inactive.bg_fill = CTRL_FILL;
    st.visuals.widgets.inactive.weak_bg_fill = CTRL_FILL;
    st.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, LINE_SOFT);
    st.visuals.widgets.hovered.bg_fill = CTRL_HOVER;
    st.visuals.widgets.hovered.weak_bg_fill = CTRL_HOVER;
    st.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, LINE);
    st.visuals.widgets.active.bg_fill = SEG_INNER;
    st.visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, FOCUS);
    for w in [
        &mut st.visuals.widgets.inactive,
        &mut st.visuals.widgets.hovered,
        &mut st.visuals.widgets.active,
    ] {
        w.rounding = Rounding::same(6.0);
    }
    st.visuals.window_rounding = Rounding::same(12.0);
    st.visuals.menu_rounding = Rounding::same(6.0);
    st.visuals.window_stroke = Stroke::new(1.0_f32, LINE);
    st.visuals.hyperlink_color = FOCUS;
    st.visuals.warn_fg_color = WARN;
    st.spacing.button_padding = Vec2::new(12.0, 6.0);
    st.spacing.item_spacing = Vec2::new(8.0, 6.0);
    st.spacing.interact_size = Vec2::new(40.0, 28.0);
    ctx.set_style(st);
}

pub fn seg_button(ui: &mut egui::Ui, text: &str, on: bool) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .size(13.0)
                .color(if on { TXT } else { DIM }),
        )
        .fill(if on { SEG_INNER } else { C::TRANSPARENT })
        .stroke(Stroke::NONE),
    )
}