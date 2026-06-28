use eframe::Frame;
use egui::Ui;
use nalgebra as na;
use strum::IntoEnumIterator;

enum RotationRepr {
    Quaternion,
    AngleAxis,
    RotationMatrix,
    RawString,
}
#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, strum_macros::EnumIter,
)]
enum RawStringType {
    ColumnMajor4x4,
    RowMajor4x4,
    ColumnMajor3x3,
    RowMajor3x3,
    QuaternionWXYZ,
    QuaternionXYZW,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    quat: [(String, String); 4],
    angleaxis: [(String, String); 4],
    rot_matrix: [String; 9],
    raw_string: String,
    raw_string_type: RawStringType,
    editted: bool,
    footer_height: f32,
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
            raw_string: String::new(),
            raw_string_type: RawStringType::ColumnMajor4x4,
            editted: false,
            footer_height: 0.0,
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

    fn update_input(&mut self, edited_item: &RotationRepr) -> anyhow::Result<()> {
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
            RotationRepr::RawString => {
                let nums = super::split_numbers(&self.raw_string);
                match self.raw_string_type {
                    RawStringType::ColumnMajor4x4 => {
                        if nums.len() == 16 {
                            let transform_mat = na::Matrix4::from_column_slice(&nums);
                            let mut rot = na::Matrix3::identity();
                            rot.copy_from(&transform_mat.fixed_view::<3, 3>(0, 0));
                            na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(
                                &rot,
                            ))
                        } else {
                            anyhow::bail!("len wrong");
                        }
                    }
                    RawStringType::RowMajor4x4 => {
                        if nums.len() == 16 {
                            let transform_mat = na::Matrix4::from_row_slice(&nums);
                            let mut rot = na::Matrix3::identity();
                            rot.copy_from(&transform_mat.fixed_view::<3, 3>(0, 0));
                            na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(
                                &rot,
                            ))
                        } else {
                            anyhow::bail!("len wrong");
                        }
                    }
                    RawStringType::ColumnMajor3x3 => {
                        if nums.len() == 9 {
                            na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(
                                &na::Matrix3::from_column_slice(&nums),
                            ))
                        } else {
                            anyhow::bail!("len wrong");
                        }
                    }
                    RawStringType::RowMajor3x3 => {
                        if nums.len() == 9 {
                            log::log!(log::Level::Debug, "{:?}", nums);
                            na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(
                                &na::Matrix3::from_row_slice(&nums),
                            ))
                        } else {
                            anyhow::bail!("len wrong");
                        }
                    }
                    RawStringType::QuaternionWXYZ => {
                        if nums.len() == 4 {
                            na::UnitQuaternion::from_quaternion(na::Quaternion::new(
                                nums[0], nums[1], nums[2], nums[3],
                            ))
                        } else {
                            anyhow::bail!("len wrong");
                        }
                    }
                    RawStringType::QuaternionXYZW => {
                        if nums.len() == 4 {
                            na::UnitQuaternion::from_quaternion(na::Quaternion::from_vector(
                                na::Vector4::from_column_slice(&nums),
                            ))
                        } else {
                            anyhow::bail!("len wrong");
                        }
                    }
                }
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
                *self.rot_matrix.get_mut(i).expect("failed access") = format!("{x:.4}");
            });

        Ok(())
    }

    fn quaternion_view(
        &mut self,
        strip_builder: egui_extras::StripBuilder<'_>,
        edited_item: &mut Option<RotationRepr>,
    ) {
        strip_builder
            .sizes(egui_extras::Size::remainder().at_least(60.0).at_most(100.0), 4)
            .horizontal(|mut strip| {
                for quat_e in &mut self.quat {
                    strip.cell(|ui| {
                        ui.label(&quat_e.0);
                        let text_input_res = ui.add(egui::TextEdit::singleline(&mut quat_e.1));
                        if text_input_res.lost_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter))
                        {
                            *edited_item = Some(RotationRepr::Quaternion);
                        }
                        self.editted = text_input_res.changed() || self.editted;
                    });
                }
            });
    }

    fn angleaxis_view(
        &mut self,
        strip_builder: egui_extras::StripBuilder<'_>,
        edited_item: &mut Option<RotationRepr>,
    ) {
        strip_builder
            .sizes(egui_extras::Size::remainder().at_least(60.0).at_most(100.0), 4)
            .horizontal(|mut strip| {
                for angleaxis_e in &mut self.angleaxis {
                    strip.cell(|ui| {
                        ui.label(&angleaxis_e.0);
                        let text_input_res = ui.add(egui::TextEdit::singleline(&mut angleaxis_e.1));
                        if text_input_res.lost_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter))
                        {
                            *edited_item = Some(RotationRepr::AngleAxis);
                        }
                        self.editted = text_input_res.changed() || self.editted;
                    });
                }
            });
    }

    fn rotation_matrix_view(
        &mut self,
        strip_builder: egui_extras::StripBuilder<'_>,
        edited_item: &mut Option<RotationRepr>,
    ) {
        strip_builder
            .sizes(egui_extras::Size::remainder().at_least(60.0).at_most(100.0), 3)
            .horizontal(|mut strip| {
                for col in 0..3 {
                    strip.cell(|ui| {
                        for row in 0..3 {
                            let text_input_res = ui.add(egui::TextEdit::singleline(
                                self.rot_matrix
                                    .get_mut(3 * col + row)
                                    .expect("out of bounds"),
                            ));
                            if text_input_res.lost_focus()
                                && ui.input(|input| input.key_pressed(egui::Key::Enter))
                            {
                                *edited_item = Some(RotationRepr::RotationMatrix);
                            }
                            self.editted = text_input_res.changed() || self.editted;
                        }
                    });
                }
            });
    }

    fn raw_string_access(&mut self, ui: &mut egui::Ui, edited_item: &mut Option<RotationRepr>) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui.button("import").clicked() {
                    *edited_item = Some(RotationRepr::RawString);
                }
                egui::ComboBox::from_label("type")
                    .selected_text(format!("{:?}", self.raw_string_type))
                    .show_ui(ui, |ui| {
                        for string_type in RawStringType::iter() {
                            ui.selectable_value(
                                &mut self.raw_string_type,
                                string_type,
                                format!("{:?}", string_type),
                            );
                        }
                    })
            });
            let text_input_res = ui.add_sized(
                [ui.available_size_before_wrap().x, 150.0],
                egui::TextEdit::multiline(&mut self.raw_string).layouter(&mut |ui, text, wrap_width| {
                    ui.fonts_mut(|f| f.layout_job(crate::render_numbers(text.as_str())))
                }),
            );
            self.editted = text_input_res.changed() || self.editted;
        });
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        let mut rotation_repr = None;

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(format!(
                    "Rotation tool {}",
                    if self.editted { "(Unsync)" } else { "(Sync)" }
                ));
                ui.separator();
                ui.label(egui::RichText::new("Quaternion:").heading());
                ui.separator();
                ui.allocate_ui_with_layout([ui.available_size_before_wrap().x, 0.0].into(), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    self.quaternion_view(egui_extras::StripBuilder::new(ui), &mut rotation_repr);
                });
                ui.separator();
                ui.label(egui::RichText::new("Angle-axis:").heading());
                ui.separator();
                ui.allocate_ui_with_layout([ui.available_size_before_wrap().x, 0.0].into(), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    self.angleaxis_view(egui_extras::StripBuilder::new(ui), &mut rotation_repr);
                });
                ui.separator();
                ui.label(egui::RichText::new("Rotation matrix:").heading());
                ui.separator();
                ui.allocate_ui_with_layout([ui.available_size_before_wrap().x, 0.0].into(), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    self.rotation_matrix_view(egui_extras::StripBuilder::new(ui), &mut rotation_repr);
                });
                ui.separator();
                self.raw_string_access(ui, &mut rotation_repr);
                if ui.available_height() > self.footer_height {
                    self.footer_height = ui
                        .with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                            powered_by_egui_and_eframe(ui);
                            egui::warn_if_debug_build(ui);
                        })
                        .response
                        .rect
                        .height();
                } else {
                    self.footer_height = ui
                        .with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            egui::warn_if_debug_build(ui);
                            powered_by_egui_and_eframe(ui);
                        })
                        .response
                        .rect
                        .height();
                }
            });
        });

        if let Some(rotation_repr) = rotation_repr
            && self.update_input(&rotation_repr).is_ok()
        {
            self.editted = false;
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
