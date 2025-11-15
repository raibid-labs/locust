//! State Management Integration Example
//!
//! This example demonstrates various patterns for integrating Locust
//! with different state management approaches:
//!
//! 1. Message Passing (Elm-like architecture)
//! 2. Shared State (Arc<RwLock<T>>)
//! 3. Event Sourcing
//! 4. Component-based State
//!
//! Run with: cargo run --example state_management

use std::collections::{HashMap, VecDeque};
use std::sync::{mpsc, Arc, RwLock};
use std::time::Instant;

// ============================================================================
// Pattern 1: Message Passing (Elm-like)
// ============================================================================

#[derive(Debug, Clone)]
enum Message {
    UserInput(String),
    LocustNavActivated(String),
    LocustCommandExecuted(String),
    SelectItem(usize),
    ToggleItem(usize),
    AddItem(String),
    DeleteItem(usize),
}

struct MessagePassingApp {
    state: AppState,
    message_rx: mpsc::Receiver<Message>,
    message_tx: mpsc::Sender<Message>,
}

#[derive(Clone, Debug)]
struct AppState {
    items: Vec<TodoItem>,
    selected: usize,
}

#[derive(Clone, Debug)]
struct TodoItem {
    text: String,
    completed: bool,
}

impl MessagePassingApp {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            state: AppState {
                items: vec![
                    TodoItem {
                        text: "Learn Locust".to_string(),
                        completed: false,
                    },
                    TodoItem {
                        text: "Integrate with app".to_string(),
                        completed: false,
                    },
                ],
                selected: 0,
            },
            message_rx: rx,
            message_tx: tx,
        }
    }

    fn sender(&self) -> mpsc::Sender<Message> {
        self.message_tx.clone()
    }

    fn update(&mut self) {
        // Process all pending messages
        while let Ok(msg) = self.message_rx.try_recv() {
            self.handle_message(msg);
        }
    }

    fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::SelectItem(index) => {
                if index < self.state.items.len() {
                    self.state.selected = index;
                }
            }
            Message::ToggleItem(index) => {
                if index < self.state.items.len() {
                    self.state.items[index].completed = !self.state.items[index].completed;
                }
            }
            Message::AddItem(text) => {
                self.state.items.push(TodoItem {
                    text,
                    completed: false,
                });
            }
            Message::DeleteItem(index) => {
                if index < self.state.items.len() {
                    self.state.items.remove(index);
                    if self.state.selected >= self.state.items.len() && self.state.selected > 0 {
                        self.state.selected -= 1;
                    }
                }
            }
            Message::LocustNavActivated(target_id) => {
                // Parse target_id like "item_2"
                if let Some(index_str) = target_id.strip_prefix("item_") {
                    if let Ok(index) = index_str.parse::<usize>() {
                        self.handle_message(Message::SelectItem(index));
                    }
                }
            }
            Message::LocustCommandExecuted(cmd_id) => match cmd_id.as_str() {
                "todo.toggle" => {
                    self.handle_message(Message::ToggleItem(self.state.selected));
                }
                "todo.delete" => {
                    self.handle_message(Message::DeleteItem(self.state.selected));
                }
                _ => {}
            },
            _ => {}
        }
    }
}

// Integration plugin for message passing
// struct MessagePassingIntegration {
//     message_tx: mpsc::Sender<Message>,
// }
//
// impl<B: Backend> LocustPlugin<B> for MessagePassingIntegration {
//     fn id(&self) -> &'static str { "app.message_passing" }
//
//     fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
//         // Forward Locust events as messages
//         if let Some(target_id) = ctx.get_data::<String>("nav_activated") {
//             let _ = self.message_tx.send(Message::LocustNavActivated(target_id.clone()));
//             ctx.remove_data("nav_activated");
//         }
//
//         if let Some(cmd_id) = ctx.get_data::<String>("omnibar_executed") {
//             let _ = self.message_tx.send(Message::LocustCommandExecuted(cmd_id.clone()));
//             ctx.remove_data("omnibar_executed");
//         }
//
//         PluginEventResult::NotHandled
//     }
//
//     fn render_overlay(&self, _frame: &mut Frame, _ctx: &LocustContext) {}
// }

