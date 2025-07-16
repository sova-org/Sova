pub mod clock;
pub mod compiler;
pub mod device_map;
pub mod lang;
pub mod logger;
pub mod protocol;
pub mod relay_client;
pub mod scene;
pub mod schedule;
pub mod server;
pub mod shared_types;
pub mod transcoder;
pub mod util;
pub mod world;

pub use protocol::message::TimedMessage;
pub use scene::Scene;
pub use shared_types::GridSelection;

// Re-export logging functionality
pub use logger::{
    init_standalone, init_embedded, set_embedded_mode, set_standalone_mode,
    create_log_channel, get_logger, Logger, LoggerMode,
};

// Re-export protocol log types
pub use protocol::log::{LogMessage, Severity};

