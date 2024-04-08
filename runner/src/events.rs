pub enum Event {
    NextAppRequest,
    PreviousAppRequest,
    ResumePauseRequest,
    ReloadAppsRequest,
}

pub struct EventListener {}

impl EventListener {
    pub fn new() -> Self {
        Self {}
    }

    pub fn has_pending_events(&self) -> bool {
        false
    }

    pub fn next(&self) -> Option<Event> {
        None
    }
}
