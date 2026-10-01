//! ClipFlow nativo — réplica de la UI clásica, 100% sin WebView2.
//!
//! Header + buscador + pills de filtro + tarjeta de bienvenida +
//! lista + vista previa + ajustes + modo rápido + toasts.

use egui::{Align, Color32, Layout, RichText};

use crate::model::{
    format_age, format_size, seed_items, ClipItem, ClipType, Filter, Settings,
};
use crate::theme::{self, Palette};

pub struct ClipFlowApp {
    items: Vec<ClipItem>,
    query: String,
    filter: Filter,
    selected_id: Option<String>,
    toast: Option<(String, std::time::Instant)>,
    dark: bool,
    show_settings: bool,
    spotlight: bool,
    clear_armed_at: Option<std::time::Instant>,
    visible: usize,
    settings: Settings,
    focus_search: bool,
    search_focused: bool,
    scroll_to_selected: bool,
    last_sig: String,
}

impl ClipFlowApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // La UI clásica es oscura: se arranca en oscuro salvo que el
        // sistema pida claro de forma explícita y el usuario lo conserve.
        let system_dark = cc.egui_ctx.style().visuals.dark_mode;
        let _ = system_dark;
        let mut app = Self {
            items: seed_items(),
            query: String::new(),
            filter: Filter::All,
            selected_id: None,
            toast: None,
            dark: true,
            show_settings: false,
            spotlight: false,
            clear_armed_at: None,
            visible: 60,
            settings: Settings::default(),
            focus_search: true,
            search_focused: false,
            scroll_to_selected: false,
            last_sig: String::new(),
        };
        app.settings.dark_mode = true;
        app.ensure_selection();
        app
    }

    // ---------------- estado ----------------

    fn filtered_indices(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, it)| self.filter.matches(it.kind) && it.matches_query(&self.query))
            .map(|(i, _)| i)
            .collect()
    }

    fn ensure_selection(&mut self) {
        let ids = self.filtered_indices();
        if ids.is_empty() {
            self.selected_id = None;
            return;
        }
        let ok = self
            .selected_id
            .as_ref()
            .is_some_and(|s| ids.iter().any(|&i| &self.items[i].id == s));
        if !ok {
            self.selected_id = Some(self.items[ids[0]].id.clone());
        }
    }

    fn selected_item(&self) -> Option<ClipItem> {
        self.selected_id
            .as_ref()
            .and_then(|id| self.items.iter().find(|i| &i.id == id).cloned())
    }

    fn notify(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), std::time::Instant::now()));
    }

    fn move_selection(&mut self, dir: i32) {
        let ids = self.filtered_indices();
        if ids.is_empty() {
            return;
        }
        let cur = self
            .selected_id
            .as_ref()
            .and_then(|s| ids.iter().position(|&i| &self.items[i].id == s))
            .unwrap_or(0) as i32;
        let next = (cur + dir).rem_euclid(ids.len() as i32) as usize;
        self.selected_id = Some(self.items[ids[next]].id.clone());
        if next + 1 > self.visible {
            self.visible = next + 1;
        }
        self.scroll_to_selected = true;
    }

    fn do_copy(&mut self, id: &str) {
        let item = match self.items.iter().find(|i| i.id == id) {
            Some(it) => it.clone(),
            None => return,
        };
        let text = if item.kind == ClipType::Image {
            item.ocr.clone().unwrap_or_else(|| item.preview.clone())
        } else {
            item.content.clone()
        };
        match arboard::Clipboard::new().and_then(|mut c| c.set_text(text)) {
            Ok(()) => self.notify("Copiado"),
            Err(_) => self.notify("No se pudo acceder al portapapeles"),
        }
    }

    fn do_delete(&mut self, id: &str) {
        self.items.retain(|i| i.id != id);
        self.ensure_selection();
        self.notify("Eliminado");
    }

    fn do_clear(&mut self) {
        let now = std::time::Instant::now();
        let armed = self
            .clear_armed_at
            .is_some_and(|t| now.duration_since(t).as_secs() < 3);
        if !armed {
            self.clear_armed_at = Some(now);
            self.notify("Pulsa de nuevo para vaciar el historial");
            return;
        }
        self.clear_armed_at = None;
        self.items.clear();
        self.selected_id = None;
        self.notify("Historial vaciado");
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        let search_focused = self.search_focused;
        let mut focus_search = false;
        let mut copy_sel = false;
        let mut del_sel = false;

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            if search_focused {
                self.search_focused = false;
                self.move_selection(0);
                self.scroll_to_selected = true;
            } else {
                self.move_selection(1);
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) && !search_focused {
            self.move_selection(-1);
        } else if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            copy_sel = true;
        } else if ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace))
            && !search_focused
        {
            del_sel = true;
        } else if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.show_settings {
                self.show_settings = false;
            } else if self.spotlight {
                self.spotlight = false;
            } else if !self.query.is_empty() {
                self.query.clear();
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::Slash)) && !search_focused {
            focus_search = true;
        }

        if focus_search {
            self.focus_search = true;
        }
        if copy_sel {
            if let Some(id) = self.selected_id.clone() {
                self.do_copy(&id);
                if self.spotlight {
                    self.spotlight = false;
                }
            }
        }
        if del_sel {
            if let Some(id) = self.selected_id.clone() {
                self.do_delete(&id);
            }
        }
    }
}

