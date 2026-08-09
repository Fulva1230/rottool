#![warn(clippy::all, rust_2018_idioms)]

mod app;

pub use app::Rotttol;
use nalgebra as na;
use std::ops::Range;
fn render_numbers(text: &str) -> egui::text::LayoutJob {
    let mut layout_job: egui::text::LayoutJob = Default::default();
    let mut rendered = 0;
    for range in split_numbers(text) {
        layout_job.append(
            &text[rendered..range.start],
            0.0,
            egui::TextFormat {
                background: egui::Color32::GRAY,
                ..Default::default()
            },
        );
        layout_job.append(
            &text[range.start..range.end],
            0.0,
            egui::TextFormat {
                ..Default::default()
            },
        );
        rendered = range.end;
    }
    layout_job.append(
        &text[rendered..],
        0.0,
        egui::TextFormat {
            background: egui::Color32::GRAY,
            ..Default::default()
        },
    );
    rendered = text.len();
    debug_assert_eq!(rendered, text.len());
    layout_job
}

fn split_numbers(s: &str) -> impl Iterator<Item = Range<usize>> {
    let re = regex::regex!(r".*?([+-]?(?:\.\d+|\d+(?:\.\d*)?)(?:[Ee][+-]?\d+)?)");
    re.captures_iter(s).map(|m| m.get(1).unwrap().range())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        let test_str = "1312.3    413.423,,,,, 5234534 ,-2.0,  -0.2 fwefawe, 1234";
        assert_eq!(
            split_numbers("1312.3    413.423,,,,, 5234534 ,-2.0,  -0.2 fwefawe, 1234")
                .map(|range| test_str[range].parse::<f32>().unwrap())
                .collect::<Vec<f32>>(),
            vec![1312.3, 413.423, 5234534.0, -2.0, -0.2, 1234.0]
        );
    }
}

enum RotationRepr {
    Quaternion,
    AngleAxis,
    RotationMatrix,
    RawString,
}

#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, strum_macros::EnumIter,
)]
enum RotRawStringType {
    ColumnMajor4x4,
    RowMajor4x4,
    ColumnMajor3x3,
    RowMajor3x3,
    QuaternionWXYZ,
    QuaternionXYZW,
}
fn rotation_to_string(rot: na::UnitQuaternion<f64>, string_type: RotRawStringType) -> String {
    match string_type {
        RotRawStringType::ColumnMajor4x4 => {
            let transform =
                na::Isometry3::from_parts(na::Translation3::identity(), rot).to_matrix();
            transform.column_iter().fold(String::default(), |mut acc, col| {
                if !acc.is_empty() {
                    acc.push_str("\n");
                }
                let col_str = col.iter().fold(String::default(), |col_str, val| {
                    if col_str.is_empty() {
                        col_str + &format!("{}", val)
                    } else {
                        col_str + &format!(", {}", val)
                    }
                });
                acc.push_str(&col_str);
                acc
            })
        }
        RotRawStringType::RowMajor4x4 => {
            let transform =
                na::Isometry3::from_parts(na::Translation3::identity(), rot).to_matrix();
            transform.row_iter().fold(String::default(), |mut acc, row| {
                if !acc.is_empty() {
                    acc.push_str("\n");
                }
                let row_str = row.iter().fold(String::default(), |row_str, val| {
                    if row_str.is_empty() {
                        row_str + &format!("{}", val)
                    } else {
                        row_str + &format!(", {}", val)
                    }
                });
                acc.push_str(&row_str);
                acc
            })
        }
        RotRawStringType::ColumnMajor3x3 => {
            let rot_matrix = rot.to_rotation_matrix();
            rot_matrix.matrix().column_iter().fold(String::default(), |mut acc, col| {
                if !acc.is_empty() {
                    acc.push_str("\n");
                }
                let col_str = col.iter().fold(String::default(), |col_str, val| {
                    if col_str.is_empty() {
                        col_str + &format!("{}", val)
                    } else {
                        col_str + &format!(", {}", val)
                    }
                });
                acc.push_str(&col_str);
                acc
            })
        }
        RotRawStringType::RowMajor3x3 => {
            let rot_matrix = rot.to_rotation_matrix();
            rot_matrix.matrix().row_iter().fold(String::default(), |mut acc, row| {
                if !acc.is_empty() {
                    acc.push_str("\n");
                }
                let row_str = row.iter().fold(String::default(), |row_str, val| {
                    if row_str.is_empty() {
                        row_str + &format!("{}", val)
                    } else {
                        row_str + &format!(", {}", val)
                    }
                });
                acc.push_str(&row_str);
                acc
            })
        }
        RotRawStringType::QuaternionWXYZ => {
            format!("{}, {}, {}, {}", rot.w, rot.i, rot.j, rot.k)
        }
        RotRawStringType::QuaternionXYZW => {
            format!("{}, {}, {}, {}", rot.i, rot.j, rot.k, rot.w)
        }
    }
}
