use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
};

use tracing::warn;

use crate::{
    app::App,
    task::{LoadSliced, MeshLoad, ProjectLoad},
};

pub struct Args {
    pub open: OpenInto,
    pub install: bool,
}

#[derive(Default)]
pub struct OpenInto {
    project: Option<PathBuf>,
    sliced: Option<PathBuf>,
    models: Vec<(String, PathBuf)>,
}

impl Args {
    pub fn parse() -> Self {
        let mut open = OpenInto::default();
        let mut install = false;

        for arg in env::args().skip(1) {
            if let Some(flag) = arg.strip_prefix("--") {
                match flag {
                    "install" => install = true,
                    x => warn!("Unknown flag `{x}`, ignoring"),
                }
            }

            open.insert(Path::new(&arg).to_path_buf());
        }

        Args { open, install }
    }
}

impl OpenInto {
    pub fn insert(&mut self, path: PathBuf) {
        let Some(ext) = path.extension() else { return };
        let ext = ext.to_string_lossy();

        match ext.to_ascii_lowercase().as_str() {
            "mslicer" if self.project.is_none() => self.project = Some(path),
            "goo" | "ctb" | "nanodlp" if self.sliced.is_none() => self.sliced = Some(path),
            "stl" | "obj" => self.models.push((ext.into_owned(), path)),
            _ => {}
        }
    }

    pub fn start(self, app: &mut App) {
        if let Some(path) = self.project {
            app.tasks.add(ProjectLoad::new(path));
        }

        if let Some(path) = self.sliced {
            app.tasks.add(LoadSliced::new(path));
        }

        for (ext, path) in self.models {
            let file = File::open(&path).unwrap();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            app.tasks.add(MeshLoad::file(file, name, ext));
        }
    }
}
