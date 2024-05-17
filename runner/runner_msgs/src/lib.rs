use serde::{Deserialize, Serialize};
use strum::AsRefStr;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone, AsRefStr)]
#[serde(tag = "msg", content = "data")]
pub enum ConsoleMessage {
    CommitRender,
    PauseRendering,
    ResumeRendering,
    NextApp,
    PreviousApp,
    SetMatrixRowRgb(SetMatrixRowRgb),
    RequestAppListing(RequestAppListing),
    AppListingResponse(AppListingResponse),
    #[cfg(test)]
    TestMessage(TestMessage),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetMatrixRowRgb {
    pub row: usize,
    pub data: Vec<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestAppListing {
    pub request_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppListingResponse {
    pub request_id: String,
    pub apps: Vec<App>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct App {
    pub md5sum: String,
    pub app_name: String,
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
