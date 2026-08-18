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

            let Some((_, ext)) = arg.rsplit_once('.') else {
                continue;
            };

            let path = Path::new(&arg).to_path_buf();
            match ext.to_ascii_lowercase().as_str() {
                "mslicer" if open.project.is_none() => open.project = Some(path),
                "goo" | "ctb" | "nanodlp" if open.sliced.is_none() => open.sliced = Some(path),
                "stl" | "obj" => open.models.push((ext.to_owned(), path)),
                _ => continue,
            }
        }

        Args { open, install }
    }
}

impl OpenInto {
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
