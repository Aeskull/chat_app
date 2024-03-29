use serde::{Deserialize, Serialize};

/// ### Message
///
/// A structure that represents a message sent by a user.
///
/// Each Message contains the name of the user who sent it, the time it was sent, and the payload (contents of the message).
///
/// Derives Serialize and Deserialize for easy transmission.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    from: String,
    time: String,
    payload: String,
}

impl Message {
    /// Constructs a new Message.
    ///
    /// Parameters:
    /// ```
    ///     user: &str // The string representing the user who sent the message.
    ///     content: &str // The string representing the contents of the message.
    /// ```
    /// Returns a new Message structure
    pub fn new(user: &str, payload: &str) -> Self {
        let now = chrono::offset::Local::now()
            .format("%H:%M | %Y %d %m")
            .to_string();
        Self {
            from: user.to_owned(),
            time: now,
            payload: payload.to_owned(),
        }
    }

    /// Get's the length of the header of the message (The username and time)
    pub fn get_header(&self) -> String {
        format!("{} @ {}: ", self.from, self.time)
    }
}

/// Implement Display for Message
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.payload)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn json_test() {
        let m = crate::message::Message::new("Aeskul", "Hello there!");
        let j = serde_json::to_string_pretty(&m).unwrap();
        println!("{j}");

        assert_eq!(m.time.len(), 16);
    }
}
