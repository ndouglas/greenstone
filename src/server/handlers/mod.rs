mod health;
pub use health::health_handler;

mod registration;
pub use registration::registration_handler;

mod deregistration;
pub use deregistration::deregistration_handler;

mod websocket;
pub use websocket::websocket_handler;
