use std::{
    error::Error,
    time::{Duration, Instant},
};

#[allow(unused)] // in case mouse capture is enabled in the future
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use super::profiler::{LogBuffer, ProfilerExt, StateBuffer};

mod ui;

pub struct Dash<P: ProfilerExt + 'static> {
    profiler: &'static P,
    state_buffer: StateBuffer,
    tabs: TabsState<'static>,
    q_counter: u8,
    should_quit: bool,
    show_log: bool,
    log_buffer: LogBuffer,
    // Just so we don't allocate
    domain: Vec<f64>,
}

impl<P: ProfilerExt + 'static> Dash<P> {
    pub fn from_profiler(profiler: &'static P) -> Dash<P> {
        Dash {
            profiler,
            state_buffer: profiler.state_buffer(),
            log_buffer: profiler.log_buffer(),
            tabs: TabsState::new(vec![P::TITLE]),
            q_counter: 0,
            should_quit: false,
            show_log: true,
            domain: (0..P::NUM_AVERAGES).map(|i| i as f64).collect(),
        }
    }

    pub fn run(&mut self, tick_rate: Duration) -> Result<(), Box<dyn Error>> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen /*, EnableMouseCapture */)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // create app and run it
        let res = self.run_app(&mut terminal, tick_rate);

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen /*,
                                 DisableMouseCapture*/
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }

    fn run_app<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        tick_rate: Duration,
    ) -> std::io::Result<()> {
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|f| ui::draw(f, self))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char(c) => self.on_key(c),
                        KeyCode::Left => self.on_left(),
                        KeyCode::Up => self.on_up(),
                        KeyCode::Right => self.on_right(),
                        KeyCode::Down => self.on_down(),
                        _ => {}
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }
            if self.should_quit {
                return Ok(());
            }
        }
    }

    pub fn on_key(&mut self, key: char) {
        match key {
            // Quit key
            'q' => {
                // Increment q counter
                self.q_counter += 1;

                // If hit enough times, quit
                const NUM_Q_TO_QUIT: u8 = 2;
                if self.q_counter == NUM_Q_TO_QUIT {
                    self.should_quit = true
                }
            }
            // toggle logs
            'l' => {
                // Reset q counter on non-q key
                self.q_counter = 0;

                // If hit enough times, quit
                self.show_log = !self.show_log;
            }
            _ => {
                // Reset q counter on any other key
                self.q_counter = 0;

                // Do nothing for now
            }
        }
    }

    pub fn on_up(&mut self) {
        // do nothing for now
    }

    pub fn on_down(&mut self) {
        // do nothing for now
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_tick(&mut self) {
        // Update state buffer
        self.profiler.update_buffer(&mut self.state_buffer);
        // Update log buffer
        self.profiler.update_logs(&mut self.log_buffer);
    }
}

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}
