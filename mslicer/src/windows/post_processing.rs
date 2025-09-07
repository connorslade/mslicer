use common::misc::human_duration;
use egui::{Color32, Context, ProgressBar, RichText, Ui};
use parking_lot::Mutex;
use std::sync::Arc;

use crate::app::App;
use crate::post_processing::{PassOutput, PassProgress, PassState};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    if let Some(ppo) = &mut app.post_processing_operation {
        if let Some(sop) = &app.slice_operation {
            // post-processing can only work on finished sliced file
            if sop.completion().is_some() {
                for pass in ppo.passes_mut() {
                    let progress = app
                        .state
                        .progress_bars
                        .entry(pass.name().into())
                        .or_insert_with(|| Arc::new(Mutex::new(PassProgress::default())));
                    pass.ui(ui, &app.slice_config);
                    let state = *pass.state().read().unwrap();
                    match state {
                        PassState::Running { .. } => {
                            let (progress, message) = {
                                let p = progress.lock();
                                (p.progress, p.message.clone())
                            };
                            ui.add(ProgressBar::new(progress).text(&message));
                            ui.ctx().request_repaint();
                        }
                        PassState::Ready => {
                            if ui.button(format!("Execute {}", pass.name())).clicked() {
                                tracing::debug!("starting pass...");
                                pass.run(&app.slice_config, sop.result.clone(), progress.clone());
                                tracing::debug!("started pass");
                            }
                        }
                        PassState::Completed { run_time } => {
                            ui.label(format!("Result after {}:", human_duration(run_time)));
                            match pass.result().lock().as_ref() {
                                Some(PassOutput::Analysis(report)) => {
                                    report.ui(ui, sop.result.clone());
                                    let mut lock = sop.result.lock();
                                    let res = lock.as_mut().unwrap();
                                    res.annotations.extend(report.annotations().iter());
                                }
                                // todo: other cases? errors?
                                _ => {}
                            }
                        }
                    }
                }
            } else {
                ui.label("... waiting for slicing to finish ...");
            }
        }
    } else {
        ui.label(
            RichText::new("Error: no post processing passes are configured.").color(Color32::RED),
        );
    }
}