impl eframe::App for ClipFlowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply_theme(ctx, self.dark);
        let pal = Palette::accent();

        if let Some((_, t)) = &self.toast {
            if t.elapsed().as_millis() > 2200 {
                self.toast = None;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(300));
            }
        }
        if let Some(t) = self.clear_armed_at {
            if t.elapsed().as_secs() >= 3 {
                self.clear_armed_at = None;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(500));
            }
        }

        let sig = format!(
            "{}|{}",
            self.query,
            match self.filter {
                Filter::All => "all".to_string(),
                Filter::One(k) => format!("one-{}", k.label()),
            }
        );
        if sig != self.last_sig {
            self.last_sig = sig;
            self.visible = 60;
        }
        self.ensure_selection();
        self.handle_keys(ctx);

        let title = if self.spotlight {
            "ClipFlow — acceso rápido"
        } else if self.show_settings {
            "ClipFlow — ajustes"
        } else {
            "ClipFlow"
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.to_owned()));

        // ================= HEADER (como la captura) =================
        if !self.spotlight {
            egui::TopBottomPanel::top("clipflow-header")
                .exact_height(68.0)
                .show(ctx, |ui| {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        // Logo: cuadrado azul redondeado con portapapeles.
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
                        ui.painter().rect_filled(
                            rect,
                            egui::CornerRadius::same(9),
                            Color32::from_rgb(30, 158, 234),
                        );
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "📋",
                            egui::FontId::proportional(18.0),
                            Color32::WHITE,
                        );

                        ui.add_space(2.0);
                        ui.vertical(|ui| {
                            ui.add_space(1.0);
                            ui.label(RichText::new("ClipFlow").size(17.0).strong());
                            let n = self.items.len();
                            ui.label(
                                RichText::new(format!(
                                    "{} elemento{} · se guarda todo lo que copies",
                                    n,
                                    if n == 1 { "" } else { "s" }
                                ))
                                .size(11.0)
                                .color(Color32::from_rgb(161, 161, 170)),
                            );
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(6.0);
                            // Papelera
                            let trash = egui::Button::new(
                                RichText::new("🗑").size(15.0),
                            )
                            .min_size(egui::vec2(30.0, 30.0))
                            .fill(Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(egui::CornerRadius::same(8));
                            if ui
                                .add(trash)
                                .on_hover_text("Vaciar historial (pide confirmación)")
                                .clicked()
                            {
                                self.do_clear();
                            }
                            // Ajustes
                            let gear = egui::Button::new(RichText::new("⚙").size(15.0))
                                .min_size(egui::vec2(30.0, 30.0))
                                .fill(Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                                .corner_radius(egui::CornerRadius::same(8));
                            if ui.add(gear).on_hover_text("Abrir ajustes").clicked() {
                                self.show_settings = true;
                            }
                            // Recargar (azul, como en la captura)
                            let reload = egui::Button::new(
                                RichText::new("↻").size(16.0).color(Color32::WHITE),
                            )
                            .min_size(egui::vec2(30.0, 30.0))
                            .fill(Color32::from_rgb(30, 158, 234))
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(egui::CornerRadius::same(8));
                            if ui.add(reload).on_hover_text("Recargar historial").clicked() {
                                if self.items.is_empty() {
                                    self.items = seed_items();
                                    self.ensure_selection();
                                    self.notify("Ejemplos cargados");
                                } else {
                                    self.ensure_selection();
                                    self.notify("Historial actualizado");
                                }
                            }
                            // Tema claro/oscuro + modo rápido (discretos)
                            if ui.small_button(if self.dark { "☀" } else { "🌙" }).clicked() {
                                self.dark = !self.dark;
                                self.settings.dark_mode = self.dark;
                            }
                            if ui.small_button("⚡").on_hover_text("Modo acceso rápido").clicked() {
                                self.spotlight = true;
                                self.focus_search = true;
                            }
                        });
                    });
                    ui.add_space(10.0);
                });
        }

        // ================= CONTENIDO =================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(if self.spotlight { 10.0 } else { 10.0 });

            if self.spotlight {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new("ClipFlow").size(14.0).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Enter pega · Esc cierra")
                                .size(11.0)
                                .color(Color32::from_rgb(161, 161, 170)),
                        );
                    });
                });
                ui.add_space(6.0);
            }

            // ---- Buscador (como la captura: lupa + hint + "/" dentro) ----
            let search_bg = if self.dark {
                Color32::from_rgb(22, 22, 25)
            } else {
                Color32::WHITE
            };
            let search_bd = if self.dark {
                Color32::from_rgb(44, 44, 50)
            } else {
                Color32::from_rgb(228, 228, 231)
            };
            let mut search_resp_opt = None;
            egui::Frame::new()
                .fill(search_bg)
                .stroke(egui::Stroke::new(1.0, search_bd))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin {
                    left: 10,
                    right: 10,
                    top: 4,
                    bottom: 4,
                })
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("⌕")
                                .size(16.0)
                                .color(Color32::from_rgb(161, 161, 170)),
                        );
                        let w = (ui.available_width() - 34.0).max(80.0);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.query)
                                .hint_text("Buscar en tu historial... (texto, enlaces, colores...)")
                                .desired_width(w)
                                .frame(false)
                                .margin(egui::Margin::same(6)),
                        );
                        search_resp_opt = Some(resp);
                        theme::kbd(ui, "/");
                    });
                });
            if let Some(resp) = search_resp_opt {
                if self.focus_search {
                    resp.request_focus();
                    self.focus_search = false;
                }
                self.search_focused = resp.has_focus();
            }
            if self.spotlight && self.query.is_empty() && !self.search_focused {
                self.focus_search = true;
            }

            ui.add_space(10.0);

            // ---- Pills de filtro (como la captura) ----
            if !self.spotlight {
                egui::ScrollArea::horizontal()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(2.0);
                            let pills: Vec<(Filter, &str)> = vec![
                                (Filter::All, "Todos"),
                                (Filter::One(ClipType::Text), "📄 Texto"),
                                (Filter::One(ClipType::Url), "🔗 Enlaces"),
                                (Filter::One(ClipType::Code), "⌨ Código"),
                                (Filter::One(ClipType::Color), "🎨 Colores"),
                                (Filter::One(ClipType::Image), "🖼 Imágenes"),
                            ];
                            for (f, label) in pills {
                                let active = self.filter == f;
                                let btn = egui::Button::new(
                                    RichText::new(label)
                                        .size(12.5)
                                        .color(if active {
                                            Color32::WHITE
                                        } else if self.dark {
                                            Color32::from_rgb(212, 212, 216)
                                        } else {
                                            Color32::from_rgb(82, 82, 91)
                                        }),
                                )
                                .min_size(egui::vec2(0.0, 30.0))
                                .fill(if active {
                                    Color32::from_rgb(30, 158, 234)
                                } else if self.dark {
                                    Color32::from_rgb(28, 28, 32)
                                } else {
                                    Color32::WHITE
                                })
                                .stroke(if active {
                                    egui::Stroke::NONE
                                } else {
                                    egui::Stroke::new(
                                        1.0,
                                        if self.dark {
                                            Color32::from_rgb(46, 46, 53)
                                        } else {
                                            Color32::from_rgb(228, 228, 231)
                                        },
                                    )
                                })
                                .corner_radius(egui::CornerRadius::same(15));
                                if ui.add(btn).clicked() {
                                    self.filter = f;
                                }
                            }
                        });
                    });
                ui.add_space(10.0);
            }

            // ---- Cuerpo: bienvenida / sin resultados / lista ----
            let indices = self.filtered_indices();
            let total = indices.len();
            let shown = total.min(self.visible.max(20));

            if self.items.is_empty() {
                welcome_card(ui, self.dark);
            } else if total == 0 {
                empty_no_results(ui);
            } else {
                let list_h = if self.spotlight { 320.0 } else { 300.0 };
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(list_h)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for &idx in indices.iter().take(shown) {
                            let item = self.items[idx].clone();
                            let is_sel = self.selected_id.as_deref() == Some(item.id.as_str());
                            let resp = item_card(ui, &item, is_sel, self.dark, &pal);
                            if resp.clicked() {
                                self.selected_id = Some(item.id.clone());
                                if self.spotlight {
                                    self.do_copy(&item.id);
                                    self.spotlight = false;
                                }
                            }
                            if is_sel && self.scroll_to_selected {
                                resp.scroll_to_me(Some(egui::Align::Center));
                            }
                            if is_sel && !self.spotlight {
                                ui.horizontal(|ui| {
                                    ui.add_space(48.0);
                                    if ui.small_button("Copiar").clicked() {
                                        self.do_copy(&item.id);
                                    }
                                    if ui.small_button("Pegar").clicked() {
                                        self.do_copy(&item.id);
                                    }
                                    if ui.small_button("Eliminar").clicked() {
                                        self.do_delete(&item.id);
                                    }
                                    ui.with_layout(
                                        Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(format_age(item.timestamp))
                                                    .size(11.0)
                                                    .color(Color32::from_rgb(161, 161, 170)),
                                            );
                                        },
                                    );
                                });
                                ui.add_space(2.0);
                            }
                        }
                    });
                self.scroll_to_selected = false;

                if total > shown {
                    ui.add_space(6.0);
                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new(format!(
                                "Mostrar más (quedan {})",
                                total - shown
                            )),
                        )
                        .clicked()
                    {
                        self.visible += 60;
                    }
                }
            }

            // ---- Vista previa ----
            if !self.spotlight {
                if let Some(sel) = self.selected_item() {
                    if !self.items.is_empty() && total > 0 {
                        ui.add_space(8.0);
                        let action = preview_panel(ui, &sel, self.dark, &pal);
                        match action {
                            PreviewOut::None => {}
                            PreviewOut::Copy(id) => self.do_copy(&id),
                            PreviewOut::Delete(id) => self.do_delete(&id),
                        }
                    }
                }
            }

            // ---- Footer (como la captura) ----
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                theme::kbd(ui, "↑");
                theme::kbd(ui, "↓");
                ui.label(RichText::new("navegar").size(11.0));
                theme::kbd(ui, "Enter");
                ui.label(RichText::new("copiar").size(11.0));
                theme::kbd(ui, "Supr");
                ui.label(RichText::new("borrar").size(11.0));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("nativa · sin WebView2")
                            .size(11.0)
                            .color(Color32::from_rgb(113, 113, 122)),
                    );
                });
            });
            ui.add_space(2.0);
        });

        // ================= AJUSTES =================
        if self.show_settings {
            let mut open = self.show_settings;
            let mut save_clicked = false;
            egui::Window::new("Ajustes de ClipFlow")
                .open(&mut open)
                .resizable(true)
                .default_size([470.0, 560.0])
                .show(ctx, |ui| {
                    save_clicked = settings_view(ui, &mut self.settings, &mut self.dark, &pal);
                });
            self.show_settings = open;
            if save_clicked {
                self.notify("Ajustes guardados");
            }
        }

        // ================= TOAST (píldora blanca, como la captura) =================
        if let Some((msg, _)) = self.toast.clone() {
            egui::Area::new(egui::Id::new("clipflow-toast"))
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -30.0))
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(Color32::from_rgb(250, 250, 250))
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(228, 228, 231)))
                        .corner_radius(egui::CornerRadius::same(18))
                        .inner_margin(egui::Margin {
                            left: 18,
                            right: 18,
                            top: 9,
                            bottom: 9,
                        })
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(msg)
                                    .size(13.0)
                                    .color(Color32::from_rgb(24, 24, 27)),
                            );
                        });
                });
        }
    }
}

