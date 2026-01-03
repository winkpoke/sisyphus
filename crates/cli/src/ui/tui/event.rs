use super::action::Action;
use crossterm::event::{Event as CrosstermEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

#[allow(dead_code)]
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Action>,
    receiver: mpsc::UnboundedReceiver<Action>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: std::time::Duration) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();

        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                let tick_delay = tick_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = tick_delay => {
                        if _sender.send(Action::Tick).is_err() {
                            break;
                        }
                    }
                    Some(Ok(evt)) = crossterm_event => {
                        match evt {
                            CrosstermEvent::Key(key) => {
                                if key.kind == crossterm::event::KeyEventKind::Press {
                                    if _sender.send(Action::Key(key)).is_err() {
                                        break;
                                    }
                                }
                            }
                            CrosstermEvent::Resize(w, h) => {
                                if _sender.send(Action::Resize(w, h)).is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub async fn next(&mut self) -> Option<Action> {
        self.receiver.recv().await
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Action> {
        self.sender.clone()
    }
}
