mod cli_config;
pub mod player;
pub mod utils;
pub mod view;
pub use anyhow::{Result as AnyResult, anyhow};
pub use cli_config::Args;
pub use player::*;
pub use utils::*;
pub use view::*;
