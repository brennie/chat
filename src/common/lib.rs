extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GreetingMessage {
    pub motd: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GoodbyeMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "kind")]
pub enum MessageKind {
    Greeting(GreetingMessage),
    Goodbye(GoodbyeMessage),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {

        let tests = vec![
            (
                MessageKind::Greeting(GreetingMessage {
                    motd: "This is the message of the day".into(),
                }),
                json!({
                    "kind": "greeting",
                    "motd": "This is the message of the day",
                }),
            ),
            (
                MessageKind::Goodbye(GoodbyeMessage{
                    reason: Some("User quit".into()),
                }),
                json!({
                    "kind": "goodbye",
                    "reason": "User quit",
                }),
            ),
            (
                MessageKind::Goodbye(GoodbyeMessage {
                    reason: None,
                }),
                json!({"kind": "goodbye"})
            )
        ];

        for (msg, expected) in tests {
            let serialized = serde_json::to_string(&msg).unwrap();

            assert_eq!(serde_json::to_value(&msg).unwrap(), expected);
            assert_eq!(serde_json::from_str::<MessageKind>(&serialized).unwrap(), msg);
        }

    }
}
