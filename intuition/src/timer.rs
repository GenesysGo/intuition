use std::{
    borrow::Cow,
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, MutexGuard,
    },
    time::Instant,
};

/// [Timer] is a submodule of a profiler; a profiler can contain many timers.
pub struct Timer<const W: usize, const A: usize> {
    pub total_count: AtomicUsize,
    pub total_time: AtomicUsize,
    pub recent_averages: Mutex<Vec<usize>>,
    pub current_count: AtomicUsize,
    pub current_time: AtomicUsize,
    pub logs: Mutex<Vec<Log>>,
}

pub struct Log {
    pub level: LogLevel,
    pub log: Cow<'static, str>,
}

pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => f.write_str("INFO"),
            LogLevel::Warn => f.write_str("WARN"),
            LogLevel::Error => f.write_str("ERROR"),
        }
    }
}

impl<const W: usize, const A: usize> Timer<W, A> {
    pub fn iteration<T, F: FnOnce() -> T>(&self, iter: F) -> T {
        let start = Instant::now();
        let output = iter();
        self.add_time(
            start
                .elapsed()
                .as_nanos()
                .try_into()
                .expect("nanos shouldn't overflow usize"),
        );
        output
    }

    /// This function takes the loop time and adds it to total and current time,
    /// increments total and current count, and resets current/recent time and
    /// count if necessary.
    fn add_time(&self, loop_time: usize) {
        // Add to times
        self.total_time.fetch_add(loop_time, Ordering::AcqRel);
        self.current_time.fetch_add(loop_time, Ordering::AcqRel);

        // Increment counters
        self.total_count.fetch_add(1, Ordering::AcqRel);
        // fetch_add returns previous value so we add 1 to get the value we stored
        let current_count = self.current_count.fetch_add(1, Ordering::AcqRel) + 1;

        // Check if time to average
        if current_count == W {
            // Reset current_count
            self.current_count.store(0, Ordering::Release);

            // Reset timer but get last value via fetch_update
            let current_time: usize = self
                .current_time
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |_| Some(0))
                .unwrap();

            // Calculate recent average
            let recent_average: usize = current_time / W;

            #[allow(unused_labels)]
            'mutex_scope: {
                let mut recent_averages: MutexGuard<Vec<usize>> =
                    self.recent_averages.lock().unwrap();
                // If at capacity, remove front element before adding to back
                if recent_averages.len() == A {
                    recent_averages.remove(0);
                }
                // Add to back
                // NOTE: because we hold lock, should never double remove or have
                // anything weird happen in between remove and push
                recent_averages.push(recent_average);
            }
        }
    }

    pub fn info<L: Into<Cow<'static, str>>>(&self, log: L) {
        self.logs.lock().unwrap().push(Log {
            level: LogLevel::Info,
            log: log.into(),
        });
    }

    pub fn error<L: Into<Cow<'static, str>>>(&self, log: L) {
        self.logs.lock().unwrap().push(Log {
            level: LogLevel::Error,
            log: log.into(),
        });
    }

    pub fn warn<L: Into<Cow<'static, str>>>(&self, log: L) {
        self.logs.lock().unwrap().push(Log {
            level: LogLevel::Warn,
            log: log.into(),
        });
    }
}

impl<const W: usize, const A: usize> Default for Timer<W, A> {
    fn default() -> Self {
        Self {
            total_count: AtomicUsize::new(0),
            total_time: AtomicUsize::new(0),
            // Allocate for A elements
            recent_averages: Mutex::new(vec![0;A]),
            current_count: AtomicUsize::new(0),
            current_time: AtomicUsize::new(0),
            logs: Mutex::new(vec![]),
        }
    }
}
