use bevy::prelude::Color;
use bevy_egui::egui::Color32;
pub const MY_ACCENT_COLOR: Color = Color::Rgba {
    red: 0.901,
    green: 0.4,
    blue: 0.01,
    alpha: 1.0,
};
pub const MY_ACCENT_COLOR32: Color32 = Color32::from_rgb(230, 102, 1);

pub const GIT_HASH: &str = env!("GIT_HASH");
pub const GIT_DATE: &str = env!("GIT_DATE");

pub const BASE_FONT: &str = "fonts/Alagard.ttf";
