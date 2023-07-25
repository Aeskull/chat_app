use crate::{message::Message, prelude::*};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, OnceLock},
};
use tokio::sync::mpsc::channel;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Widget},
    Terminal,
};

// static INBOX: Arc<Mutex<VecDeque<Message>>> = Arc::new(Mutex::new(VecDeque::new()));

/// ### Terminal update loop.
///
/// Both the `sender_loop` and `reciever_loop` start from here.
///
/// Two sets of senders and recievers are made. One Sender is set to the `reciever_loop`, and one Reciever is passed to the `sender_loop`
pub async fn terminal_loop() -> Result<()> {
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let (stx, srx) = channel::<Message>(100);
    let (rtx, mut rrx) = channel::<Message>(100);

    tokio::spawn(async {
        if let Err(e) = crate::sender::sender_loop(srx).await {
            println!("ERROR: {e}");
            return;
        }
    });
    tokio::spawn(async {
        if let Err(e) = crate::reciever::reciever_loop(rtx).await {
            println!("ERROR: {e}");
            return;
        }
    });

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title("Block").borders(Borders::ALL);
        f.render_widget(block, size);
    })?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::message::Message;
    use crossterm::event::{KeyEvent, KeyEventKind};
    use crossterm::style;
    use tokio::sync::mpsc::channel;
    use tokio::{
        spawn,
        sync::mpsc::{Receiver, Sender},
    };

    /// Test of just figuring out how mpsc channels work.
    #[tokio::test]
    async fn async_test() {
        async fn sender_test(mut rx: Receiver<Message>) {
            while let Some(m) = rx.recv().await {
                println!("To be Sent: {m:?}");
            }
        }

        async fn reciever_test(tx: Sender<Message>) {
            std::thread::sleep(std::time::Duration::from_millis(5000));
            _ = tx.send(Message::new("Akachi", "I hate you!")).await;
        }

        let (stx, srx) = channel::<Message>(100);
        let (rtx, mut rrx) = channel::<Message>(100);

        let h1 = spawn(async move {
            sender_test(srx).await;
        });
        let h2 = spawn(async move {
            reciever_test(rtx).await;
        });

        _ = stx.send(Message::new("Aeskul", "Hello!")).await;
        while let Some(m) = rrx.recv().await {
            println!("From Reciever: {m:?}");
        }
    }

    #[test]
    fn tui_input_test() {
        use crossterm::{
            event::{
                self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode,
                KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
                PushKeyboardEnhancementFlags,
            },
            execute,
            terminal::{
                disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
            },
        };
        use std::{error::Error, io};
        use tui::{
            backend::{Backend, CrosstermBackend},
            layout::{Constraint, Direction, Layout},
            style::{Color, Modifier, Style},
            text::{Span, Spans, Text},
            widgets::{Block, Borders, List, ListItem, Paragraph},
            Frame, Terminal,
        };
        use unicode_width::UnicodeWidthStr;

        fn ui<B: Backend>(f: &mut Frame<B>, im: &mut InputMode, s: &str) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let (msg, style) = match im {
                InputMode::Normal => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to exit, "),
                        Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to start editing."),
                    ],
                    Style::default().add_modifier(Modifier::RAPID_BLINK),
                ),
                InputMode::Editing => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to stop editing, "),
                        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to record the message"),
                    ],
                    Style::default(),
                ),
            };

            let mut text = Text::from(Spans::from(msg));
            text.patch_style(style);
            let help_message = Paragraph::new(text);
            f.render_widget(help_message, chunks[0]);

            let input = Paragraph::new(s.as_ref())
                .style(match im {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                })
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, chunks[1]);
            match im {
                InputMode::Normal => {}
                InputMode::Editing => {
                    f.set_cursor(chunks[1].x + s.width() as u16 + 1, chunks[1].y + 1);
                }
            }
        }

        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        #[derive(Eq, PartialEq)]
        enum InputMode {
            Normal,
            Editing,
        }
        let mut im = InputMode::Normal;
        let mut s = String::new();

        loop {
            terminal.draw(|f| ui(f, &mut im, &s)).unwrap();
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(KeyEvent { code, kind, .. }) = event::read().unwrap() {
                    if kind == KeyEventKind::Press {
                        match im {
                            InputMode::Normal => match code {
                                KeyCode::Char('e') => {
                                    im = InputMode::Editing;
                                }
                                KeyCode::Char('q') => {
                                    break;
                                }
                                _ => {}
                            },
                            InputMode::Editing => match code {
                                KeyCode::Enter => {}
                                KeyCode::Char(c) => {
                                    s.push(c);
                                }
                                KeyCode::Backspace => {
                                    s.pop();
                                }
                                KeyCode::Esc => {
                                    im = InputMode::Normal;
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }
        }

        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
        )
        .unwrap();
        terminal.show_cursor().unwrap();
    }
}
