use std::{iter::Sum, sync::atomic::Ordering};

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};

use crate::{GENERATOR_STOP_COUNT, PROFILER};

pub struct Summer<T: Send + Sum, const S: usize> {
    generator_to_summer_rx: Receiver<[T; S]>,
    summer_to_hasher_tx: Sender<T>,
}

impl<T: Send + Sum, const S: usize> Summer<T, S> {
    pub fn new(rx: Receiver<[T; S]>) -> (Self, Receiver<T>) {
        let (tx_, rx_) = bounded(100_000);
        (
            Summer {
                generator_to_summer_rx: rx,
                summer_to_hasher_tx: tx_,
            },
            rx_,
        )
    }
    pub fn wield(&self) {
        loop {
            // Check if we should break out of loop
            if PROFILER.summer.total_count.load(Ordering::Acquire) == GENERATOR_STOP_COUNT {
                return;
            }

            // Otherwise, perform one iteration
            PROFILER.summer.iteration(|| {
                let sample = self
                    .generator_to_summer_rx
                    .recv()
                    .expect("tx never dropped");
                let mut sample_sum: Option<T> = Some(
                    // Sum sample
                    sample.into_iter().sum(),
                );

                // // Send sum to hasher
                // self.summer_to_hasher_tx
                //     .send(sample_sum)
                //     .expect("rx should never be dropped");

                // Send sum to hasher
                while sample_sum.is_some() {
                    match self
                        .summer_to_hasher_tx
                        // .send(sample)
                        .try_send(sample_sum.take().unwrap())
                    {
                        Ok(()) => {}

                        #[allow(unused_must_use)]
                        Err(TrySendError::Full(s)) => {
                            sample_sum.insert(s);
                            PROFILER.summer.warn("sum to hash full")
                        }

                        Err(TrySendError::Disconnected(_)) => {
                            unreachable!("hasher rx will be waiting")
                        }
                    }
                }
            });
        }
    }
}
