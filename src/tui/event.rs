use anyhow::Result;
use crossterm::event::{self, KeyEvent, MouseEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Terminal events
#[derive(Debug)]
pub enum Event {
    /// Key press
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Tick (for animations/updates)
    Tick,
}

/// Event handler
pub struct EventHandler {
    rx: mpsc::Receiver<Event>,
    _tx: mpsc::Sender<Event>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate in milliseconds
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (tx, rx) = mpsc::channel();
        let _tx = tx.clone();

        thread::spawn(move || {
            loop {
                // Poll for events with timeout
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(event::Event::Key(key)) if tx.send(Event::Key(key)).is_err() => {
                            break;
                        }
                        Ok(event::Event::Mouse(mouse)) if tx.send(Event::Mouse(mouse)).is_err() => {
                            break;
                        }
                        Ok(event::Event::Resize(w, h)) if tx.send(Event::Resize(w, h)).is_err() => {
                            break;
                        }
                        Ok(_) => {}
                        _ => {}
                    }
                } else {
                    // Send tick event
                    if tx.send(Event::Tick).is_err() {
                        break;
                    }
                }
            }
        });

        Self { rx, _tx }
    }

    /// Get the next event
    pub fn next(&self) -> Result<Event> {
        Ok(self.rx.recv()?)
    }
}
