pub mod clock;
pub mod compiler;
pub mod device_map;
pub mod init;
pub mod logger;
pub mod protocol;
pub mod scene;
pub mod schedule;
pub mod util;
pub mod vm;
pub mod world;
pub mod error;

pub use protocol::TimedMessage;
pub use scene::Scene;

// Re-export logging functionality
pub use logger::{
    Logger, LoggerMode, create_log_channel, get_logger, init_embedded, init_network,
    init_standalone, set_dual_mode, set_embedded_mode, set_network_mode, set_standalone_mode,
};

// Re-export protocol log types
pub use protocol::log::{LogMessage, Severity};