// ================= piezas =================

fn card_frame(ui: &egui::Ui, selected: bool, dark: bool, pal: &Palette) -> egui::Frame {
    let (fill, stroke) = if selected {
        (
            if dark {
                Color32::from_rgb(26, 34, 44)
            } else {
                Color32::from_rgb(240, 249, 255)
            },
            egui::Stroke::new(1.5, pal.accent),
        )
    } else if dark {
        (
            Color32::from_rgb(22, 22, 25),
            egui::Stroke::new(1.0, Color32::from_rgb(42, 42, 48)),
        )
    } else {
        (
            Color32::WHITE,
            egui::Stroke::new(1.0, Color32::from_rgb(228, 228, 231)),
        )
    };
    let _ = ui;
    egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(10))
}

fn item_card(
    ui: &mut egui::Ui,
    item: &ClipItem,
    selected: bool,
    dark: bool,
    pal: &Palette,
) -> egui::Response {
    let frame = card_frame(ui, selected, dark, pal);
    let inner = frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            if item.kind == ClipType::Color {
                let c = parse_hex(&item.content).unwrap_or(pal.accent);
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, egui::CornerRadius::same(10), c);
                ui.painter().rect_stroke(
                    rect,
                    egui::CornerRadius::same(10),
                    egui::Stroke::new(1.0, Color32::from_rgb(161, 161, 170)),
                    egui::StrokeKind::Inside,
                );
            } else {
                let col = crate::theme::kind_color(item.kind, dark);
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 18.0, col);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    item.kind.glyph(),
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
            }

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(item.kind.label())
                            .size(11.0)
                            .strong()
                            .color(Color32::from_rgb(161, 161, 170)),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format_age(item.timestamp))
                                .size(11.0)
                                .color(Color32::from_rgb(161, 161, 170)),
                        );
                    });
                });
                let preview = if item.preview.chars().count() > 140 {
                    format!("{}…", item.preview.chars().take(140).collect::<String>())
                } else {
                    item.preview.clone()
                };
                if item.kind == ClipType::Code {
                    ui.label(RichText::new(preview).size(13.0).monospace());
                } else {
                    ui.label(RichText::new(preview).size(13.0));
                }
                ui.horizontal(|ui| {
                    if item.kind != ClipType::Color {
                        ui.label(
                            RichText::new(format_size(item.size))
                                .size(11.0)
                                .color(Color32::from_rgb(161, 161, 170)),
                        );
                    } else {
                        ui.label(
                            RichText::new(item.content.clone())
                                .size(12.0)
                                .monospace()
                                .color(Color32::from_rgb(161, 161, 170)),
                        );
                    }
                    if item.ocr.is_some() {
                        ui.label(
                            RichText::new("OCR")
                                .size(11.0)
                                .strong()
                                .color(pal.accent),
                        );
                    }
                });
            });
        });
    });
    let mut resp = ui.interact(
        inner.response.rect,
        ui.make_persistent_id(format!("clip-{}", item.id)),
        egui::Sense::click(),
    );
    if inner.response.hovered() {
        resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    resp
}

