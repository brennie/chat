use super::client::ClientMessageKind;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ServerMessage {
    /// A message that is being forwarded by the server from another client.
    FromClient {
        /// The source of the message.
        source: String,

        /// The content of the message.
        #[serde(flatten)]
        content: ClientMessageKind,
    },

    FromServer(ServerMessageKind),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ServerMessageKind {
    Greeting(GreetingMessage),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GreetingMessage {
    pub motd: String,
}

#[cfg(test)]
mod tests {
    use super::{*, ServerMessage::*};
    use super::super::client::*;

    use serde_json;

    #[test]
    fn test_serde() {
        let tests = vec![
            (
                FromServer(ServerMessageKind::Greeting(GreetingMessage {
                    motd: "Hello, world!".into(),
                })),
                json!({
                    "kind": "greeting",
                    "motd": "Hello, world!",
                }),
            ),
            (
                FromClient {
                    source: "user".into(),
                    content: ClientMessageKind::Goodbye(GoodbyeMessage { reason: None }),
                },
                json!({
                    "kind": "goodbye",
                    "source": "user",
                }),
            ),
            (
                FromClient {
                    source: "user".into(),
                    content: ClientMessageKind::Goodbye(GoodbyeMessage {
                        reason: Some("Goodbye, world.".into()),
                    }),
                },
                json!({
                    "kind": "goodbye",
                    "source": "user",
                    "reason": "Goodbye, world.",
                }),
            ),
        ];

        for (msg, expected_value) in tests {
            let serialized = serde_json::to_string(&msg).unwrap();

            assert_eq!(serde_json::to_value(&msg).unwrap(), expected_value);
            assert_eq!(
                serde_json::from_str::<ServerMessage>(&serialized).unwrap(),
                msg
            );
        }
    }
}