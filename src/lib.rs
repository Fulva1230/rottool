#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod editor;

pub use app::Rotttol;
use nalgebra as na;
use std::fmt::Display;
use std::ops::Range;
fn render_numbers(text: &str) -> egui::text::LayoutJob {
    let mut layout_job: egui::text::LayoutJob = Default::default();
    let mut rendered = 0;
    for range in split_numbers(text) {
        layout_job.append(
            &text[rendered..range.start],
            0.0,
            egui::TextFormat {
                background: egui::Color32::DARK_GRAY,
                font_id: egui::FontId {
                    family: egui::FontFamily::Monospace,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        layout_job.append(
            &text[range.clone()],
            0.0,
            egui::TextFormat {
                font_id: egui::FontId {
                    family: egui::FontFamily::Monospace,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        rendered = range.end;
    }
    layout_job.append(
        &text[rendered..],
        0.0,
        egui::TextFormat {
            background: egui::Color32::DARK_GRAY,
            font_id: egui::FontId {
                family: egui::FontFamily::Monospace,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    rendered = text.len();
    debug_assert_eq!(rendered, text.len(), "all the text should be rendered");
    layout_job
}
fn split_numbers(s: &str) -> impl Iterator<Item = Range<usize>> {
    let re = regex::regex!(r".*?([+-]?(?:\.\d+|\d+(?:\.\d*)?)(?:[Ee][+-]?\d+)?)");
    re.captures_iter(s).map(|m| m.get(1).expect("must having one capture group").range())
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, strum_macros::EnumIter)]
enum RotRawStringType {
    ColumnMajor4x4,
    RowMajor4x4,
    ColumnMajor3x3,
    RowMajor3x3,
    QuaternionWXYZ,
    QuaternionXYZW,
}
fn format_lines_by_lines(line_iter: impl Iterator<Item = impl IntoIterator<Item = impl Display>>) -> String {
    line_iter.fold(String::default(), |mut acc, col| {
        if !acc.is_empty() {
            acc.push('\n');
        }
        let col_str = col.into_iter().fold(String::default(), |mut col_str, val| {
            if col_str.is_empty() {
                col_str.push_str(&format!("{val}"));
            } else {
                col_str.push_str(&format!(", {val}"));
            }
            col_str
        });
        acc.push_str(&col_str);
        acc
    })
}
fn rotation_to_string(rot: na::UnitQuaternion<f64>, string_type: RotRawStringType) -> String {
    match string_type {
        RotRawStringType::ColumnMajor4x4 => {
            let transform = na::Isometry3::from_parts(na::Translation3::identity(), rot).to_matrix();
            format_lines_by_lines(transform.column_iter())
        }
        RotRawStringType::RowMajor4x4 => {
            let transform = na::Isometry3::from_parts(na::Translation3::identity(), rot).to_matrix();
            format_lines_by_lines(transform.row_iter())
        }
        RotRawStringType::ColumnMajor3x3 => {
            let rot_matrix = rot.to_rotation_matrix();
            format_lines_by_lines(rot_matrix.matrix().column_iter())
        }
        RotRawStringType::RowMajor3x3 => {
            let rot_matrix = rot.to_rotation_matrix();
            format_lines_by_lines(rot_matrix.matrix().row_iter())
        }
        RotRawStringType::QuaternionWXYZ => {
            format!("{}, {}, {}, {}", rot.w, rot.i, rot.j, rot.k)
        }
        RotRawStringType::QuaternionXYZW => {
            format!("{}, {}, {}, {}", rot.i, rot.j, rot.k, rot.w)
        }
    }
}
enum RotationEditorResponse {
    TriggerSync(na::UnitQuaternion<f64>),
    Edited,
}
trait RotationEditor {
    fn import(&mut self, rot: na::UnitQuaternion<f64>);
    fn export(&self) -> anyhow::Result<na::UnitQuaternion<f64>>;
    fn ui(&mut self, ui: &mut egui::Ui, edited: bool) -> Vec<RotationEditorResponse>;
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        let test_str = "1312.3    413.423,,,,, 5234534 ,-2.0,  -0.2 fwefawe, 1234";
        assert_eq!(
            split_numbers("1312.3    413.423,,,,, 5234534 ,-2.0,  -0.2 fwefawe, 1234")
                .map(|range| test_str[range].parse::<f32>().expect("the capture substring must be parseable"))
                .collect::<Vec<f32>>(),
            vec![1312.3, 413.423, 5234534.0, -2.0, -0.2, 1234.0]
        );
    }
}
