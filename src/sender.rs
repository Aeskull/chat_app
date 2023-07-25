use crate::{prelude::*, TERMINAL};
use crossterm::terminal;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdin, stdout, Write};
use std::sync::OnceLock;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Widget},
    Terminal,
};

pub static USER: OnceLock<String> = OnceLock::new();

/// Macro to get the input in a concise manor.
macro_rules! get {
    ($x: expr) => {{
        print!("> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut $x)
    }};
}

/// ### The main Sender loop.
///
/// Loops ad infinitum. It will handle input, parsing of input, and passing the data along to be sent.
pub async unsafe fn sender_loop() -> Result<()> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let terminal = TERMINAL.get_mut().unwrap();

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title("Block").borders(Borders::ALL);
        f.render_widget(block, size);
    })?;
    std::thread::sleep(std::time::Duration::from_millis(5000));

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
    use crate::message::Message;
    use std::io::{stdin, stdout, Write};

    #[test]
    fn input_test() {
        let mut user = String::new();
        _ = get!(&mut user);
        let mut content = String::new();
        _ = get!(&mut content);

        let constructed_msg = format!("{} @ : {}", user.trim(), content.trim()).len() + 16;
        let m = Message::new(user.trim(), content.trim());
        assert_eq!(format!("{m}").len(), constructed_msg)
    }
}
