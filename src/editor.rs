use super::{RotRawStringType, RotationEditor, RotationEditorResponse, na, rotation_to_string, split_numbers};
use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator as _;

#[derive(Serialize, Deserialize)]
pub struct QuaternionEditor {
    quat: [(String, String); 4],
}
impl Default for QuaternionEditor {
    fn default() -> Self {
        Self {
            quat: [
                ("Qw".to_owned(), "1.0".to_owned()),
                ("Qx".to_owned(), "0.0".to_owned()),
                ("Qy".to_owned(), "0.0".to_owned()),
                ("Qz".to_owned(), "0.0".to_owned()),
            ],
        }
    }
}
impl RotationEditor for QuaternionEditor {
    fn import(&mut self, rot: na::UnitQuaternion<f64>) {
        self.quat[0].1 = format!("{:.4}", rot.w);
        self.quat[1].1 = format!("{:.4}", rot.i);
        self.quat[2].1 = format!("{:.4}", rot.j);
        self.quat[3].1 = format!("{:.4}", rot.k);
    }

    fn export(&self) -> anyhow::Result<na::UnitQuaternion<f64>> {
        Ok(na::UnitQuaternion::<f64>::from_quaternion(na::Quaternion::new(
            self.quat[0].1.parse()?,
            self.quat[1].1.parse()?,
            self.quat[2].1.parse()?,
            self.quat[3].1.parse()?,
        )))
    }

    fn ui(&mut self, ui: &mut egui::Ui, _edited: bool) -> Vec<RotationEditorResponse> {
        let mut ret = vec![];
        let strip_builder = egui_extras::StripBuilder::new(ui);
        let mut trigger_sync = false;
        strip_builder
            .sizes(egui_extras::Size::remainder().at_least(60.0).at_most(100.0), 4)
            .horizontal(|mut strip| {
                for quat_e in &mut self.quat {
                    strip.cell(|ui| {
                        ui.label(&quat_e.0);
                        let text_input_res = ui.add(egui::TextEdit::singleline(&mut quat_e.1));
                        if text_input_res.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                            trigger_sync = true;
                        }
                        if text_input_res.changed() {
                            ret.push(RotationEditorResponse::Edited);
                        }
                    });
                }
            });
        if trigger_sync && let Ok(rot) = self.export() {
            ret.push(RotationEditorResponse::TriggerSync(rot));
        }
        ret
    }
}
#[derive(Serialize, Deserialize)]
pub struct RotMatrixEditor {
    rot_matrix: [String; 9],
}
impl Default for RotMatrixEditor {
    fn default() -> Self {
        Self {
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
        }
    }
}
impl RotationEditor for RotMatrixEditor {
    fn import(&mut self, rot: na::UnitQuaternion<f64>) {
        rot.to_rotation_matrix().matrix().iter().enumerate().for_each(|(i, &x)| {
            *self.rot_matrix.get_mut(i).expect("failed access") = format!("{x:.4}");
        });
    }
    fn export(&self) -> anyhow::Result<na::UnitQuaternion<f64>> {
        let mut matrix = na::Matrix3::from_iterator(self.rot_matrix.iter().map(|e| e.parse::<f64>().unwrap_or(0.0)));
        if matrix.rank(0.0001) < 3 {
            matrix = na::Matrix3::identity();
        }
        Ok(na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(&matrix)))
    }
    fn ui(&mut self, ui: &mut egui::Ui, _edited: bool) -> Vec<RotationEditorResponse> {
        let mut ret = vec![];
        let strip_builder = egui_extras::StripBuilder::new(ui);
        let mut trigger_sync = false;
        strip_builder
            .sizes(egui_extras::Size::remainder().at_least(60.0).at_most(100.0), 3)
            .horizontal(|mut strip| {
                for col in 0..3 {
                    strip.cell(|ui| {
                        for row in 0..3 {
                            let text_input_res = ui.add(egui::TextEdit::singleline(self.rot_matrix.get_mut(3 * col + row).expect("out of bounds")));
                            if text_input_res.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                                trigger_sync = true;
                            }
                            if text_input_res.changed() {
                                ret.push(RotationEditorResponse::Edited);
                            }
                        }
                    });
                }
            });
        if trigger_sync && let Ok(rot) = self.export() {
            ret.push(RotationEditorResponse::TriggerSync(rot));
        }
        ret
    }
}
#[derive(Serialize, Deserialize)]
pub struct AngleAxisEditor {
    angleaxis: [(String, String); 4],
}
impl Default for AngleAxisEditor {
    fn default() -> Self {
        Self {
            angleaxis: [
                ("Ang (rad)".to_owned(), "0.0".to_owned()),
                ("AxisX".to_owned(), "1.0".to_owned()),
                ("AxisY".to_owned(), "0.0".to_owned()),
                ("AxisZ".to_owned(), "0.0".to_owned()),
            ],
        }
    }
}
impl RotationEditor for AngleAxisEditor {
    fn import(&mut self, rot: na::UnitQuaternion<f64>) {
        if let Some(angleaxis) = rot.axis_angle() {
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
    }
    fn export(&self) -> anyhow::Result<na::UnitQuaternion<f64>> {
        let angle = self.angleaxis[0].1.parse()?;
        let axis = na::UnitVector3::new_normalize(na::Vector3::new(
            self.angleaxis[1].1.parse()?,
            self.angleaxis[2].1.parse()?,
            self.angleaxis[3].1.parse()?,
        ));
        Ok(na::UnitQuaternion::from_axis_angle(&axis, angle))
    }
    fn ui(&mut self, ui: &mut egui::Ui, _edited: bool) -> Vec<RotationEditorResponse> {
        let mut ret = vec![];
        let strip_builder = egui_extras::StripBuilder::new(ui);
        let mut trigger_sync = false;
        strip_builder
            .sizes(egui_extras::Size::remainder().at_least(60.0).at_most(100.0), 4)
            .horizontal(|mut strip| {
                for angleaxis_e in &mut self.angleaxis {
                    strip.cell(|ui| {
                        ui.label(&angleaxis_e.0);
                        let text_input_res = ui.add(egui::TextEdit::singleline(&mut angleaxis_e.1));
                        if text_input_res.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                            trigger_sync = true;
                        }
                        if text_input_res.changed() {
                            ret.push(RotationEditorResponse::Edited);
                        }
                    });
                }
            });
        if trigger_sync && let Ok(rot) = self.export() {
            ret.push(RotationEditorResponse::TriggerSync(rot));
        }
        ret
    }
}
#[derive(Serialize, Deserialize)]
pub struct RawStringEditor {
    rot: na::UnitQuaternion<f64>,
    raw_string: String,
    raw_string_type: RotRawStringType,
}
impl Default for RawStringEditor {
    fn default() -> Self {
        Self {
            raw_string: String::new(),
            raw_string_type: RotRawStringType::ColumnMajor4x4,
            rot: na::UnitQuaternion::identity(),
        }
    }
}
impl RotationEditor for RawStringEditor {
    fn import(&mut self, rot: na::UnitQuaternion<f64>) {
        self.rot = rot;
    }

