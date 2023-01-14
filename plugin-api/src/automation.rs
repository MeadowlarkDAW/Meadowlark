use clack_host::utils::Cookie;

// Contents of AutomationBuffer
#[derive(Copy, Clone)]
pub struct AutomationIoEvent {
    pub header: IoEventHeader,
    pub parameter_id: u32,
    pub event_type: AutomationIoEventType,
    pub plugin_instance_id: u64,
    pub cookie: Option<Cookie>,
}

// Contains common data
#[derive(Copy, Clone)]
pub struct IoEventHeader {
    pub time: u32,
    // TODO: add event flags here when we implement them
}

#[derive(Copy, Clone)]
pub enum AutomationIoEventType {
    Value(f64),
    Modulation(f64),
    BeginGesture,
    EndGesture,
}