// ============================================================================
// Pattern 2: Shared State with Arc<RwLock<T>>
// ============================================================================

#[derive(Clone, Debug)]
struct SharedAppState {
    items: Vec<String>,
    selected: usize,
    view_mode: ViewMode,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
enum ViewMode {
    List,
    Grid,
    Details,
}

struct SharedStateApp {
    state: Arc<RwLock<SharedAppState>>,
}

impl SharedStateApp {
    fn new() -> Self {
        let state = Arc::new(RwLock::new(SharedAppState {
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            selected: 0,
            view_mode: ViewMode::List,
        }));

        Self { state }
    }

    fn state(&self) -> Arc<RwLock<SharedAppState>> {
        self.state.clone()
    }

    fn select_item(&self, index: usize) {
        if let Ok(mut state) = self.state.write() {
            if index < state.items.len() {
                state.selected = index;
            }
        }
    }

    fn set_view_mode(&self, mode: ViewMode) {
        if let Ok(mut state) = self.state.write() {
            state.view_mode = mode;
        }
    }
}

// Integration plugin for shared state
// struct SharedStateIntegration {
//     state: Arc<RwLock<SharedAppState>>,
// }
//
// impl<B: Backend> LocustPlugin<B> for SharedStateIntegration {
//     fn id(&self) -> &'static str { "app.shared_state" }
//
//     fn on_frame_begin(&mut self, ctx: &mut LocustContext) {
//         // Sync state to context for other plugins
//         if let Ok(state) = self.state.read() {
//             ctx.store_data("app_state", Box::new(state.clone()));
//         }
//     }
//
//     fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
//         // Handle Locust events and update shared state
//         if let Some(target_id) = ctx.get_data::<String>("nav_activated") {
//             if let Some(index_str) = target_id.strip_prefix("item_") {
//                 if let Ok(index) = index_str.parse::<usize>() {
//                     if let Ok(mut state) = self.state.write() {
//                         state.selected = index;
//                     }
//                 }
//             }
//             ctx.remove_data("nav_activated");
//         }
//
//         PluginEventResult::NotHandled
//     }
//
//     fn render_overlay(&self, _frame: &mut Frame, _ctx: &LocustContext) {}
// }

// ============================================================================
// Pattern 3: Event Sourcing
// ============================================================================

#[derive(Clone, Debug)]
enum AppEvent {
    ItemAdded { id: usize, text: String },
    ItemToggled { id: usize },
    ItemDeleted { id: usize },
    ItemSelected { id: usize },
    ViewChanged { mode: ViewMode },
}

struct EventSourcedApp {
    events: Vec<(Instant, AppEvent)>,
    state: EventSourcedState,
}

#[derive(Clone, Debug)]
struct EventSourcedState {
    items: HashMap<usize, TodoItem>,
    next_id: usize,
    selected: Option<usize>,
}

impl EventSourcedApp {
    fn new() -> Self {
        let mut app = Self {
            events: Vec::new(),
            state: EventSourcedState {
                items: HashMap::new(),
                next_id: 0,
                selected: None,
            },
        };

        // Initialize with some events
        app.apply_event(AppEvent::ItemAdded {
            id: 0,
            text: "First item".to_string(),
        });
        app.apply_event(AppEvent::ItemAdded {
            id: 1,
            text: "Second item".to_string(),
        });

        app
    }

