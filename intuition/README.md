# Intuition: a super simple profiler with a terminal ui based on tui-rs.
Gain intuition about the goings-on of your multithreaded/multicomponent programs.

![Alt text](hashsum.png?raw=true "Title")

Intuition is a profiler that measures the average iteration time over time of a block of code. This block can be a single line or a block of code. In addition, every block you can emit its own compartmentalized logs.

Existing profilers that are either more fine/detailed or granular exist, but this offers a good balance of ease-of-use and utility. It is a rust pure rust library and uses the terminal for the gui so no external libraries are required. 

This profiler in particular was built for quickly being able to gain an understanding of multicomponent, multitheaded systems during a spike in activity or over time (e.g. detecting some slowdown due to the growth of a data structure or a bounded mpsc channel filling up). Other profilers like `coz` are wonderful for causal profiling of steady state systems, but are a little harder to gain insight from more dynamic systems.


# Example Usage
This example can be found in `examples/simple.rs`.
```rust
use intuition::{construct_profiler, Dash};

construct_profiler!(MyProgramProfiler for MyProgram: part_1);
const WINDOW_SIZE: usize = 100;
const AVERAGE_HISTORY_SIZE: usize = 1000;
static PROFILER: MyProgramProfiler<WINDOW_SIZE, AVERAGE_HISTORY_SIZE> = MyProgramProfiler::new();

fn main() {
    let mut dash = Dash::from_profiler(&PROFILER);

    let handle = std::thread::spawn(|| {
        let mut i: u64 = 0;
        loop {
            // do something before iteration which you don't want to time

            // Define a block to time
            let _new_i: u64 = PROFILER.part_1.iteration(|| {
                i += 1;
                i
            });

            // do something with result which you don't want to time
        }
    });

    let dash_handle =
        std::thread::spawn(
            move || match dash.run(std::time::Duration::from_millis(50)) {
                Ok(()) => (),
                Err(e) => println!("Dashboard exited with error {e:?}"),
            },
        );
    handle.join().unwrap();
    dash_handle.join().unwrap();
}
```
To quit the tui/dashboard, simply press `q` twice.





