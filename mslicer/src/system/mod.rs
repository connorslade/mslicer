use egui::IconData;

pub mod arguments;
#[cfg(windows)]
pub mod windows;

#[cfg(not(target_os = "macos"))]
use crate::include_dist;

// On MacOS the icons are loaded automacally from the icon.icns file.
#[cfg(target_os = "macos")]
pub fn icon() -> IconData {
    IconData::default()
}

#[cfg(not(target_os = "macos"))]
pub fn icon() -> IconData {
    let icon = image::load_from_memory(include_dist!("icon.png")).unwrap();
    IconData {
        rgba: icon.to_rgba8().to_vec(),
        width: icon.width(),
        height: icon.height(),
    }
}