    fn apply_event(&mut self, event: AppEvent) {
        // Store event for history/replay
        self.events.push((Instant::now(), event.clone()));

        // Apply event to state
        match event {
            AppEvent::ItemAdded { id, text } => {
                self.state.items.insert(
                    id,
                    TodoItem {
                        text,
                        completed: false,
                    },
                );
                if self.state.next_id <= id {
                    self.state.next_id = id + 1;
                }
            }
            AppEvent::ItemToggled { id } => {
                if let Some(item) = self.state.items.get_mut(&id) {
                    item.completed = !item.completed;
                }
            }
            AppEvent::ItemDeleted { id } => {
                self.state.items.remove(&id);
                if self.state.selected == Some(id) {
                    self.state.selected = None;
                }
            }
            AppEvent::ItemSelected { id } => {
                self.state.selected = Some(id);
            }
            _ => {}
        }
    }

    fn replay_events(&mut self, from_index: usize) {
        // Replay events from a specific point
        let events: Vec<_> = self.events[from_index..]
            .iter()
            .map(|(_, e)| e.clone())
            .collect();

        for event in events {
            self.apply_event(event);
        }
    }

    fn undo(&mut self) {
        if !self.events.is_empty() {
            self.events.pop();
            // Rebuild state from events
            self.state = EventSourcedState {
                items: HashMap::new(),
                next_id: 0,
                selected: None,
            };
            self.replay_events(0);
        }
    }
}

// Integration plugin for event sourcing
// struct EventSourcedIntegration {
//     event_tx: mpsc::Sender<AppEvent>,
// }
//
// impl<B: Backend> LocustPlugin<B> for EventSourcedIntegration {
//     fn id(&self) -> &'static str { "app.event_sourced" }
//
//     fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
//         // Convert Locust events to app events
//         if let Some(target_id) = ctx.get_data::<String>("nav_activated") {
//             if let Some(id_str) = target_id.strip_prefix("item_") {
//                 if let Ok(id) = id_str.parse::<usize>() {
//                     let _ = self.event_tx.send(AppEvent::ItemSelected { id });
//                 }
//             }
//             ctx.remove_data("nav_activated");
//         }
//
//         PluginEventResult::NotHandled
//     }
//
//     fn render_overlay(&self, _frame: &mut Frame, _ctx: &LocustContext) {}
// }

// ============================================================================
// Pattern 4: Component-based State
// ============================================================================

trait Component {
    fn update(&mut self, msg: ComponentMessage);
    fn view(&self) -> String;
}

#[derive(Clone, Debug)]
enum ComponentMessage {
    Input(String),
    TargetActivated(String),
    CommandExecuted(String),
}

struct TodoListComponent {
    items: Vec<TodoItem>,
    selected: usize,
}

impl Component for TodoListComponent {
    fn update(&mut self, msg: ComponentMessage) {
        match msg {
            ComponentMessage::TargetActivated(target_id) => {
                if let Some(index_str) = target_id.strip_prefix("todo_") {
                    if let Ok(index) = index_str.parse::<usize>() {
                        if index < self.items.len() {
                            self.selected = index;
                        }
                    }
                }
            }
            ComponentMessage::CommandExecuted(cmd_id) => match cmd_id.as_str() {
                "todo.toggle" => {
                    if self.selected < self.items.len() {
                        self.items[self.selected].completed = !self.items[self.selected].completed;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn view(&self) -> String {
        self.items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let marker = if i == self.selected { ">" } else { " " };
                let status = if item.completed { "✓" } else { " " };
                format!("{} [{}] {}", marker, status, item.text)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

struct ComponentBasedApp {
    components: HashMap<String, Box<dyn Component>>,
}

impl ComponentBasedApp {
    fn new() -> Self {
        let mut components: HashMap<String, Box<dyn Component>> = HashMap::new();

        components.insert(
            "todo_list".to_string(),
            Box::new(TodoListComponent {
                items: vec![
                    TodoItem {
                        text: "Component 1".to_string(),
                        completed: false,
                    },
                    TodoItem {
                        text: "Component 2".to_string(),
                        completed: false,
                    },
                ],
                selected: 0,
            }),
        );

        Self { components }
    }

    fn send_message(&mut self, component_id: &str, msg: ComponentMessage) {
        if let Some(component) = self.components.get_mut(component_id) {
            component.update(msg);
        }
    }

    fn view(&self, component_id: &str) -> Option<String> {
        self.components.get(component_id).map(|c| c.view())
    }
}

// Integration plugin for component-based
// struct ComponentIntegration {
//     message_router: HashMap<String, (String, ComponentMessage)>,
// }
//
// impl<B: Backend> LocustPlugin<B> for ComponentIntegration {
//     fn id(&self) -> &'static str { "app.component" }
//
//     fn on_event(&mut self, event: &Event, ctx: &mut LocustContext) -> PluginEventResult {
//         // Route Locust events to appropriate components
//         if let Some(target_id) = ctx.get_data::<String>("nav_activated") {
//             // Determine which component owns this target
//             let component_id = target_id.split('_').next().unwrap_or("");
//             ctx.store_data(
//                 "component_message",
//                 Box::new((
//                     component_id.to_string(),
//                     ComponentMessage::TargetActivated(target_id.clone()),
//                 )),
//             );
//             ctx.remove_data("nav_activated");
//         }
//
//         PluginEventResult::NotHandled
//     }
//
//     fn render_overlay(&self, _frame: &mut Frame, _ctx: &LocustContext) {}
// }

// ============================================================================
// Main Example
// ============================================================================

fn main() {
    println!("State Management Integration Patterns\n");

    println!("=== Pattern 1: Message Passing (Elm-like) ===");
    let mut msg_app = MessagePassingApp::new();
    let sender = msg_app.sender();

    sender.send(Message::SelectItem(1)).unwrap();
    sender.send(Message::ToggleItem(1)).unwrap();
    msg_app.update();
    println!("Items: {:?}", msg_app.state.items);
    println!("Selected: {}\n", msg_app.state.selected);

    println!("=== Pattern 2: Shared State (Arc<RwLock<T>>) ===");
    let shared_app = SharedStateApp::new();
    shared_app.select_item(1);
    shared_app.set_view_mode(ViewMode::Grid);
    {
        let state = shared_app.state.read().unwrap();
        println!("Selected: {}", state.selected);
        println!("View Mode: {:?}\n", state.view_mode);
    }

    println!("=== Pattern 3: Event Sourcing ===");
    let mut event_app = EventSourcedApp::new();
    event_app.apply_event(AppEvent::ItemToggled { id: 0 });
    event_app.apply_event(AppEvent::ItemSelected { id: 1 });
    println!("Events: {} total", event_app.events.len());
    println!("Current state: {:?}\n", event_app.state);

    println!("=== Pattern 4: Component-based ===");
    let mut comp_app = ComponentBasedApp::new();
    comp_app.send_message(
        "todo_list",
        ComponentMessage::TargetActivated("todo_1".to_string()),
    );
    if let Some(view) = comp_app.view("todo_list") {
        println!("Todo List View:\n{}\n", view);
    }

    println!("=== Integration Guidelines ===\n");
    println!("1. Message Passing:");
    println!("   - Best for: Redux-like architectures");
    println!("   - Pros: Clear data flow, easy testing");
    println!("   - Cons: Boilerplate, message overhead\n");

    println!("2. Shared State:");
    println!("   - Best for: Simple apps, React-like patterns");
    println!("   - Pros: Simple, direct access");
    println!("   - Cons: Lock contention, harder to debug\n");

    println!("3. Event Sourcing:");
    println!("   - Best for: Apps needing undo/replay");
    println!("   - Pros: Complete history, time travel debugging");
    println!("   - Cons: Memory usage, complexity\n");

    println!("4. Component-based:");
    println!("   - Best for: Complex UIs with many components");
    println!("   - Pros: Encapsulation, reusability");
    println!("   - Cons: Routing complexity, indirection\n");

    println!("Choose based on your app's needs and team familiarity!");
}
