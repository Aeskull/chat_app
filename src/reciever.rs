use crate::prelude::*;

/// ### The main reciever function
/// 
/// It will be listening for incoming messages. If one is found, it will parse it and display it.
pub async fn reciever_loop() -> Result<()> {
    loop {
        // println!("Bae");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_test() {
        assert!("hello" == "hello");
    }
}