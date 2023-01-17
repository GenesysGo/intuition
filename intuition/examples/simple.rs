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
