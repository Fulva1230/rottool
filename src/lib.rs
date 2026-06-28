#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;
fn render_numbers(text: &str) -> egui::text::LayoutJob {
    let mut layout_job: egui::text::LayoutJob = Default::default();
    let re = regex::Regex::new(r"-?\d+\.?\d*").expect("Failed to compile regex");
    let mut rendered = 0;
    for match_e in re.find_iter(text) {
        layout_job.append(&text[rendered..match_e.start()], 0.0, egui::TextFormat {
            background: egui::Color32::GRAY,
            ..Default::default()
        });
        layout_job.append(&text[match_e.start()..match_e.end()], 0.0, egui::TextFormat {
            ..Default::default()
        });
        rendered = match_e.end();
    };
    layout_job.append(&text[rendered..], 0.0, egui::TextFormat {
        background: egui::Color32::GRAY,
        ..Default::default()
    });
    rendered = text.len();
    debug_assert_eq!(rendered, text.len());
    layout_job
}

fn split_numbers(s: &str) -> Vec<f64> {
    let re = regex::Regex::new(r"-?\d+\.?\d*").expect("Failed to compile regex");
    re.captures_iter(s).map(|c| c[0].parse().unwrap_or_default()).collect::<Vec<f64>>()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        assert_eq!(split_numbers("1312.3    413.423,,,,, 5234534 ,-2.0,  -0.2 fwefawe, 1234"), vec![1312.3, 413.423, 5234534.0, -2.0, -0.2, 1234.0]);
    }
}