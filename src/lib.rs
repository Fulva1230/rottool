#![warn(clippy::all, rust_2018_idioms)]

mod app;

use std::ops::Range;
pub use app::TemplateApp;
fn render_numbers(text: &str) -> egui::text::LayoutJob {
    let mut layout_job: egui::text::LayoutJob = Default::default();
    let split_ranges = split_numbers(text);
    let mut rendered = 0;
    for range in split_ranges {
        layout_job.append(&text[rendered..range.start], 0.0, egui::TextFormat {
            background: egui::Color32::GRAY,
            ..Default::default()
        });
        layout_job.append(&text[range.start..range.end], 0.0, egui::TextFormat {
            ..Default::default()
        });
        rendered = range.end;
    };
    layout_job.append(&text[rendered..], 0.0, egui::TextFormat {
        background: egui::Color32::GRAY,
        ..Default::default()
    });
    rendered = text.len();
    debug_assert_eq!(rendered, text.len());
    layout_job
}

fn split_numbers(s: &str) -> Vec<Range<usize>> {
    let re = regex::regex!(r".*?([+-]?(?:\.\d+|\d+(?:\.\d*)?)(?:[Ee][+-]?\d+)?)");
    re.captures_iter(s).map(|m| {
        m.get(1).unwrap().range()
    }).collect()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        let test_str = "1312.3    413.423,,,,, 5234534 ,-2.0,  -0.2 fwefawe, 1234";
        assert_eq!(split_numbers("1312.3    413.423,,,,, 5234534 ,-2.0,  -0.2 fwefawe, 1234")
                       .into_iter().map(|range| test_str[range].parse::<f32>().unwrap()).collect::<Vec<f32>>(), vec![1312.3, 413.423, 5234534.0, -2.0, -0.2, 1234.0]);
    }
}