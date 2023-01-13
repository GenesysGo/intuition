use super::timer::Log;

#[macro_export]
macro_rules! construct_profiler {
    ($name:ident for $title:ident: $( $scope:ident ),*) => {
        #[derive(Default)]
        pub(crate) struct $name<const W: usize, const A: usize> {
            $(
                pub(crate) $scope: $crate::timer::Timer<W,A>,
            )*
        }

        impl<const W: usize, const A: usize> $name<W,A> {
            pub(crate) fn new() -> Self {
                $name::default()
            }

        }

        impl<const W: usize, const A: usize> $crate::profiler::ProfilerExt for $name<W,A> {
            const SCOPES: &'static [&'static str] = &[$(stringify!($scope),)*];
            const TITLE: &'static str = stringify!($title);
            const WINDOW_SIZE: usize = W;
            const NUM_AVERAGES: usize = A;

            fn state_buffer(&self) -> $crate::profiler::StateBuffer {
                vec![
                    $(
                        (stringify!($scope), Vec::with_capacity(A)),
                    )*
                ]
            }

            fn log_buffer(&self) -> $crate::profiler::LogBuffer {
                vec![
                    $(
                        (stringify!($scope), vec![]),
                    )*
                ]
            }

            fn update_buffer(&self, buffer: &mut $crate::profiler::StateBuffer) {

                assert_eq!(buffer.len(), Self::SCOPES.len());

                let mut i = 0;
                #[allow(unused_assignments)] // i is incremented on the last loop as well
                {
                    $(
                        // Unpack tuple
                        let (scope_name, recent_averages) = &mut buffer[i];
                        // Ensure we are updating proper scope
                        assert_eq!(scope_name, &stringify!($scope));
                        assert_eq!(recent_averages.capacity(), A);
                        self.$scope.recent_averages.lock().unwrap().clone_into(recent_averages);
                        i += 1;
                    )*
                }
            }

            fn update_logs(&self, buffer: &mut $crate::profiler::LogBuffer) {

                assert_eq!(buffer.len(), Self::SCOPES.len());

                let mut i = 0;
                #[allow(unused_assignments)] // i is incremented on the last loop as well
                {
                    $(
                        // Unpack tuple
                        let (scope_name, logs) = &mut buffer[i];
                        // Ensure we are updating proper scope
                        assert_eq!(scope_name, &stringify!($scope));
                        // Take current logs
                        logs.append(&mut std::mem::take(&mut *self.$scope.logs.lock().unwrap()));
                        i += 1;
                    )*
                }
            }
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
