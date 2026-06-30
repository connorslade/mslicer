//! References:
//! - [Applying Simulated Annealing and the No Fit Polygon to the Nesting Problem](https://www.graham-kendall.com/papers/bk1999c.pdf)

use std::{
    f32::consts::{PI, TAU},
    mem,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
};

use common::progress::Progress;
use nalgebra::Vector2;
use parking_lot::Mutex;
use rand::{RngExt, rng, rngs::ThreadRng};

use crate::auto_layout::{AutoLayoutNfp, Model, Objective, Placement, cache::LayoutCache};

pub struct AutoLayoutAnnealing {
    pub config: Config,
    pub models: Vec<Model>,
    pub running: Option<Running>,
}

#[derive(Clone)]
pub struct Config {
    pub objective: Objective,
    pub rotation: Rotation,
    pub padding: f32,
    pub segment_steps: f32,
    pub platform_size: Vector2<f32>,

    pub start_temp: f32,
    pub end_temp: f32,
    pub cooling: f32,
    pub iters: u32,
    pub bounds_penalty: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    Disabled,
    Cardinal,
    Intercardinal,
    Continuous,
}

pub struct Running {
    pub rx: Receiver<Vec<Placement>>,
    pub history: Arc<Mutex<Vec<(u64, f32)>>>,
    pub iteration: Arc<AtomicU64>,
    stop: mpsc::SyncSender<()>,
}

impl AutoLayoutAnnealing {
    pub fn run(&mut self, mut cache: LayoutCache) {
        let (tx, rx) = mpsc::sync_channel(16);
        let (stop_tx, stop_rx) = mpsc::sync_channel(1);

        let history = Arc::new(Mutex::new(Vec::new()));
        let iteration = Arc::new(AtomicU64::new(0));
        self.running = Some(Running {
            rx,
            history: history.clone(),
            iteration: iteration.clone(),
            stop: stop_tx,
        });

        let mut models = self.models.clone();
        let config = self.config.clone();

        let mut score = move |models| {
            AutoLayoutNfp::new(config.platform_size, models, &mut cache)
                .objective(config.objective)
                .segment_steps(config.segment_steps)
                .bounds_penalty(config.bounds_penalty)
                .layout(Progress::new())
        };

        thread::spawn(move || {
            let mut temperature = config.start_temp;
            let (mut best_score, mut best) = score(models.to_vec());
            let mut global_best = f32::MAX;

            let mut i = 0;
            while temperature > config.end_temp && stop_rx.try_recv().is_err() {
                for _ in 0..config.iters {
                    let iter_models = perturb(config.rotation, &models);
                    let (iter_score, result) = score(iter_models.clone());

                    let delta = iter_score - best_score;

                    if delta < 0.0 || rng().random::<f32>() < (-delta / temperature).exp() {
                        if iter_score < global_best {
                            history.lock().push((i, iter_score));
                            global_best = iter_score;
                            let _ = tx.send(result.clone());
                        }

                        models = iter_models;
                        best_score = iter_score;
                        best = result;
                    }

                    i += 1;
                    iteration.store(i, Ordering::Relaxed);
                }
                temperature *= config.cooling;
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

impl Rotation {
    pub const ALL: [Self; 4] = [
        Self::Disabled,
        Self::Cardinal,
        Self::Intercardinal,
        Self::Continuous,
    ];

    pub fn name(&self) -> &str {
        match self {
            Rotation::Disabled => "Disabled",
            Rotation::Cardinal => "Cardinal (90°)",
            Rotation::Intercardinal => "Intercardinal (45°)",
            Rotation::Continuous => "Continuous",
        }
    }

    pub fn random(&self, rng: &mut ThreadRng) -> f32 {
        match self {
            Rotation::Disabled => 0.0,
            Rotation::Cardinal => PI / 2.0 * rng.random_range(0..=4) as f32,
            Rotation::Intercardinal => PI / 4.0 * rng.random_range(0..=8) as f32,
            Rotation::Continuous => rng.random::<f32>() * TAU,
        }
    }
}

fn perturb(rotation: Rotation, models: &[Model]) -> Vec<Model> {
    let mut out = models.to_vec();

    let mut rng = rng();
    let range = 0..models.len();

    match rng.random_range(0..=[5, 3][matches!(rotation, Rotation::Disabled) as usize]) {
        // Swap two random model's insertion order
        0 => out.swap(rng.random_range(range.clone()), rng.random_range(range)),
        // Swap two model's with an adjacent insertion order
        1 => {
            let i = rng.random_range(range);
            out.swap(i, (i + 1) % models.len())
        }
        // Re-insert a model at a new point in the insertion order, shifting all
        // the models between
        2 => {
            let (i, j) = (rng.random_range(range.clone()), rng.random_range(range));
            if i != j {
                let item = out.remove(i);
                out.insert(j, item);
            }
        }
        // Reverse a run of insertions orders
        3 => {
            let (i, j) = (rng.random_range(range.clone()), rng.random_range(range));
            out[i.min(j)..=i.max(j)].reverse();
        }
        // Rotate a model (if rotation mode is non Disabled)
        4 => {
            let i = rng.random_range(range);
            out[i].rotation = rotation.random(&mut rng);
        }
        // Rotate all models by the same amount
        5 => {
            for model in out.iter_mut() {
                model.rotation = (model.rotation + rotation.random(&mut rng)).rem_euclid(TAU);
            }
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
                rotation: Rotation::Disabled,
                padding: 2.0,
                segment_steps: 1.0,
                platform_size: Default::default(),

                start_temp: 100.0,
                end_temp: 0.01,
                cooling: 0.99,
                iters: 50,
                bounds_penalty: 10_000.0,
            },
            models: Default::default(),
            running: Default::default(),
        }
    }
}
