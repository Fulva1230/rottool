use crate::{RotationEditor, RotationEditorResponse, editor, rotation_to_string};
use eframe::Frame;
use nalgebra as na;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Rotttol {
    rot: na::UnitQuaternion<f64>,
    quat: editor::QuaternionEditor,
    angle_axis: editor::AngleAxisEditor,
    rot_matrix: editor::RotMatrixEditor,
    raw_string: editor::RawStringEditor,
    edited: bool,
    footer_height: f32,
}

impl Default for Rotttol {
    fn default() -> Self {
        Self {
            rot: na::UnitQuaternion::identity(),
            quat: Default::default(),
            angle_axis: Default::default(),
            rot_matrix: Default::default(),
            raw_string: Default::default(),
            edited: false,
            footer_height: 0.0,
        }
    }
}

impl Rotttol {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "JetbrainsMono".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/JetBrainsMono-Regular.ttf")).into(),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .expect("No monospace font family")
            .insert(0, "JetbrainsMono".to_owned());
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.all_styles_mut(|style| {
            style
                .text_styles
                .get_mut(&egui::TextStyle::Body)
                .expect("No body text style component")
                .family = egui::FontFamily::Monospace;
        });

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
    fn ui_rotation_editor(ui: &mut egui::Ui, editor: &mut impl RotationEditor, edited: &mut bool, sync_rot: &mut Option<na::UnitQuaternion<f64>>) {
        for response in editor.ui(ui, *edited) {
            match response {
                RotationEditorResponse::TriggerSync(rot) => {
                    *sync_rot = Some(rot);
                }
                RotationEditorResponse::Edited => {
                    *edited = true;
                }
            }
        }
    }
}

impl eframe::App for Rotttol {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Frame) {
        egui::Panel::top("top_panel").show(ui, |ui| {
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
        let mut sync_rot = None;
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(format!("Rotation tool {}", if self.edited { "(Unsync)" } else { "(Sync)" }));
                ui.separator();
                ui.label(egui::RichText::new("Quaternion:").heading());
                ui.separator();
                ui.allocate_ui_with_layout(
                    [ui.available_size_before_wrap().x, 0.0].into(),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        Self::ui_rotation_editor(ui, &mut self.quat, &mut self.edited, &mut sync_rot);
                    },
                );
                ui.separator();
                ui.label(egui::RichText::new("Angle-axis:").heading());
                ui.separator();
                ui.allocate_ui_with_layout(
                    [ui.available_size_before_wrap().x, 0.0].into(),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        Self::ui_rotation_editor(ui, &mut self.angle_axis, &mut self.edited, &mut sync_rot);
                    },
                );
                ui.separator();
                ui.label(egui::RichText::new("Rotation matrix:").heading());
                ui.separator();
                ui.allocate_ui_with_layout(
                    [ui.available_size_before_wrap().x, 0.0].into(),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        Self::ui_rotation_editor(ui, &mut self.rot_matrix, &mut self.edited, &mut sync_rot);
                    },
                );
                ui.separator();
                ui.allocate_ui_with_layout(
                    [ui.available_size_before_wrap().x, 0.0].into(),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        Self::ui_rotation_editor(ui, &mut self.raw_string, &mut self.edited, &mut sync_rot);
                    },
                );
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
        if let Some(rot) = sync_rot {
            self.quat.import(rot);
            self.angle_axis.import(rot);
            self.rot_matrix.import(rot);
            self.raw_string.import(rot);
            self.edited = false;
        }
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/crates/eframe");
        ui.label(".");
    });
}
