use std::sync::atomic::Ordering;

use crate::{components::hasher::Sha256Hash, GENERATOR_STOP_COUNT, PROFILER};

use crossbeam_channel::Receiver;

pub struct Heap {
    hasher_to_heap_rx: Receiver<Sha256Hash>,
    heap: Vec<Sha256Hash>,
}

impl Heap {
    pub fn new(rx: Receiver<Sha256Hash>) -> Self {
        Heap {
            hasher_to_heap_rx: rx,
            heap: vec![],
        }
    }
    pub fn wield(&mut self) {
        loop {
            // Check if we should break out of loop
            if PROFILER.summer.total_count.load(Ordering::Acquire) == GENERATOR_STOP_COUNT {
                return;
            }

            let hash = self
                .hasher_to_heap_rx
                .recv()
                .expect("should receive a hash");

            // Otherwise, perform one iteration
            PROFILER.heap.iteration(|| {
                // Add hash to heap
                self.heap.push(hash);

                if self.heap.len() == 1_500_000 {
                    self.heap.append(&mut self.heap.clone());
                    PROFILER.heap.warn("expensive operation detected");
                }

                // // drop hash
                // drop(hash)
            });
        }
    }
}