#[derive(Debug, Clone)]
enum PreviewOut {
    None,
    Copy(String),
    Delete(String),
}

fn preview_panel(ui: &mut egui::Ui, item: &ClipItem, dark: bool, pal: &Palette) -> PreviewOut {
    let fill = if dark {
        Color32::from_rgb(22, 22, 25)
    } else {
        Color32::WHITE
    };
    let mut out = PreviewOut::None;
    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::new(
            1.0,
            if dark {
                Color32::from_rgb(42, 42, 48)
            } else {
                Color32::from_rgb(228, 228, 231)
            },
        ))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new("Vista previa").size(12.0).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("Eliminar").clicked() {
                        out = PreviewOut::Delete(item.id.clone());
                    }
                    if ui.small_button("Copiar").clicked() {
                        out = PreviewOut::Copy(item.id.clone());
                    }
                });
            });
            ui.add_space(4.0);
            if item.kind == ClipType::Color {
                ui.horizontal(|ui| {
                    let c = parse_hex(&item.content).unwrap_or(pal.accent);
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(56.0, 56.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::same(12), c);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(&item.content)
                                .size(15.0)
                                .monospace()
                                .strong(),
                        );
                        if ui.small_button("Copiar color").clicked() {
                            out = PreviewOut::Copy(item.id.clone());
                        }
                    });
                });
            } else if item.kind == ClipType::Image {
                ui.label(RichText::new("Imagen del historial").size(13.0));
                ui.label(
                    RichText::new(&item.preview)
                        .size(12.0)
                        .color(Color32::from_rgb(161, 161, 170)),
                );
                if let Some(ocr) = &item.ocr {
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("OCR: {}", ocr)).size(12.0).italics());
                }
            } else if item.kind == ClipType::Code {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .max_height(120.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new(&item.content).size(13.0).monospace());
                    });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .max_height(120.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new(&item.content).size(13.0));
                    });
            }
        });
    out
}

