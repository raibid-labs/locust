// Test modules
mod unit {
    mod core_types;
    mod fuzzy_matcher;
    mod omnibar_state;
}

mod integration {
    mod omnibar_plugin;
    mod plugin_lifecycle;
}

mod examples {
    mod example_plugin;
}

// Property-based tests (requires proptest)
#[cfg(test)]
mod property;
