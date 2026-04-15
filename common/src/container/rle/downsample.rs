use crate::container::Run;

#[derive(Clone)]
struct RunQueue<'a, T> {
    runs: &'a [Run<T>],
    next: usize,
    active: Run<T>,
}

pub fn downsample_adjacent(factor: u8, runs: &[Run], out: &mut Vec<Run>) {
    let mut queue = RunQueue::new(runs);
    let factor = factor as u64;

    let mut i = 0;
    while queue.remaining() {
        let run = queue.next();
        i += run.length;

        let length = run.length / factor;
        let mut remaining = i % factor;
        if length > 0 {
            out.push(Run {
                length,
                value: run.value,
            });
        }

        if remaining == 0 {
            continue;
        }

        // if not a clean split, we will need to do some averaging
        let mut interp = remaining * run.value as u64;
        while remaining != factor {
            // try to complete remaining by pulling from next run
            let next = queue.take_up_to(factor - remaining);
            interp += next.length * next.value as u64;
            remaining += next.length;
            i += next.length;
        }

        out.push(Run {
            length: 1,
            value: (interp / factor) as u8,
        });
    }
}

pub fn downsample(chunks: &[Vec<Run>], width: u64, out: &mut Vec<Run>) {
    let mut queues = chunks
        .iter()
        .map(|x| RunQueue::new_fallback(x, width))
        .collect::<Vec<_>>();
    while queues[0].remaining() {
        let length = queues.iter().map(|x| x.active.length).min().unwrap();

        let mut value = 0;
        for chunk in queues.iter_mut() {
            let front = chunk.take_up_to(length);
            value += front.value as u64;
        }

        out.push(Run::new(length, (value / chunks.len() as u64) as u8));
    }
}

impl<'a, T: Copy + Default> RunQueue<'a, T> {
    pub fn new(runs: &'a [Run<T>]) -> Self {
        Self {
            runs,
            next: 1,
            active: runs[0],
        }
    }

    pub fn new_fallback(runs: &'a [Run<T>], fallback: u64) -> Self {
        if runs.is_empty() {
            Self {
                runs,
                next: 1,
                active: Run::new(fallback, T::default()),
            }
        } else {
            Self::new(runs)
        }
    }

    pub fn remaining(&self) -> bool {
        self.active.length > 0
    }

    pub fn next(&mut self) -> Run<T> {
        let out = self.active;
        if self.next < self.runs.len() {
            self.active = self.runs[self.next];
            self.next += 1;
        } else {
            self.active.length = 0;
        }
        out
    }

    pub fn take_up_to(&mut self, n: u64) -> Run<T> {
        if self.active.length <= n {
            self.next()
        } else {
            self.active.length -= n;
            Run::new(n, self.active.value)
        }
    }
}