/// Tarjeta de bienvenida: réplica de la captura.
fn welcome_card(ui: &mut egui::Ui, dark: bool) {
    let (fill, stroke) = if dark {
        (
            Color32::from_rgb(22, 22, 25),
            Color32::from_rgb(42, 42, 48),
        )
    } else {
        (Color32::WHITE, Color32::from_rgb(228, 228, 231))
    };
    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(18))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.add_space(14.0);
                // Icono: círculo con portapapeles + insignia "+".
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(76.0, 76.0), egui::Sense::hover());
                ui.painter().circle_filled(
                    rect.center(),
                    34.0,
                    if dark {
                        Color32::from_rgb(38, 38, 44)
                    } else {
                        Color32::from_rgb(244, 244, 245)
                    },
                );
                ui.painter().text(
                    rect.center() + egui::vec2(-4.0, -3.0),
                    egui::Align2::CENTER_CENTER,
                    "📋",
                    egui::FontId::proportional(30.0),
                    Color32::from_rgb(139, 139, 146),
                );
                let badge = rect.center() + egui::vec2(17.0, 17.0);
                ui.painter()
                    .circle_filled(badge, 11.0, Color32::from_rgb(82, 82, 91));
                ui.painter().text(
                    badge,
                    egui::Align2::CENTER_CENTER,
                    "+",
                    egui::FontId::proportional(15.0),
                    Color32::WHITE,
                );
                ui.add_space(10.0);
                ui.label(RichText::new("Tu portapapeles, siempre a mano").size(17.0).strong());
                ui.add_space(2.0);
                ui.label(
                    RichText::new("Copia algo y aparecerá aquí automáticamente")
                        .size(13.0)
                        .color(Color32::from_rgb(161, 161, 170)),
                );
                ui.add_space(12.0);
            });
            // Pasos (alineados a la izquierda, con kbd en línea).
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("1. ").size(13.0).strong());
                ui.label(RichText::new("Copia como siempre con ").size(13.0));
                theme::kbd(ui, "ctrl");
                ui.label(RichText::new(" + ").size(13.0));
                theme::kbd(ui, "c");
                ui.label(RichText::new(": texto, enlaces, código, colores e imágenes.").size(13.0));
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("2. ").size(13.0).strong());
                ui.label(RichText::new("Pulsa ").size(13.0));
                theme::kbd(ui, "ctrl");
                ui.label(RichText::new(" + ").size(13.0));
                theme::kbd(ui, "shift");
                ui.label(RichText::new(" + ").size(13.0));
                theme::kbd(ui, "V");
                ui.label(RichText::new(" en cualquier app para buscar y pegar.").size(13.0));
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("3. ").size(13.0).strong());
                ui.label(
                    RichText::new(
                        "Clic en un elemento para previsualizarlo y copiarlo en el formato que quieras.",
                    )
                    .size(13.0),
                );
            });
            ui.add_space(12.0);
        });
}

