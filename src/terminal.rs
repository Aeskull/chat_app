use crate::{message::Message, prelude::*};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::mpsc::channel;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
    Frame, Terminal,
};
use tui_textarea::{Input, Key, TextArea};

/// ### Terminal update loop.
///
/// Both the `sender_loop` and `reciever_loop` start from here.
///
/// Two sets of senders and recievers are made. One Sender is set to the `reciever_loop`, and one Reciever is passed to the `sender_loop`
pub async fn terminal_loop(user: String, ip: String) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let (stx, srx) = channel::<String>(100);
    let (rtx, mut rrx) = channel::<Message>(100);
    let (sstx, ssrx) = channel::<String>(100);
    let rip = ip.clone();

    tokio::spawn(async {
        if let Err(e) = crate::sender::sender_loop(srx, user, ip, sstx).await {
            println!("ERROR: {e}");
            return;
        }
    });
    tokio::spawn(async {
        if let Err(e) = crate::reciever::reciever_loop(rtx, rip, ssrx).await {
            println!("ERROR: {e}");
            return;
        }
    });

    let mut text_input = TextArea::default();
    text_input.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Input"),
    );

    let mut text_messages = TextArea::default();
    text_messages.set_block(
        Block::default()
            .title("Messages")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded),
    );
    text_messages.set_cursor_style(Style::default().fg(Color::Black));

    loop {
        terminal.draw(|f| draw_ui(f, &mut text_input, &mut text_messages))?;

        if let Ok(Event::Key(k)) = event::read() {
            if k.kind == KeyEventKind::Press && k.code == KeyCode::Enter && !k.modifiers.contains(KeyModifiers::SHIFT) {
                let s = text_input.lines().join("\n");
                stx.send(s).await?;
                while text_input.delete_char() {}
            } else if k.kind == KeyEventKind::Press && k.code != KeyCode::Esc {
                text_input.input(to_input(k));
            } else if k.code == KeyCode::Esc {
                break;
            }
        }

        tokio::select! {
            Some(m) = rrx.recv() => {
                text_messages.insert_str(format!("{m}"));
                text_messages.insert_newline();
            },
            _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => {}
        }
    }

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    disable_raw_mode()?;

    Ok(())
}

fn draw_ui<B: Backend>(f: &mut Frame<B>, ta: &mut TextArea, msg: &mut TextArea) {
    let msg_widget = msg.widget();
    let widget = ta.widget();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(f.size());

    f.render_widget(msg_widget, chunks[0]);
    f.render_widget(widget, chunks[1]);
}

fn to_input(key: KeyEvent) -> Input {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let key = match key.code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Tab => Key::Tab,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Esc => Key::Esc,
        KeyCode::F(x) => Key::F(x),
        _ => Key::Null,
    };
    Input { key, ctrl, alt }
}

#[cfg(test)]
mod tests {
    use crate::message::Message;
    use crate::terminal::to_input;
    use crossterm::event::{KeyCode, KeyEventKind, EnableMouseCapture, self, Event, KeyEvent, DisableMouseCapture};
    use crossterm::execute;
    use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen};
    use tokio::sync::mpsc::channel;
    use tokio::{
        spawn,
        sync::mpsc::{Receiver, Sender},
    };
    use tui::Terminal;
    use tui::backend::CrosstermBackend;
    use tui::widgets::{Block, Borders};
    use tui_textarea::TextArea;

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

        spawn(async move {
            sender_test(srx).await;
        });
        spawn(async move {
            reciever_test(rtx).await;
        });

        _ = stx.send(Message::new("Aeskul", "Hello!")).await;
        while let Some(m) = rrx.recv().await {
            println!("From Reciever: {m:?}");
        }
    }

    #[test]
    fn tui_input_test() {
        enable_raw_mode().unwrap();
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut ta = TextArea::default();
        ta.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input")
                .border_type(tui::widgets::BorderType::Rounded),
        );
        let mut msg = TextArea::default();
        msg.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Messages")
                .border_type(tui::widgets::BorderType::Rounded),
        );
        let mut edit = false;
        loop {
            terminal
                .draw(|f| crate::terminal::draw_ui(f, &mut ta, &mut msg))
                .unwrap();

            if edit {
                if let Ok(Event::Key(k)) = event::read() {
                    if k.kind == KeyEventKind::Press && k.code != KeyCode::Esc {
                        ta.input(to_input(k));
                    } else if k.code == KeyCode::Esc {
                        edit = false;
                    }
                }
            } else {
                if let Ok(Event::Key(KeyEvent {
                    code: KeyCode::Char(k),
                    ..
                })) = event::read()
                {
                    if k == 'e' {
                        edit = true
                    };
                    if k == 'q' {
                        break;
                    };
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

    #[test]
    fn tui_msg_test() {
        enable_raw_mode().unwrap();
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        let m = Message::new("Aeskul", "Hello");
        let o = Message::new("Akachi", "I hate you");

        let mut ta = TextArea::default();
        ta.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input")
                .border_type(tui::widgets::BorderType::Rounded),
        );
        let mut msg = TextArea::default();
        msg.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Messages")
                .border_type(tui::widgets::BorderType::Rounded),
        );
        let mut edit = false;

        msg.insert_str(format!("{m}"));
        msg.insert_newline();
        msg.insert_str(format!("{o}"));
        msg.insert_newline();
        
        loop {
            terminal
                .draw(|f| crate::terminal::draw_ui(f, &mut ta, &mut msg))
                .unwrap();

            if edit {
                if let Ok(Event::Key(k)) = event::read() {
                    if k.kind == KeyEventKind::Press && k.code != KeyCode::Esc {
                        ta.input(to_input(k));
                    } else if k.code == KeyCode::Esc {
                        edit = false;
                    }
                }
            } else {
                if let Ok(Event::Key(KeyEvent {
                    code,
                    ..
                })) = event::read()
                {
                    match code {
                        KeyCode::Char('e') => {
                            edit = true
                        },
                        KeyCode::Char('q') => {
                            break;
                        },
                        KeyCode::Enter => {

                        },
                        _ => {}
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
