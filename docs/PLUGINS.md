# Locust Plugins

Locust plugins extend the framework with new overlay behaviors, such as:

- Vimium-style hint navigation (the built-in `NavPlugin`).
- Omnibar / command palette.
- Tooltip or popover rendering.
- Guided tours / onboarding overlays.

## Implementing a Plugin

A plugin implements the `LocustPlugin<B>` trait:

```rust
use locust::core::context::LocustContext;
use locust::core::input::PluginEventResult;
use locust::core::plugin::LocustPlugin;
use crossterm::event::Event;
use ratatui::backend::Backend;
use ratatui::prelude::Frame;

pub struct MyPlugin;

impl<B> LocustPlugin<B> for MyPlugin
where
    B: Backend,
{
    fn id(&self) -> &'static str {
        "example.my-plugin"
    }

    fn init(&mut self, _ctx: &mut LocustContext) {
        // optional initialization
    }

    fn on_event(
        &mut self,
        event: &Event,
        ctx: &mut LocustContext,
    ) -> PluginEventResult {
        // inspect event, maybe mutate ctx, and decide:
        PluginEventResult::NotHandled
    }

    fn render_overlay(&self, frame: &mut Frame<'_, B>, ctx: &LocustContext) {
        // draw overlay widgets on top of the existing UI
    }
}
```

## Registering Plugins

In your application:

```rust
let mut locust = Locust::new(LocustConfig::default());
locust.register_plugin(NavPlugin::new());      // built-in navigation
locust.register_plugin(MyPlugin);             // your custom plugin
```

Plugins are called in registration order.
