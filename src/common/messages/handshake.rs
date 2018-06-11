/// An authentication request.
///
/// This currently exists as an enum due to an [issue in serde][1].
///
/// [1]: https://github.com/serde-rs/serde/issues/271
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum AuthRequest {
    AuthRequest { username: String },
}

/// An authentication response.
///
/// This is a response to [`AuthRequest`].
///
/// This currently exists as an enum due to an [issue in serde][1].
///
/// [1]: https://github.com/serde-rs/serde/issues/271
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum AuthResponse {
    AuthResponse {
        /// Whether or not authentication was successful.
        ///
        /// The `Ok` is the username.
        /// The `Err` is a string representation of why the authentication failed.
        #[serde(flatten)]
        result: Result<String, String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json;

    #[test]
    fn test_serde_auth_request_message() {
        let msg = AuthRequest::AuthRequest {
            username: "wiz".into(),
        };

        let expected_value = json!({
            "kind": "auth_request",
            "username": "wiz",
        });

        let serialized = serde_json::to_string(&msg).unwrap();

        assert_eq!(serde_json::to_value(&msg).unwrap(), expected_value);
        assert_eq!(
            serde_json::from_str::<AuthRequest>(&serialized).unwrap(),
            msg
        );
    }

    #[test]
    fn test_serde_auth_response_message() {
        let tests = vec![
            (
                AuthResponse::AuthResponse {
                    result: Ok("wiz".into()),
                },
                json!({
                    "kind": "auth_response",
                    "Ok": "wiz",
                }),
            ),
            (
                AuthResponse::AuthResponse {
                    result: Err("Invalid username.".into()),
                },
                json!({
                    "kind": "auth_response",
                    "Err": "Invalid username.",
                }),
            ),
        ];

        for (msg, expected_value) in tests {
            let serialized = serde_json::to_string(&msg).unwrap();
            assert_eq!(serde_json::to_value(&msg).unwrap(), expected_value);
            assert_eq!(
                serde_json::from_str::<AuthResponse>(&serialized).unwrap(),
                msg
            );
        }
    }
}
