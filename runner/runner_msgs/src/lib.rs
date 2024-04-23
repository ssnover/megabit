use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "msg", content = "data")]
pub enum ConsoleMessage {
    PauseRendering,
    ResumeRendering,
    NextApp,
    PreviousApp,
    #[cfg(test)]
    TestMessage(TestMessage),
}

#[cfg(test)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestMessage {
    a: u32,
    b: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_no_args() {
        assert_eq!(
            serde_json::to_string(&ConsoleMessage::PauseRendering).unwrap(),
            r#"{"msg":"PauseRendering"}"#
        );
    }

    #[test]
    fn test_serialize_with_args() {
        assert_eq!(
            serde_json::to_string(&ConsoleMessage::TestMessage(TestMessage {
                a: 42,
                b: String::from("hello world")
            }))
            .unwrap(),
            r#"{"msg":"TestMessage","data":{"a":42,"b":"hello world"}}"#
        );
    }
}
