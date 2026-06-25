use std::{
    mem,
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    thread,
};

use common::progress::Progress;
use nalgebra::{Vector2, Vector3};
use parking_lot::Mutex;
use rand::{Rng, rng};

use crate::auto_layout::{AutoLayoutNFP, Model, Objective};

pub struct AutoLayoutAnnealing {
    pub config: Config,
    pub models: Vec<Model>,
    pub running: Option<Running>,
}

#[derive(Clone)]
pub struct Config {
    pub objective: Objective,
    pub padding: f32,
    pub segment_steps: f32,
    pub platform_size: Vector2<f32>,

    pub start_temp: f32,
    pub end_temp: f32,
    pub cooling: f32,
    pub iters: u32,
}

pub struct Running {
    pub rx: Receiver<Vec<(u32, Vector3<f32>)>>,
    pub history: Arc<Mutex<Vec<f32>>>,
    stop: mpsc::SyncSender<()>,
}

impl AutoLayoutAnnealing {
    pub fn run(&mut self) {
        let (tx, rx) = mpsc::sync_channel(16);
        let (stop_tx, stop_rx) = mpsc::sync_channel(1);

        let history = Arc::new(Mutex::new(Vec::new()));
        self.running = Some(Running {
            rx,
            history: history.clone(),
            stop: stop_tx,
        });

        let mut models = self.models.clone();
        let config = self.config.clone();

        let score = move |models| {
            AutoLayoutNFP::new_unsorted(config.platform_size, models)
                .objective(config.objective)
                .padding(config.padding)
                .segment_steps(config.segment_steps)
                .layout(true, Progress::new())
        };

        thread::spawn(move || {
            let mut temp = config.start_temp;
            let (mut best_score, mut best) =
                score(models.to_vec()).unwrap_or_else(|| (f32::MAX, Vec::new()));
            let mut global_best = f32::MAX;

            while temp > config.end_temp && stop_rx.try_recv().is_err() {
                for _ in 0..config.iters {
                    let iter_models = perturb(&models);

                    let Some((iter_score, result)) = score(iter_models.clone()) else {
                        continue;
                    };

                    let delta = iter_score - best_score;

                    if delta < 0.0 || rng().random::<f32>() < (-delta / temp).exp() {
                        history.lock().push(iter_score);
                        if iter_score < global_best {
                            global_best = best_score;
                            let _ = tx.send(result.clone());
                        }

                        models = iter_models;
                        best_score = iter_score;
                        best = result;
                    }
                }
                temp *= config.cooling;
            }

            (best_score, best)
        });
    }

    pub fn stop(&mut self) {
        if let Some(running) = mem::take(&mut self.running) {
            let _ = running.stop.send(());
        }
    }
}

fn perturb(models: &[Model]) -> Vec<Model> {
    let mut out = models.to_vec();

    let mut rng = rng();
    let range = 0..models.len();

    match rng.random_range(0..2) {
        0 => out.swap(rng.random_range(range.clone()), rng.random_range(range)),
        1 => {
            let i = rng.random_range(range);
            (i + 1 < models.len()).then(|| out.swap(i, i + 1));
        }
        2 => {
            // todo: rotation
        }
        _ => unreachable!(),
    }

    out
}

impl Default for AutoLayoutAnnealing {
    fn default() -> Self {
        Self {
            config: Config {
                objective: Objective::Area,
                padding: 2.0,
                segment_steps: 10.0,
                platform_size: Default::default(),

                start_temp: 100.0,
                end_temp: 0.01,
                cooling: 0.99,
                iters: 50,
            },
            models: Default::default(),
            running: Default::default(),
        }
    }
}
