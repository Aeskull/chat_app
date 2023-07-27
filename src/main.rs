use prelude::*;
use tokio::task::*;

mod prelude;
mod sender;
mod reciever;
mod message;
mod terminal;

#[tokio::main]
async fn main() -> Result<()> {
    let mut s = String::new();
    println!("Enter your username:");
    std::io::stdin().read_line(&mut s)?;
    let user = s.trim().to_owned();

    s.clear();
    println!("And what is the ip of the server you will be joining?");
    std::io::stdin().read_line(&mut s)?;
    let ip = s.trim().to_owned();

    spawn(async {
        if let Err(e) = terminal::terminal_loop(user, ip).await {
            println!("ERROR: {e}");
            return;
        }
    }).await?;
    Ok(())
}
