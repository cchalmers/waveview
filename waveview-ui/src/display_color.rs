use eframe::egui;
use waveview_model::viewer::DisplayColor;

pub fn resolve(color: DisplayColor, fallback: egui::Color32) -> egui::Color32 {
    match color {
        DisplayColor::Default => fallback,
        DisplayColor::Red => egui::Color32::from_rgb(224, 108, 117),
        DisplayColor::Orange => egui::Color32::from_rgb(209, 154, 102),
        DisplayColor::Yellow => egui::Color32::from_rgb(229, 192, 123),
        DisplayColor::Green => egui::Color32::from_rgb(152, 195, 121),
        DisplayColor::Cyan => egui::Color32::from_rgb(86, 182, 194),
        DisplayColor::Blue => egui::Color32::from_rgb(97, 175, 239),
        DisplayColor::Purple => egui::Color32::from_rgb(198, 120, 221),
        DisplayColor::Gray => egui::Color32::from_gray(150),
    }
}
