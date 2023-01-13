use std::{iter::Sum, sync::atomic::Ordering};

use bytemuck::{bytes_of, Pod};
use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError, TrySendError};
use sha2::{
    digest::{
        generic_array::GenericArray,
        typenum::{UInt, UTerm, B0, B1},
    },
    Digest, Sha256,
};

use crate::{GENERATOR_STOP_COUNT, PROFILER};

pub type Sha256Hash =
    GenericArray<u8, UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>>;

pub struct Hasher<T: Send + Sum> {
    summer_to_hasher_rx: Receiver<T>,
    hasher_to_heap_tx: Sender<Sha256Hash>,
    hasher: Sha256,
}

impl<T: Send + Sum + Pod> Hasher<T> {
    pub fn new(rx: Receiver<T>) -> (Self, Receiver<Sha256Hash>) {
        let (tx_, rx_) = bounded(100);

        let hasher = Sha256::new();
        (
            Hasher {
                summer_to_hasher_rx: rx,
                hasher_to_heap_tx: tx_,
                hasher,
            },
            rx_,
        )
    }
    pub fn wield(&mut self) {
        loop {
            // Check if we should break out of loop
            if PROFILER.summer.total_count.load(Ordering::Acquire) == GENERATOR_STOP_COUNT {
                return;
            }

            // Otherwise perform one iteration
            PROFILER.hasher.iteration(|| {
                let mut newest_hash: Option<Sha256Hash> = Some({
                    match self.summer_to_hasher_rx.try_recv() {
                        // Get recent sum if available
                        Ok(sum) => {
                            self.hasher.update(bytes_of(&sum));
                            self.hasher.clone().finalize()
                        }
                        // If empty, hash self
                        Err(TryRecvError::Empty) => {
                            self.hasher.update(self.hasher.clone().finalize());
                            self.hasher.clone().finalize()
                        }
                        Err(TryRecvError::Disconnected) => unreachable!(),
                    }
                });

                // // Send hash to heap
                // self.hasher_to_heap_tx
                //     .send(newest_hash)
                //     .expect("rx should never be dropped");

                while newest_hash.is_some() {
                    match self.hasher_to_heap_tx.try_send(newest_hash.take().unwrap()) {
                        Ok(()) => {}

                        #[allow(unused_must_use)]
                        Err(TrySendError::Full(h)) => {
                            newest_hash.insert(h);
                            PROFILER.hasher.warn("hash to heap full")
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
