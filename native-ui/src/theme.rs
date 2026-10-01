//! Paleta y estilos compartidos (réplica del tema oscuro de ClipFlow).

use egui::{Color32, Stroke, Visuals};

pub struct Palette {
    pub accent: Color32,
    pub accent_dark: Color32,
    #[allow(dead_code)]
    pub accent_soft: Color32,
}

impl Palette {
    pub fn accent() -> Self {
        Self {
            accent: Color32::from_rgb(30, 158, 234), // azul de marca
            accent_dark: Color32::from_rgb(2, 132, 199),
            accent_soft: Color32::from_rgba_unmultiplied(30, 158, 234, 30),
        }
    }
}

/// Color del círculo por tipo de clip.
pub fn kind_color(kind: crate::model::ClipType, dark: bool) -> Color32 {
    use crate::model::ClipType as T;
    match kind {
        T::Text => {
            if dark {
                Color32::from_rgb(56, 189, 248)
            } else {
                Color32::from_rgb(2, 132, 199)
            }
        }
        T::Url => Color32::from_rgb(34, 197, 94),
        T::Code => Color32::from_rgb(124, 58, 237),
        T::Color => Color32::from_rgb(249, 115, 22),
        T::Image => Color32::from_rgb(236, 72, 153),
    }
}

/// Tema global. Por defecto oscuro, como en tu captura.
pub fn apply_theme(ctx: &egui::Context, dark: bool) {
    let mut v = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    v.window_corner_radius = egui::CornerRadius::same(14);
    v.menu_corner_radius = egui::CornerRadius::same(10);
    v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(10);
    v.widgets.inactive.corner_radius = egui::CornerRadius::same(10);
    v.widgets.hovered.corner_radius = egui::CornerRadius::same(10);
    v.widgets.active.corner_radius = egui::CornerRadius::same(10);
    v.widgets.open.corner_radius = egui::CornerRadius::same(10);

    v.widgets.noninteractive.bg_stroke = Stroke::NONE;
    v.selection.bg_fill = Color32::from_rgb(30, 158, 234);
    v.selection.stroke = Stroke::NONE;
    v.hyperlink_color = Color32::from_rgb(30, 158, 234);

    if dark {
        v.panel_fill = Color32::from_rgb(10, 10, 12);
        v.window_fill = Color32::from_rgb(22, 22, 25);
        v.extreme_bg_color = Color32::from_rgb(10, 10, 12);
        v.code_bg_color = Color32::from_rgb(30, 30, 34);
        v.widgets.inactive.bg_fill = Color32::from_rgb(28, 28, 32);
    } else {
        v.panel_fill = Color32::from_rgb(247, 248, 250);
        v.window_fill = Color32::WHITE;
        v.extreme_bg_color = Color32::from_rgb(247, 248, 250);
        v.code_bg_color = Color32::from_rgb(244, 244, 245);
    }

    ctx.set_visuals(v);
}

/// Etiqueta estilo <kbd>: fondo sutil + borde + monoespaciada.
pub fn kbd(ui: &mut egui::Ui, text: &str) {
    let (fill, stroke, fg) = if ui.visuals().dark_mode {
        (
            Color32::from_rgb(42, 42, 47),
            Color32::from_rgb(62, 62, 70),
            Color32::from_rgb(228, 228, 231),
        )
    } else {
        (
            Color32::from_rgb(244, 244, 245),
            Color32::from_rgb(228, 228, 231),
            Color32::from_rgb(82, 82, 91),
        )
    };
    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin {
            left: 7,
            right: 7,
            top: 2,
            bottom: 2,
        })
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .size(11.0)
                    .monospace()
                    .color(fg),
            );
        });
}
