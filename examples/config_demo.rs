//! Configuration system demonstration.
//!
//! This example shows how to:
//! - Load configuration from TOML/JSON files
//! - Update configuration at runtime
//! - Access plugin-specific configuration
//! - Use hot reload support
//!
//! Run with: cargo run --example config_demo

use locust::core::config::*;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Locust Configuration Demo ===\n");

    // 1. Create default configuration
    println!("1. Creating default configuration...");
    let mut config = LocustConfig::new();
    println!("   Default FPS limit: {:?}", config.global.fps_limit);
    println!("   Mouse support: {}", config.global.mouse_support);
    println!();

    // 2. Update global settings
    println!("2. Updating global settings...");
    config.global.fps_limit = Some(144);
    config.global.enable_logging = true;
    config.global.log_level = LogLevel::Debug;
    println!("   New FPS limit: {:?}", config.global.fps_limit);
    println!("   Logging enabled: {}", config.global.enable_logging);
    println!();

    // 3. Add plugin configurations
    println!("3. Adding plugin configurations...");

    let nav_config = NavConfig {
        hint_key: 'g',
        charset: "asdfjkl".to_string(),
        min_target_area: 10,
        max_hints: 75,
    };
    config.update_plugin("nav", nav_config)?;
    println!("   Nav plugin configured: hint_key='g', max_hints=75");

    let omnibar_config = OmnibarConfig {
        activation_key: ':',
        activation_modifiers: vec!["Ctrl".to_string(), "Shift".to_string()],
        max_results: 15,
        fuzzy_threshold: 0.8,
    };
    config.update_plugin("omnibar", omnibar_config)?;
    println!("   Omnibar plugin configured: key=':', modifiers=Ctrl+Shift");

    let tooltip_config = TooltipConfig {
        default_delay_ms: 300,
        default_position: TooltipPosition::Below,
        auto_hide_ms: 5000,
    };
    config.update_plugin("tooltip", tooltip_config)?;
    println!("   Tooltip plugin configured: delay=300ms, position=Below");

    let highlight_config = HighlightConfig {
        default_animation: HighlightAnimation::Shimmer,
        dim_opacity: 0.8,
        border_thickness: 3,
    };
    config.update_plugin("highlight", highlight_config)?;
    println!("   Highlight plugin configured: animation=Shimmer");
    println!();

    // 4. Validate configuration
    println!("4. Validating configuration...");
    let errors = config.validate();
    if errors.is_empty() {
        println!("   ✓ Configuration is valid!");
    } else {
        println!("   Found {} validation issues:", errors.len());
        for error in &errors {
            let severity = match error.severity {
                Severity::Error => "ERROR",
                Severity::Warning => "WARN",
            };
            println!("   [{}] {}: {}", severity, error.field, error.message);
        }
    }
    println!();

    // 5. Save to TOML file
    println!("5. Saving configuration to TOML...");
    let toml_path = Path::new("demo_config.toml");
    config.save_to(toml_path)?;
    println!("   Saved to: {}", toml_path.display());
    println!();

    // 6. Load from file
    println!("6. Loading configuration from file...");
    let loaded = LocustConfig::from_file(toml_path)?;
    println!("   Loaded FPS limit: {:?}", loaded.global.fps_limit);
    println!("   Plugin count: {}", loaded.plugins.len());
    println!();

    // 7. Access plugin configuration
    println!("7. Accessing plugin configurations...");
    if let Some(nav) = loaded.get_plugin_config::<NavConfig>("nav") {
        println!("   Nav hint_key: '{}'", nav.hint_key);
        println!("   Nav charset: '{}'", nav.charset);
        println!("   Nav max_hints: {}", nav.max_hints);
    }
    if let Some(omnibar) = loaded.get_plugin_config::<OmnibarConfig>("omnibar") {
        println!("   Omnibar activation_key: '{}'", omnibar.activation_key);
        println!("   Omnibar max_results: {}", omnibar.max_results);
    }
    println!();

    // 8. Demonstrate hot reload
    println!("8. Demonstrating hot reload...");
    let mut watcher = ConfigWatcher::new(toml_path.to_path_buf());
    println!("   Initial check: {}", watcher.check_for_changes());

    // Simulate file modification
    println!("   Modifying configuration file...");
    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut modified = loaded;
    modified.global.fps_limit = Some(240);
    modified.save_to(toml_path)?;

    println!("   Checking for changes: {}", watcher.check_for_changes());
    println!("   Checking again: {}", watcher.check_for_changes());
    println!();

    // 9. Save as JSON
    println!("9. Saving configuration as JSON...");
    let json_path = Path::new("demo_config.json");
    config.save_to(json_path)?;
    println!("   Saved to: {}", json_path.display());
    println!();

    // 10. Demonstrate validation errors
    println!("10. Demonstrating validation errors...");
    let mut invalid_config = LocustConfig::new();
    invalid_config.global.fps_limit = Some(0);

    let nav_bad = NavConfig {
        hint_key: 'f',
        charset: String::new(), // Empty charset - invalid!
        min_target_area: 1,
        max_hints: 0,
    };
    invalid_config.update_plugin("nav", nav_bad)?;

    let errors = invalid_config.validate();
    println!("   Found {} validation errors:", errors.len());
    for error in &errors {
        let severity = match error.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
        };
        println!("   [{}] {}: {}", severity, error.field, error.message);
    }
    println!();

    // Cleanup
    println!("Cleaning up demo files...");
    std::fs::remove_file(toml_path).ok();
    std::fs::remove_file(json_path).ok();

    println!("\n=== Demo Complete ===");

    Ok(())
}
