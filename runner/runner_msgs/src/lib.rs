use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConsoleMessage {
    PauseRendering,
    ResumeRendering,
    NextApp,
    PreviousApp,
}
