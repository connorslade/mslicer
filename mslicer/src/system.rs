use std::{env, fs::File, path::PathBuf, str::FromStr};

use egui::IconData;

#[cfg(not(target_os = "macos"))]
use crate::include_dist;
use crate::{
    app::App,
    task::{LoadSliced, MeshLoad, ProjectLoad},
};

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

pub fn open_arguments(app: &mut App) {
    let mut project = None;
    let mut sliced = None;
    let mut mesh = Vec::new();

    for path in env::args().skip(1) {
        let Some((_, ext)) = path.rsplit_once('.') else {
            continue;
        };

        match ext.to_ascii_lowercase().as_str() {
            "mslicer" if project.is_none() => project = Some(path),
            "goo" | "ctb" | "nanodlp" if sliced.is_none() => sliced = Some(path),
            "stl" | "obj" => mesh.push((ext.to_owned(), path)),
            _ => continue,
        }
    }

    if let Some(project) = project {
        let path = PathBuf::from_str(&project).unwrap();
        app.tasks.add(ProjectLoad::new(path));
    }

    if let Some(sliced) = sliced {
        let path = PathBuf::from_str(&sliced).unwrap();
        app.tasks.add(LoadSliced::new(path));
    }

    for (ext, mesh) in mesh {
        let path = PathBuf::from_str(&mesh).unwrap();
        let file = File::open(&path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        app.tasks.add(MeshLoad::file(file, name, ext));
    }
}
