use super::timer::Log;
pub use concat_idents::concat_idents as ci;
pub use once_cell::sync::Lazy;

#[macro_export(local_inner_macros)]
macro_rules! construct_profiler {
    ($title:ident: $( $scope:ident ),*) => {
        construct_profiler!(Profiler for $title: $($scope),*);
    };

    ($name:ident for $title:ident: $( $scope:ident ),*) => {

        use __inner_profiler_module::$name;
        mod __inner_profiler_module {

        $crate::profiler::ci!(inner = $name, Inner {

            pub(crate) struct $name<const W: usize, const A: usize>(intuition::profiler::Lazy<inner<W,A>>);

            impl<const W: usize, const A: usize> $name<W,A> {
                pub const fn new() -> Self {
                    Self(intuition::profiler::Lazy::new(inner::new))
                }
            }

            impl<const W: usize, const A: usize> std::ops::Deref for $name<W,A> {
                type Target = inner<W, A>;
                fn deref(&self) -> &Self::Target {
                    self.0.deref()
                }
            }

            #[derive(Default)]
            pub(crate) struct inner<const W: usize, const A: usize> {
                $(
                    pub(crate) $scope: $crate::timer::Timer<W,A>,
                )*
            }

            impl<const W: usize, const A: usize> inner<W,A> {
                pub(crate) fn new() -> Self {
                    inner::default()
                }

            }

            impl<const W: usize, const A: usize> $crate::profiler::ProfilerExt for inner<W,A> {
                const SCOPES: &'static [&'static str] = &[$(std::stringify!($scope),)*];
                const TITLE: &'static str = std::stringify!($title);
                const WINDOW_SIZE: usize = W;
                const NUM_AVERAGES: usize = A;

                fn state_buffer(&self) -> $crate::profiler::StateBuffer {
                    std::vec![
                        $(
                            (std::stringify!($scope), Vec::with_capacity(A)),
                        )*
                        ]
                    }

                fn log_buffer(&self) -> $crate::profiler::LogBuffer {
                    std::vec![
                        $(
                            (std::stringify!($scope), std::vec![]),
                        )*
                        ]
                    }

                    fn update_buffer(&self, buffer: &mut $crate::profiler::StateBuffer) {

                        std::assert_eq!(buffer.len(), Self::SCOPES.len());

                        let mut i = 0;
                        #[allow(unused_assignments)] // i is incremented on the last loop as well
                        {
                            $(
                                // Unpack tuple
                            let (scope_name, recent_averages) = &mut buffer[i];
                            // Ensure we are updating proper scope
                            std::assert_eq!(scope_name, &std::stringify!($scope));
                            std::assert_eq!(recent_averages.capacity(), A);
                            self.$scope.recent_averages.lock().unwrap().clone_into(recent_averages);
                            i += 1;
                        )*
                    }
                }

                fn update_logs(&self, buffer: &mut $crate::profiler::LogBuffer) {

                    std::assert_eq!(buffer.len(), Self::SCOPES.len());

                    let mut i = 0;
                    #[allow(unused_assignments)] // i is incremented on the last loop as well
                    {
                        $(
                            // Unpack tuple
                            let (scope_name, logs) = &mut buffer[i];
                            // Ensure we are updating proper scope
                            std::assert_eq!(scope_name, &std::stringify!($scope));
                            // Take current logs
                            logs.append(&mut std::mem::take(&mut *self.$scope.logs.lock().unwrap()));
                            i += 1;
                        )*
                    }
                }
            }
        });
    }
        };
    }

pub type StateBuffer = Vec<(&'static str, Vec<usize>)>;
pub type LogBuffer = Vec<(&'static str, Vec<Log>)>;

pub trait ProfilerExt {
    const SCOPES: &'static [&'static str];
    const TITLE: &'static str;
    const WINDOW_SIZE: usize;
    const NUM_AVERAGES: usize;
    fn update_logs(&self, buffer: &mut LogBuffer);
    fn update_buffer(&self, buffer: &mut StateBuffer);
    fn state_buffer(&self) -> StateBuffer;
    fn log_buffer(&self) -> LogBuffer;
}