fn empty_no_results(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(28.0);
        ui.label(RichText::new("Sin resultados").size(16.0).strong());
        ui.label(
            RichText::new("Prueba con otra búsqueda o cambia de filtro.")
                .size(13.0)
                .color(Color32::from_rgb(161, 161, 170)),
        );
    });
}

fn settings_view(ui: &mut egui::Ui, s: &mut Settings, dark: &mut bool, pal: &Palette) -> bool {
    let mut save_clicked = false;
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new("General").size(14.0).strong());
        ui.add_space(4.0);
        ui.checkbox(&mut s.launch_at_startup, "Iniciar con Windows (bandeja)");
        ui.checkbox(
            &mut s.exclude_passwords,
            "Excluir contraseñas y datos sensibles",
        );
        ui.checkbox(&mut s.ocr_enabled, "OCR automático en imágenes (100% local)");
        ui.add(egui::Slider::new(&mut s.history_size, 50..=1000).text("Tamaño del historial"));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Tema:");
            if ui.selectable_label(!*dark, "Claro").clicked() {
                *dark = false;
                s.dark_mode = false;
            }
            if ui.selectable_label(*dark, "Oscuro").clicked() {
                *dark = true;
                s.dark_mode = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Idioma:");
            egui::ComboBox::from_id_salt("lang")
                .selected_text(s.language.clone())
                .show_ui(ui, |ui| {
                    for lang in ["Español", "English", "Português", "Français", "Deutsch"] {
                        ui.selectable_value(&mut s.language, lang.to_string(), lang);
                    }
                });
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);
        ui.label(RichText::new("Apariencia nativa").size(14.0).strong());
        ui.label(
            RichText::new(
                "Esta ventana se dibuja por GPU con egui: sin WebView2, \
                 sin Edge embebido y sin descargas en tiempo de instalación.",
            )
            .size(12.0)
            .color(Color32::from_rgb(161, 161, 170)),
        );
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            for (name, hex) in [
                ("Cielo", "#0EA5E9"),
                ("Violeta", "#7C3AED"),
                ("Verde", "#22C55E"),
            ] {
                let c = parse_hex(hex).unwrap_or(pal.accent);
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, egui::CornerRadius::same(8), c);
                ui.label(RichText::new(name).size(12.0));
            }
        });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui
                .add_sized([120.0, 32.0], egui::Button::new("Guardar"))
                .clicked()
            {
                save_clicked = true;
            }
            if ui.small_button("Restablecer").clicked() {
                let dark_now = *dark;
                *s = Settings::default();
                s.dark_mode = dark_now;
                *dark = dark_now;
            }
        });
        ui.add_space(6.0);
    });
    save_clicked
}

fn parse_hex(hex: &str) -> Option<Color32> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some(Color32::from_rgb(r, g, b))
}
