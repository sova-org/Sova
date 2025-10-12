pub mod clock;
pub mod compiler;
pub mod device_map;
pub mod lang;
pub mod logger;
pub mod protocol;
pub mod scene;
pub mod schedule;
pub mod server;
pub mod util;
pub mod world;
pub mod init;

pub use protocol::TimedMessage;
pub use scene::Scene;

// Re-export logging functionality
pub use logger::{
    init_standalone, init_embedded, init_network, set_embedded_mode, set_network_mode, set_dual_mode, set_standalone_mode,
    create_log_channel, get_logger, Logger, LoggerMode,
};

// Re-export protocol log types
pub use protocol::log::{LogMessage, Severity};

