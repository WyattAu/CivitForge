#![forbid(unsafe_code)]

pub mod bus;
pub mod model;
pub mod websocket;

pub use bus::{EventBus, EventSubscriber};
pub use model::{Event, EventCategory, EventPayload};
pub use websocket::WebSocketManager;
