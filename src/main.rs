use prelude::*;
use tokio::task::*;
use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders},
    layout::{Layout, Constraint, Direction},
    Terminal
};
use std::{sync::OnceLock, io::Write};
use std::io;

mod prelude;
mod sender;
mod reciever;
mod message;

// Global variable for the terminal, so both threads can use it. It is OnceLock for thread safety.
pub static mut TERMINAL: OnceLock<Terminal<CrosstermBackend<io::Stdout>>> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<()> {
    spawn_blocking(move || {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        print!("Enter your username: ");
        io::stdout().flush().unwrap();
        let mut user = String::new();
        io::stdin().read_line(&mut user).unwrap();
        sender::USER.get_or_init(|| user.trim().to_owned());

        unsafe {
            TERMINAL.get_or_init(|| terminal);
        }
    }).await?;
    spawn(async {
        unsafe {
            if let Err(e) = sender::sender_loop().await {
                println!("ERROR: {e}");
                return;
            }
        }
    });
    spawn(async {
        if let Err(e) = reciever::reciever_loop().await {
            println!("ERROR: {e}");
            return;
        }
    });
    Ok(())
}
