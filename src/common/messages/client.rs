#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ClientMessageKind {
    Goodbye(GoodbyeMessage),
}


#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GoodbyeMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json;

    use super::{*, ClientMessageKind::*};

    #[test]
    fn test_serde() {
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
}