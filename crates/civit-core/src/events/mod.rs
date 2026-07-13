#![forbid(unsafe_code)]

pub mod bus;
pub mod log_stream;
pub mod model;
pub mod websocket;
pub mod websocket_scaler;

pub use bus::{EventBus, EventSubscriber};
pub use log_stream::{LogBroadcaster, LogStreamEvent, PipelineStatusEvent};
pub use model::{Event, EventCategory, EventPayload};
pub use websocket::WebSocketManager;
pub use websocket_scaler::WebSocketScaler;
