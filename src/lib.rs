pub mod commands;

mod context;
mod db;
mod error;
mod logger;
mod revisions;

pub use commands::*;
pub use context::config::Config;
pub use context::environment::Environment;
pub use error::Error;
pub use logger::Logger;

pub(crate) use db::executor::Executor;

// Crate result type
pub type Result<T> = std::result::Result<T, Error>;

/// The default name of the config file
pub const CONF: &str = "jrny.toml";

/// The default name of the environment file
pub const ENV: &str = "jrny-env.toml";

/// The default name of the example environment file
pub const ENV_EX: &str = "jrny-env.example.toml";
