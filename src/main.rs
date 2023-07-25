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
mod terminal;

#[tokio::main]
async fn main() -> Result<()> {
    spawn(async {
        if let Err(e) = terminal::terminal_loop().await {
            println!("ERROR: {e}");
            return;
        }
    }).await?;
    Ok(())
}
