//! Prelude module providing convenient imports for Locust users.
//!
//! This module re-exports the most commonly used types and traits,
//! making it easy to get started with Locust:
//!
//! ```rust
//! use locust::prelude::*;
//! ```

// Core types
pub use crate::core::{Locust, LocustConfig, LocustContext};

// Plugin system
pub use crate::core::input::{LocustEventOutcome, PluginEventResult};
pub use crate::core::plugin::LocustPlugin;

// Navigation and targets
pub use crate::core::targets::{
    NavTarget, TargetAction, TargetBuilder, TargetPriority, TargetRegistry, TargetState,
};

// Overlay management
pub use crate::core::overlay::{OverlayLayer, OverlayState};

// Built-in plugins
pub use crate::plugins::nav::{NavConfig, NavMode, NavPlugin};

// Re-export commonly used ratatui types
pub use ratatui::backend::Backend;
pub use ratatui::Frame;

// Re-export crossterm event types
pub use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
