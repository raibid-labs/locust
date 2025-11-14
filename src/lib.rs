//! Locust: A ratatui plugin framework for overlay management.
//!
//! Locust provides a robust plugin system for building keyboard-driven,
//! overlay-based interactions in ratatui applications. Think Vimium for
//! your terminal UI.
//!
//! # Quick Start
//!
//! ```ignore
//! use locust::prelude::*;
//! use ratatui::backend::CrosstermBackend;
//!
//! let mut locust = Locust::<CrosstermBackend<_>>::new(LocustConfig::default());
//! locust.register_plugin(NavPlugin::new());
//!
//! // In your event loop:
//! locust.begin_frame();
//! let outcome = locust.on_event(&event);
//! // ... render your app ...
//! locust.render_overlay(&mut frame);
//! ```
//!
//! # Architecture
//!
//! - **Locust**: Main coordinator that manages plugins and context
//! - **LocustPlugin**: Trait for implementing custom plugins
//! - **LocustContext**: Shared state (targets, overlays, etc.)
//! - **NavTarget**: Navigable UI elements registered during rendering
//!
//! # Features
//!
//! - Plugin priority system for event handling order
//! - Frame lifecycle management with hooks
//! - Navigation target registry with filtering
//! - Z-layered overlay rendering
//! - Thread-safe design (when wrapped appropriately)

pub mod core;
pub mod plugins;
pub mod prelude;
pub mod ratatui_ext;

pub use core::{Locust, LocustConfig, LocustContext};
