use std::{
    cell::Cell,
    cmp::Ordering,
    env::consts::{ARCH, OS},
};

use chrono::Utc;
use egui::{Grid, RichText};
use serde::Deserialize;
use tracing::info;

use crate::{
    VERSION,
    app::{App, config::ui::UpdateCheckFrequency},
    task::{PollResult, Task, TaskApp, thread::TaskThread},
    ui::{
        components::button_row,
        popup::{Popup, PopupIcon},
    },
};

pub const VERSION_MANIFEST: &str = "https://mslicer.com/version.json";
pub const CHANGELOG: &str = "https://mslicer.com/docs/changelog";

pub struct UpdateCheck {
    handle: TaskThread<CheckResult>,
}

#[derive(Deserialize)]
struct Manifest {
    version: String,
    release_date: String,
}

enum CheckResult {
    UpToDate,
    Outdated(Manifest),
}

impl UpdateCheck {
    pub fn new(app: &App) -> Self {
        let check_freq = app.config.ui.update_check.name();
        let ignore = app.config.ui.ignore_update.clone();

        let handle = TaskThread::spawn(move || {
            let response = ureq::get(VERSION_MANIFEST)
                .header("Version", VERSION)
                .header("Check-Freq", check_freq)
                .header("Operating-System", format!("{OS} {ARCH}"))
                .call()
                .unwrap();
            let manifest =
                serde_json::from_reader::<_, Manifest>(response.into_body().into_reader()).unwrap();

            if let Some(ignore) = ignore
                && ignore == manifest.version
            {
                info!("Update available, ignoring due to user preference.");
                return CheckResult::UpToDate;
            }

            if semver_cmp(VERSION, &manifest.version) == Some(Ordering::Less) {
                CheckResult::Outdated(manifest)
            } else {
                CheckResult::UpToDate
            }
        });

        Self { handle }
    }
}

impl Task for UpdateCheck {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle.poll_ignore_err().into_poll_result(|result| {
            if let CheckResult::Outdated(manifest) = result {
                app.popup
                    .open(Popup::new("Update Available", move |app, ui| {
                        Grid::new(ui.id().with("grid"))
                            .num_columns(2)
                            .show(ui, |ui| {
                                let icon = PopupIcon::Info;
                                ui.label(
                                    RichText::new(icon.as_char()).size(30.0).color(icon.color()),
                                );
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    ui.label("There is a mslicer update available! ");
                                    ui.code(format!("v{}", manifest.version));
                                    ui.label(format!(" was released {}. ", manifest.release_date));
                                    ui.label("View the changelog ");
                                    ui.hyperlink_to("here", CHANGELOG);
                                    ui.label(".");
                                });
                                ui.end_row();
                            });
                        ui.add_space(5.0);

                        let close = Cell::new(false);
                        button_row(
                            ui,
                            [
                                ("Close", &mut || close.set(true)),
                                ("Don't Tell Me Again", &mut || {
                                    app.config.ui.ignore_update = Some(manifest.version.clone());
                                    close.set(true);
                                }),
                            ],
                        );

                        close.get()
                    }));
            }

            app.config.ui.last_update_check = Some(Utc::now());
            PollResult::complete()
        })
    }
}

pub fn update_check_if_scheduled(app: &mut App) {
    let scheduled = match app.config.ui.update_check {
        UpdateCheckFrequency::Never => false,
        freq => match app.config.ui.last_update_check {
            Some(last) => last + freq.as_duration().unwrap() < Utc::now(),
            None => true,
        },
    };
    scheduled.then(|| app.tasks.add(UpdateCheck::new(app)));
}

/// Returns how `a` compares to `b` using semver semantics.
fn semver_cmp(a: &str, b: &str) -> Option<Ordering> {
    let (a, b) = (a.split('.'), b.split('.'));

    for (a, b) in a.zip(b).take(3) {
        let (a, b) = (a.parse::<u8>().ok()?, b.parse::<u8>().ok()?);
        if a < b {
            return Some(Ordering::Less);
        } else if a > b {
            return Some(Ordering::Greater);
        }
    }

    Some(Ordering::Equal)
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::task::update_check::semver_cmp;

    #[test]
    fn test() {
        assert_eq!(semver_cmp("1.2.3", "2.0.0"), Some(Ordering::Less));
        assert_eq!(semver_cmp("1.12", "1.2"), Some(Ordering::Greater));
        assert_eq!(semver_cmp("1.1.0", "1.1.0"), Some(Ordering::Equal));
    }
}
