use {prelude::*, tokio::task::*};

mod message;
mod prelude;
mod sender;
mod terminal;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the alleged username of the user
    let mut s = String::new();
    println!("Enter your username:");
    std::io::stdin().read_line(&mut s)?;
    let user = s.trim().to_owned();
    s.clear();

    // Get the target ip for the server.
    println!("And what is the ip of the server you will be joining?");
    std::io::stdin().read_line(&mut s)?;
    let ip = s.trim().to_owned();

    // Spawn terminal thread
    spawn(async {
        if let Err(e) = terminal::terminal_loop(user, ip).await {
            println!("{}", e.message());
        }
    })
    .await?;
    Ok(())
}
