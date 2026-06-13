//! `tokito_ui` component gallery.
//!
//! `cargo run --example gallery` — browse every component in a window.
//!
//! With `TOKITO_UI_SHOT=<path>` set, the example renders a few frames, saves
//! a PNG screenshot to that path, and exits — this is how the screenshots in
//! the README are produced.

use eframe::egui;
use tokito_ui::components as c;
use tokito_ui::{icons, Tokens};

fn main() -> eframe::Result<()> {
    let shot_path = std::env::var("TOKITO_UI_SHOT").ok();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("tokito_ui — gallery")
            .with_inner_size([960.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "tokito_ui gallery",
        options,
        Box::new(move |cc| {
            let mut fonts = egui::FontDefinitions::default();
            tokito_ui::theme::add_phosphor(&mut fonts);
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(Gallery::new(shot_path)))
        }),
    )
}

struct Gallery {
    dark: bool,
    search: String,
    project_name: String,
    notify: bool,
    // Chat-surface preview state.
    header_project_name: String,
    header_editing: bool,
    top_tab_selected: usize,
    composer: tokito_ui::components::ChatComposerState,
    rail_composer: tokito_ui::components::ChatComposerState,
    rail_state: tokito_ui::components::AiHelperRailState,
    /// When set, screenshot to this path and quit.
    shot_path: Option<String>,
    frame: u32,
}

impl Gallery {
    fn new(shot_path: Option<String>) -> Self {
        Self {
            dark: std::env::var("TOKITO_UI_THEME").as_deref() != Ok("light"),
            search: String::new(),
            project_name: "Arduino Shield".to_string(),
            notify: true,
            header_project_name: "USB-C PD Breakout".to_string(),
            header_editing: false,
            top_tab_selected: 0,
            composer: tokito_ui::components::ChatComposerState::default(),
            rail_composer: tokito_ui::components::ChatComposerState::default(),
            rail_state: tokito_ui::components::AiHelperRailState::Expanded,
            shot_path,
            frame: 0,
        }
    }
}

