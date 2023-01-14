use std::sync::atomic::Ordering;

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use rand::{distributions::Standard, prelude::Distribution};

use crate::{GENERATOR_STOP_COUNT, PROFILER};

pub struct Generator<T: Send, const S: usize> {
    generator_to_summer_tx: Sender<[T; S]>,
}

impl<T: Send, const S: usize> Generator<T, S>
where
    Standard: Distribution<[T; S]>,
{
    pub fn new() -> (Self, Receiver<[T; S]>) {
        let (tx, rx) = bounded(1_000_000);
        (
            Generator {
                generator_to_summer_tx: tx,
            },
            rx,
        )
    }
    pub fn wield(&self) {
        loop {
            // Check if we should break out of loop
            if PROFILER.generator.total_count.load(Ordering::Acquire) == GENERATOR_STOP_COUNT {
                return;
            };

            // Otherwise, perform one iteration
            PROFILER.generator.iteration(|| {
                let mut sample: Option<[T; S]> = Some({
                    // Generate a sample
                    rand::random()
                });

                // Send sample to summer
                while sample.is_some() {
                    match self.generator_to_summer_tx.try_send(sample.take().unwrap()) {
                        Ok(()) => {}
                        Err(TrySendError::Disconnected(_)) => {
                            unreachable!("summer rx will be waiting")
                        }
                        #[allow(unused_must_use)]
                        Err(TrySendError::Full(s)) => {
                            sample.insert(s);
                            PROFILER.generator.warn("gen to sum full")
                        }
                    }
                }
            });
        }
    }
}
