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
    fn test_json_repr_greeting() {
        let msg = MessageKind::Greeting(GreetingMessage {
            motd: "This is the message of the day".into(),
        });

        let expected = json!({
            "kind": "greeting",
            "motd": "This is the message of the day"
        });

        assert_eq!(serde_json::to_value(&msg).unwrap(), expected);
    }

    #[test]
    fn test_json_repr_goodbye() {
        let msg = MessageKind::Goodbye(GoodbyeMessage { reason: None });

        let expected = json!({
            "kind": "goodbye",
        });

        assert_eq!(serde_json::to_value(&msg).unwrap(), expected);

        let msg = MessageKind::Goodbye(GoodbyeMessage {
            reason: Some("User quit.".into()),
        });

        let expected = json!({
            "kind": "goodbye",
            "reason": "User quit.",
        });

        assert_eq!(serde_json::to_value(&msg).unwrap(), expected);
    }
}