    fn export(&self) -> anyhow::Result<na::UnitQuaternion<f64>> {
        let nums = split_numbers(&self.raw_string)
            .map(|range| self.raw_string[range].parse().expect("the captured substring must be parsable"))
            .collect::<Vec<_>>();
        Ok(match self.raw_string_type {
            RotRawStringType::ColumnMajor4x4 => {
                if nums.len() == 16 {
                    let transform_mat = na::Matrix4::from_column_slice(&nums);
                    let mut rot = na::Matrix3::identity();
                    rot.copy_from(&transform_mat.fixed_view::<3, 3>(0, 0));
                    na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(&rot))
                } else {
                    anyhow::bail!("len wrong");
                }
            }
            RotRawStringType::RowMajor4x4 => {
                if nums.len() == 16 {
                    let transform_mat = na::Matrix4::from_row_slice(&nums);
                    let mut rot = na::Matrix3::identity();
                    rot.copy_from(&transform_mat.fixed_view::<3, 3>(0, 0));
                    na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(&rot))
                } else {
                    anyhow::bail!("len wrong");
                }
            }
            RotRawStringType::ColumnMajor3x3 => {
                if nums.len() == 9 {
                    na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(&na::Matrix3::from_column_slice(&nums)))
                } else {
                    anyhow::bail!("len wrong");
                }
            }
            RotRawStringType::RowMajor3x3 => {
                if nums.len() == 9 {
                    na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(&na::Matrix3::from_row_slice(&nums)))
                } else {
                    anyhow::bail!("len wrong");
                }
            }
            RotRawStringType::QuaternionWXYZ => {
                if nums.len() == 4 {
                    let acc = |nums: &[f64], idx| -> anyhow::Result<f64> { nums.get(idx).copied().context("access shouldn't failed") };
                    na::UnitQuaternion::from_quaternion(na::Quaternion::new(acc(&nums, 0)?, acc(&nums, 1)?, acc(&nums, 2)?, acc(&nums, 3)?))
                } else {
                    anyhow::bail!("len wrong");
                }
            }
            RotRawStringType::QuaternionXYZW => {
                if nums.len() == 4 {
                    na::UnitQuaternion::from_quaternion(na::Quaternion::from_vector(na::Vector4::from_column_slice(&nums)))
                } else {
                    anyhow::bail!("len wrong");
                }
            }
        })
    }

    fn ui(&mut self, ui: &mut egui::Ui, edited: bool) -> Vec<RotationEditorResponse> {
        let mut ret = vec![];
        let mut trigger_sync = false;
        ui.horizontal(|ui| {
            if ui.button("Import").clicked() {
                trigger_sync = true;
            }
            if ui.button("Export").clicked() && !edited {
                self.raw_string = rotation_to_string(self.rot, self.raw_string_type);
            }
            egui::ComboBox::from_label("type")
                .selected_text(format!("{:?}", self.raw_string_type))
                .show_ui(ui, |ui| {
                    for string_type in RotRawStringType::iter() {
                        ui.selectable_value(&mut self.raw_string_type, string_type, format!("{string_type:?}"));
                    }
                })
        });
        let text_input_res = ui.add_sized(
            [ui.available_size_before_wrap().x, 150.0],
            egui::TextEdit::multiline(&mut self.raw_string)
                .layouter(&mut |ui, text, _wrap_width| ui.fonts_mut(|f| f.layout_job(crate::render_numbers(text.as_str())))),
        );
        if text_input_res.changed() {
            ret.push(RotationEditorResponse::Edited);
        }
        if trigger_sync && let Ok(rot) = self.export() {
            ret.push(RotationEditorResponse::TriggerSync(rot));
        }
        ret
    }
}
