#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;

fn split_numbers(s: &str) -> Vec<f64> {
    let re = regex::Regex::new(r".*?(-?\d+\.?\d*)").unwrap();
    re.captures_iter(s).map(|c| c[1].parse().unwrap()).collect::<Vec<f64>>()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        assert_eq!(split_numbers("1312.3    413.423,,,,, 5234534 ,-2.0,  -0.2 fwefawe, 1234"), vec![1312.3, 413.423, 5234534.0, -2.0, -0.2, 1234.0]);
    }
}