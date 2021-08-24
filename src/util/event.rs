use std::{
    io,
    thread,
    time::Duration,
    env
};

use std::sync::{
    mpsc,
    atomic::{AtomicBool, Ordering},
    Arc,
};

use termion::{
    event::Key,
    input::TermRead,
};

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
    tick_handle: thread::JoinHandle<()>,
}

const DEFAULT_TICK_RATE: u64 = 250;


#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: match env::var("TICK_RATE").ok() {
                Some(x) => Duration::from_millis(*x.parse::<u64>().ok().get_or_insert(DEFAULT_TICK_RATE)),
                None => Duration::from_millis(DEFAULT_TICK_RATE),
            },
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                        if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                            return;
                        }
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };
        Events {
            rx,
            ignore_exit_key,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }
}
