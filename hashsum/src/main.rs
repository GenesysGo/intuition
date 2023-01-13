use std::ops::Deref;

use crate::components::{generator::Generator, hasher::Hasher, heap::Heap, summer::Summer};
use intuition::{construct_profiler, dash::Dash};

use once_cell::sync::Lazy;

pub mod components;

construct_profiler!(Profiler for Traced: generator, summer, hasher, heap);

const WINDOW_SIZE: usize = 1_000;
const AVERAGES: usize = 1_000;
static PROFILER: Lazy<Profiler<WINDOW_SIZE, AVERAGES>> = Lazy::new(Profiler::new);

const GENERATOR_STOP_COUNT: usize = AVERAGES * WINDOW_SIZE * 1_000_000;

fn main() {
    // Initialize components
    let (generator, rx) = Generator::<f32, 16>::new();
    let (summer, rx) = Summer::new(rx);
    let (mut hasher, rx) = Hasher::new(rx);
    let mut heap = Heap::new(rx);

    // Initialize dashboard
    let mut dash = Dash::from_profiler(PROFILER.deref());

    println!("starting up modules");
    let handles = vec![
        std::thread::spawn(move || generator.wield()),
        std::thread::spawn(move || summer.wield()),
        std::thread::spawn(move || hasher.wield()),
        std::thread::spawn(move || heap.wield()),
    ];

    let dash_handle =
        std::thread::spawn(
            move || match dash.run(std::time::Duration::from_millis(50)) {
                Ok(()) => (),
                Err(e) => println!("Dashboard exited with error {e:?}"),
            },
        );

    for handle in handles {
        handle.join().unwrap();
    }

    println!("done");

    dash_handle.join().unwrap();
}
