/// State management for overlay rendering.
///
/// Tracks which overlays are active, their z-order, and per-frame
/// rendering state. Overlays are rendered in layer order after the
/// base application UI has been drawn.
///
/// # Z-Layers
///
/// Lower z-index values render first (bottom layers), higher values
/// render last (top layers). Standard layers:
/// - 0-99: Background overlays
/// - 100-199: Normal overlays (default)
/// - 200-299: Modal dialogs
/// - 300+: Critical notifications
#[derive(Debug, Default)]
pub struct OverlayState {
    /// Whether any plugin has requested overlay rendering this frame.
    pub has_overlay: bool,

    /// Active overlay layers by plugin ID.
    layers: Vec<OverlayLayer>,

    /// Total number of frames where overlays were active.
    pub total_overlay_frames: u64,
}

/// Represents a single overlay layer from a plugin.
#[derive(Debug, Clone)]
pub struct OverlayLayer {
    /// Plugin ID that owns this layer.
    pub plugin_id: String,

    /// Z-index for rendering order (lower = bottom, higher = top).
    pub z_index: i32,

    /// Whether this layer is currently visible.
    pub visible: bool,
}

impl OverlayLayer {
    /// Create a new overlay layer.
    pub fn new(plugin_id: impl Into<String>, z_index: i32) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            z_index,
            visible: true,
        }
    }
}

impl OverlayState {
    /// Create a new overlay state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset state at the start of each frame.
    pub fn begin_frame(&mut self) {
        self.has_overlay = false;
    }

    /// Mark that at least one overlay is active this frame.
    pub fn mark_has_overlay(&mut self) {
        if !self.has_overlay {
            self.has_overlay = true;
            self.total_overlay_frames += 1;
        }
    }

    /// Register an overlay layer for a plugin.
    pub fn add_layer(&mut self, layer: OverlayLayer) {
        // Remove existing layer from same plugin if present
        self.layers.retain(|l| l.plugin_id != layer.plugin_id);
        self.layers.push(layer);
        // Keep sorted by z-index
        self.layers.sort_by_key(|l| l.z_index);
    }

    /// Remove an overlay layer by plugin ID.
    pub fn remove_layer(&mut self, plugin_id: &str) {
        self.layers.retain(|l| l.plugin_id != plugin_id);
    }

    /// Get all active overlay layers in render order.
    pub fn layers(&self) -> &[OverlayLayer] {
        &self.layers
    }

    /// Set visibility for a specific plugin's overlay.
    pub fn set_layer_visibility(&mut self, plugin_id: &str, visible: bool) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.plugin_id == plugin_id) {
            layer.visible = visible;
        }
    }

    /// Check if a plugin has an active overlay layer.
    pub fn has_layer(&self, plugin_id: &str) -> bool {
        self.layers
            .iter()
            .any(|l| l.plugin_id == plugin_id && l.visible)
    }

    /// Clear all overlay layers.
    pub fn clear_layers(&mut self) {
        self.layers.clear();
    }
}
