use egui_extras::{Size, StripBuilder};
use nalgebra as na;

enum RotationRepr {
    Quaternion,
    AngleAxis,
    RotationMatrix,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    quat: [(String, String); 4],
    angleaxis: [(String, String); 4],
    rot_matrix: [String; 9],
    editted: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            quat: [
                ("Qw".to_owned(), "1.0".to_owned()),
                ("Qx".to_owned(), "0.0".to_owned()),
                ("Qy".to_owned(), "0.0".to_owned()),
                ("Qz".to_owned(), "0.0".to_owned()),
            ],
            angleaxis: [
                ("Ang (rad)".to_owned(), "0.0".to_owned()),
                ("AxisX".to_owned(), "1.0".to_owned()),
                ("AxisY".to_owned(), "0.0".to_owned()),
                ("AxisZ".to_owned(), "0.0".to_owned()),
            ],
            rot_matrix: [
                "1.0".to_owned(),
                "0.0".to_owned(),
                "0.0".to_owned(),
                "0.0".to_owned(),
                "1.0".to_owned(),
                "0.0".to_owned(),
                "0.0".to_owned(),
                "0.0".to_owned(),
                "1.0".to_owned(),
            ],
            editted: false,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    fn update_input(&mut self, edited_item: RotationRepr) -> anyhow::Result<()> {
        let quat = match edited_item {
            RotationRepr::Quaternion => {
                na::UnitQuaternion::<f64>::from_quaternion(na::Quaternion::new(
                    self.quat[0].1.parse()?,
                    self.quat[1].1.parse()?,
                    self.quat[2].1.parse()?,
                    self.quat[3].1.parse()?,
                ))
            }
            RotationRepr::AngleAxis => {
                let angle = self.angleaxis[0].1.parse()?;
                let axis = na::UnitVector3::new_normalize(na::Vector3::new(
                    self.angleaxis[1].1.parse()?,
                    self.angleaxis[2].1.parse()?,
                    self.angleaxis[3].1.parse()?,
                ));
                na::UnitQuaternion::from_axis_angle(&axis, angle)
            }
            RotationRepr::RotationMatrix => {
                let mut matrix = na::Matrix3::from_iterator(
                    self.rot_matrix
                        .iter()
                        .map(|e| e.parse::<f64>().unwrap_or(0.0)),
                );
                if matrix.rank(0.0001) < 3 {
                    matrix = na::Matrix3::identity();
                }
                na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(&matrix))
            }
        };
        self.quat[0].1 = format!("{:.4}", quat.w);
        self.quat[1].1 = format!("{:.4}", quat.i);
        self.quat[2].1 = format!("{:.4}", quat.j);
        self.quat[3].1 = format!("{:.4}", quat.k);
        if let Some(angleaxis) = quat.axis_angle() {
            self.angleaxis[0].1 = format!("{:.4}", angleaxis.1);
            self.angleaxis[1].1 = format!("{:.4}", angleaxis.0.x);
            self.angleaxis[2].1 = format!("{:.4}", angleaxis.0.y);
            self.angleaxis[3].1 = format!("{:.4}", angleaxis.0.z);
        } else {
            self.angleaxis[0].1 = format!("{:.4}", 0.0);
            self.angleaxis[1].1 = format!("{:.4}", 1.0);
            self.angleaxis[2].1 = format!("{:.4}", 0.0);
            self.angleaxis[3].1 = format!("{:.4}", 0.0);
        }
        quat.to_rotation_matrix()
            .matrix()
            .iter()
            .enumerate()
            .for_each(|(i, &x)| {
                self.rot_matrix[i] = format!("{:.4}", x);
            });

        Ok(())
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        let mut rotation_repr = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading(format!(
                "Rotation tool {}",
                if self.editted { "(Unsync)" } else { "(Sync)" }
            ));
            ui.separator();

            StripBuilder::new(ui)
                .size(Size::initial(0.0))
                .size(Size::initial(0.0))
                .size(Size::initial(0.0))
                .size(Size::initial(0.0))
                .size(Size::initial(0.0))
                .size(Size::initial(0.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        ui.label(egui::RichText::new("Quaternion:").heading());
                        ui.separator();
                    });
                    strip.strip(|strip_builder| {
                        strip_builder
                            .sizes(Size::remainder().at_least(60.0).at_most(100.0), 4)
                            .horizontal(|mut strip| {
                                for quat_e in self.quat.iter_mut() {
                                    strip.cell(|ui| {
                                        ui.label(&quat_e.0);
                                        let text_input_res =
                                            ui.add(egui::TextEdit::singleline(&mut quat_e.1));
                                        if text_input_res.lost_focus()
                                            && ui.input(|input| input.key_pressed(egui::Key::Enter))
                                        {
                                            rotation_repr = Some(RotationRepr::Quaternion);
                                        }
                                        self.editted = text_input_res.changed() || self.editted;
                                    });
                                }
                            });
                    });
                    strip.cell(|ui| {
                        ui.separator();
                        ui.label(egui::RichText::new("Angle-axis:").heading());
                        ui.separator();
                    });
                    strip.strip(|strip_builder| {
                        strip_builder
                            .sizes(Size::remainder().at_least(60.0).at_most(100.0), 4)
                            .horizontal(|mut strip| {
                                for angleaxis_e in self.angleaxis.iter_mut() {
                                    strip.cell(|ui| {
                                        ui.label(&angleaxis_e.0);
                                        let text_input_res =
                                            ui.add(egui::TextEdit::singleline(&mut angleaxis_e.1));
                                        if text_input_res.lost_focus()
                                            && ui.input(|input| input.key_pressed(egui::Key::Enter))
                                        {
                                            rotation_repr = Some(RotationRepr::AngleAxis);
                                        }
                                        self.editted = text_input_res.changed() || self.editted;
                                    });
                                }
                            });
                    });
                    strip.cell(|ui| {
                        ui.separator();
                        ui.label(egui::RichText::new("Rotation matrix:").heading());
                        ui.separator();
                    });
                    strip.strip(|strip_builder| {
                        strip_builder
                            .sizes(Size::remainder().at_least(60.0).at_most(100.0), 3)
                            .horizontal(|mut strip| {
                                for col in 0..3 {
                                    strip.cell(|ui| {
                                        for row in 0..3 {
                                            let text_input_res =
                                                ui.add(egui::TextEdit::singleline(
                                                    &mut self.rot_matrix[3 * col + row],
                                                ));
                                            if text_input_res.lost_focus()
                                                && ui.input(|input| {
                                                    input.key_pressed(egui::Key::Enter)
                                                })
                                            {
                                                rotation_repr = Some(RotationRepr::RotationMatrix);
                                            }
                                            self.editted = text_input_res.changed() || self.editted;
                                        }
                                    });
                                }
                            });
                    });
                });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        if let Some(rotation_repr) = rotation_repr {
            match self.update_input(rotation_repr) {
                Ok(_) => {
                    self.editted = false;
                }
                Err(_) => {}
            }
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