impl eframe::App for Gallery {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let t = if self.dark {
            Tokens::dark()
        } else {
            Tokens::light()
        };
        tokito_ui::theme::apply(ctx, &t);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(t.bg)
                    .inner_margin(egui::Margin::same(28.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.body(ui, &t));
            });

        // Screenshot mode: render a few frames so fonts/layout settle, grab
        // the framebuffer, write the PNG, and quit.
        if let Some(path) = self.shot_path.clone() {
            self.frame += 1;
            ctx.request_repaint();
            if self.frame == 4 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
            }
            let shot = ctx.input(|i| {
                i.raw.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(image) = shot {
                let pixels: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                if let Err(e) = image::save_buffer(
                    &path,
                    &pixels,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                ) {
                    eprintln!("screenshot save failed: {e}");
                } else {
                    eprintln!("wrote {path}");
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

impl Gallery {
    fn body(&mut self, ui: &mut egui::Ui, t: &Tokens) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("tokito_ui")
                    .text_style(egui::TextStyle::Heading)
                    .strong()
                    .color(t.accent),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let glyph = if self.dark {
                    icons::ph::SUN
                } else {
                    icons::ph::MOON
                };
                if c::icon_button(ui, t, glyph, 32.0, t.text_2).clicked() {
                    self.dark = !self.dark;
                }
            });
        });
        ui.label(
            egui::RichText::new("An egui 0.29 component library — the gallery.")
                .size(14.0)
                .color(t.text_2),
        );
        ui.add_space(24.0);

        // ---- buttons ----
        c::section_header(ui, t, "Buttons", None);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            c::text_button(ui, t, c::ButtonKind::Primary, "Primary", 34.0);
            ui.add_space(8.0);
            c::text_button(ui, t, c::ButtonKind::Secondary, "Secondary", 34.0);
            ui.add_space(12.0);
            c::icon_button(ui, t, icons::ph::GEAR_SIX, 34.0, t.text_2);
            c::icon_button(ui, t, icons::ph::TRASH, 34.0, t.accent);
            ui.add_space(12.0);
            c::link(ui, t, "A text link");
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            c::badge(ui, t, "3 designs");
            ui.add_space(6.0);
            c::badge(ui, t, "v0.1.0");
            ui.add_space(6.0);
            c::toggle(ui, t, &mut self.notify, "Enable notifications");
        });
        ui.add_space(26.0);

        // ---- inputs ----
        c::section_header(ui, t, "Inputs", None);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            c::text_input(
                ui,
                t,
                "g_name",
                &mut self.project_name,
                "Project name",
                220.0,
            );
            ui.add_space(10.0);
            c::search_field(ui, t, "g_search", &mut self.search, "Search…", 220.0);
        });
        ui.add_space(26.0);

        // ---- cards ----
        c::section_header(ui, t, "Cards", None);
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(14.0, 14.0);
            for (icon, name, meta) in [
                (icons::ph::FOLDER, "Bench instruments", "4 designs"),
                (icons::ph::TREE_STRUCTURE, "Buck converter", "2 h ago"),
            ] {
                c::card(ui, t, egui::vec2(210.0, 120.0), |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(icons::icon(icon, 22.0, t.accent));
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(name).size(14.5).strong().color(t.text));
                    ui.add_space(8.0);
                    ui.label(icons::icon_text(
                        icons::ph::CLOCK,
                        13.0,
                        meta,
                        12.0,
                        t.text_3,
                    ));
                });
            }
            c::new_tile(ui, t, "New project", None, egui::vec2(210.0, 120.0));
        });
        ui.add_space(26.0);

        // ---- list ----
        c::section_header(ui, t, "List rows", None);
        ui.add_space(10.0);
        for (i, (icon, label)) in [
            (icons::ph::FOLDER, "Default"),
            (icons::ph::FOLDER, "Personal RF"),
            (icons::ph::TREE_STRUCTURE, "Sensor node rev B"),
        ]
        .into_iter()
        .enumerate()
        {
            let job = icons::icon_text(icon, 14.0, label, 13.0, t.text);
            c::list_row(ui, t, job, i == 0);
        }
        ui.add_space(30.0);

        // ---- chat shell preview ----
        c::section_header(ui, t, "Chat shell", None);
        ui.add_space(10.0);

        // App header.
        c::app_header(
            ui,
            t,
            icons::ph::ARROW_UP,
            "Tokito",
            &mut self.header_project_name,
            &mut self.header_editing,
        );
        ui.add_space(2.0);

        // Top tab bar.
        let tabs = [
            c::TabItem::Tab {
                icon: icons::ph::CHAT_CIRCLE,
                label: "Chat",
            },
            c::TabItem::Tab {
                icon: icons::ph::FILE_TEXT,
                label: "Plan",
            },
            c::TabItem::Tab {
                icon: icons::ph::FOLDER,
                label: "Artifacts",
            },
            c::TabItem::Divider,
            c::TabItem::Tab {
                icon: icons::ph::TREE_STRUCTURE,
                label: "Schematic",
            },
            c::TabItem::Tab {
                icon: icons::ph::CPU,
                label: "PCB",
            },
            c::TabItem::Tab {
                icon: icons::ph::SHOPPING_CART,
                label: "BOM",
            },
            c::TabItem::Tab {
                icon: icons::ph::DATABASE,
                label: "Research",
            },
            c::TabItem::Tab {
                icon: icons::ph::DOWNLOAD_SIMPLE,
                label: "Export",
            },
        ];
        if let Some(i) = c::tab_bar(ui, t, &tabs, self.top_tab_selected) {
            self.top_tab_selected = i;
        }
        ui.add_space(14.0);

        // Three-column chat surface mock — sidebar | history | rail.
        ui.horizontal_top(|ui| {
            // Sidebar.
            ui.allocate_ui_with_layout(
                egui::vec2(220.0, 260.0),
                egui::Layout::top_down(egui::Align::Min).with_cross_justify(true),
                |ui| {
                    c::conversation_sidebar(ui, t, |ui| {
                        c::thread_row(
                            ui,
                            t,
                            "Power supply",
                            "Let's switch the LDO to AMS1117…",
                            "2m",
                            true,
                            false,
                        );
                        c::thread_row(
                            ui,
                            t,
                            "USB-C subsystem",
                            "Add CC pull-downs and the…",
                            "3h",
                            false,
                            false,
                        );
                        c::thread_row(
                            ui,
                            t,
                            "Bootstrap caps",
                            "What size for the buck stage?",
                            "yesterday",
                            false,
                            false,
                        );
                        c::sidebar_divider(ui, t);
                        c::thread_row(
                            ui,
                            t,
                            "Workshop",
                            "Cross-design notes & one-offs",
                            "",
                            false,
                            true,
                        );
                    });
                },
            );
            ui.add_space(12.0);

            // History column.
            ui.allocate_ui_with_layout(
                egui::vec2(380.0, 260.0),
                egui::Layout::top_down(egui::Align::Min).with_cross_justify(true),
                |ui| {
                    egui::Frame::none()
                        .fill(t.bg)
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("gallery_chat_history")
                                .auto_shrink([false, false])
                                .max_height(180.0)
                                .show(ui, |ui| {
                                    c::chat_bubble(ui, t, c::BubbleKind::Assistant, "", |ui| {
                                        ui.label(
                                            egui::RichText::new(
                                                "Your Arduino shield design is ready! I've created a motor driver \
                                                 board with sensors and USB-C power. What would you like to adjust?",
                                            )
                                            .color(t.text),
                                        );
                                    });
                                    ui.add_space(10.0);
                                    c::chat_bubble(ui, t, c::BubbleKind::User, "JA", |ui| {
                                        ui.label(
                                            egui::RichText::new("Replace the buzzer with a louder one.")
                                                .color(t.text),
                                        );
                                    });
                                    ui.add_space(10.0);
                                    c::chat_bubble(ui, t, c::BubbleKind::Assistant, "", |ui| {
                                        ui.label(
                                            egui::RichText::new("Searching catalog… one moment.")
                                                .color(t.text),
                                        );
                                    });
                                });
                            ui.add_space(8.0);
                            c::chat_composer(
                                ui,
                                t,
                                &mut self.composer,
                                "Ask Tokito to modify your design…",
                            );
                        });
                },
            );
            ui.add_space(12.0);

            // Rail.
            let rail_w = match self.rail_state {
                c::AiHelperRailState::Hidden => 0.0,
                c::AiHelperRailState::CollapsedGlyph => 44.0,
                c::AiHelperRailState::Expanded => 240.0,
            };
            if rail_w > 0.0 {
                ui.allocate_ui_with_layout(
                    egui::vec2(rail_w, 260.0),
                    egui::Layout::top_down(egui::Align::Min).with_cross_justify(true),
                    |ui| {
                        let suggestions = [
                            "Make the buzzer louder",
                            "Add a power LED indicator",
                            "Replace with lower cost parts",
                        ];
                        if let Some(action) = c::ai_helper_rail(
                            ui,
                            t,
                            self.rail_state,
                            &suggestions,
                            &mut self.rail_composer,
                        ) {
                            match action {
                                c::AiHelperRailAction::Collapse => {
                                    self.rail_state = c::AiHelperRailState::CollapsedGlyph;
                                }
                                c::AiHelperRailAction::Expand => {
                                    self.rail_state = c::AiHelperRailState::Expanded;
                                }
                                c::AiHelperRailAction::Close => {
                                    self.rail_state = c::AiHelperRailState::Hidden;
                                }
                                _ => {}
                            }
                        }
                    },
                );
            }
        });

        // Help button (floating bottom-right of the example window).
        let area_pos = ui.ctx().screen_rect().right_bottom() - egui::vec2(24.0, 24.0);
        egui::Area::new(egui::Id::new("gallery_help"))
            .order(egui::Order::Foreground)
            .fixed_pos(area_pos - egui::vec2(32.0, 32.0))
            .show(ui.ctx(), |ui| {
                c::floating_help_button(ui, t);
            });
    }
}
