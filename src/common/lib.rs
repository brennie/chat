extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

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
pub enum ClientMessageKind {
    Goodbye(GoodbyeMessage),
    AuthRequest(AuthRequestMessage),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ServerMessageKind {
    Greeting(GreetingMessage),
    AuthResponse(AuthResponseMessage),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GoodbyeMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GreetingMessage {
    pub motd: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuthRequestMessage {
    pub username: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuthResponseMessage {
    pub result: Result<String, String>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_client_message() {
        use ClientMessageKind::*;

        let tests = vec![
            (
                Goodbye(GoodbyeMessage { reason: None }),
                json!({
                    "kind": "goodbye",
                }),
            ),
            (
                Goodbye(GoodbyeMessage {
                    reason: Some("user quit".into()),
                }),
                json!({
                    "kind": "goodbye",
                    "reason": "user quit",
                }),
            ),
            (
                AuthRequest(AuthRequestMessage {
                    username: "foo".into(),
                }),
                json!({
                    "kind": "auth_request",
                    "username": "foo",
                }),
            ),
        ];

        for (msg, expected_value) in tests {
            let serialized = serde_json::to_string(&msg).unwrap();

            assert_eq!(serde_json::to_value(&msg).unwrap(), expected_value);
            assert_eq!(
                serde_json::from_str::<ClientMessageKind>(&serialized).unwrap(),
                msg
            );
        }
    }

    #[test]
    fn test_serde_server_message() {
        use ServerMessage::*;

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
            (
                FromServer(ServerMessageKind::AuthResponse(AuthResponseMessage {
                    result: Ok("username".into()),
                })),
                json!({
                    "kind": "auth_response",
                    "result": {
                        "Ok": "username",
                    },
                }),
            ),
            (
                FromServer(ServerMessageKind::AuthResponse(AuthResponseMessage {
                    result: Err("Invalid username".into()),
                })),
                json!({
                    "kind": "auth_response",
                    "result": {
                        "Err": "Invalid username",
                    },
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
