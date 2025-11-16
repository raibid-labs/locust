//! Performance benchmarks for plugin operations
//!
//! Benchmarks plugin event handling, overlay rendering, and other critical paths.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use locust::core::context::LocustContext;
use locust::core::plugin::LocustPlugin;
use locust::core::targets::NavTarget;
use locust::plugins::highlight::{HighlightPlugin, Tour, TourStep};
use locust::plugins::nav::NavPlugin;
use locust::plugins::omnibar::OmnibarPlugin;
use locust::plugins::tooltip::{TooltipContent, TooltipPlugin};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

fn benchmark_plugin_event_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_events");

    // Benchmark single plugin event handling
    group.bench_function("nav_plugin_single_event", |b| {
        let mut ctx = LocustContext::default();
        let mut plugin = NavPlugin::new();
        LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));

        b.iter(|| LocustPlugin::<TestBackend>::on_event(&mut plugin, black_box(&event), &mut ctx));
    });

    // Benchmark multiple plugins
    for count in [2, 4, 8].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut ctx = LocustContext::default();
            let mut plugins: Vec<Box<dyn LocustPlugin<TestBackend>>> = Vec::new();

            for _ in 0..count {
                plugins.push(Box::new(NavPlugin::new()));
            }

            for plugin in &mut plugins {
                plugin.init(&mut ctx);
            }

            let event = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));

            b.iter(|| {
                for plugin in &mut plugins {
                    plugin.on_event(black_box(&event), &mut ctx);
                }
            });
        });
    }

    group.finish();
}

fn benchmark_overlay_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("overlay_rendering");

    group.bench_function("nav_overlay_render", |b| {
        let mut ctx = LocustContext::default();
        let plugin = NavPlugin::new();

        // Register targets
        for i in 0..50 {
            ctx.targets
                .register(NavTarget::new(i, Rect::new(i * 5, 10, 10, 2)));
        }

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        b.iter(|| {
            terminal
                .draw(|frame| {
                    plugin.render_overlay(black_box(frame), &ctx);
                })
                .unwrap();
        });
    });

    group.bench_function("tooltip_overlay_render", |b| {
        let mut ctx = LocustContext::default();
        let mut plugin = TooltipPlugin::new();

        ctx.targets
            .register(NavTarget::new(1, Rect::new(10, 10, 20, 3)));
        ctx.tooltips
            .register(1, TooltipContent::new("Test tooltip"));

        LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        b.iter(|| {
            terminal
                .draw(|frame| {
                    plugin.render_overlay(black_box(frame), &ctx);
                })
                .unwrap();
        });
    });

    group.finish();
}

fn benchmark_target_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("target_operations");

    group.bench_function("register_100_targets", |b| {
        b.iter(|| {
            let mut ctx = LocustContext::default();
            for i in 0..100 {
                ctx.targets
                    .register(NavTarget::new(i, Rect::new(i * 2, i * 2, 10, 2)));
            }
        });
    });

    group.bench_function("lookup_target_by_id", |b| {
        let mut ctx = LocustContext::default();
        for i in 0..1000 {
            ctx.targets
                .register(NavTarget::new(i, Rect::new(i * 2, i * 2, 10, 2)));
        }

        b.iter(|| {
            ctx.targets.by_id(black_box(500));
        });
    });

    group.bench_function("nearest_target_search", |b| {
        let mut ctx = LocustContext::default();
        for i in 0..100 {
            ctx.targets
                .register(NavTarget::new(i, Rect::new(i * 5, i * 3, 10, 2)));
        }

        b.iter(|| {
            ctx.targets.nearest_to(black_box(250), black_box(150));
        });
    });

    group.finish();
}

fn benchmark_tooltip_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("tooltip_operations");

    group.bench_function("register_100_tooltips", |b| {
        b.iter(|| {
            let mut ctx = LocustContext::default();
            for i in 0..100 {
                ctx.tooltips
                    .register(i, TooltipContent::new(&format!("Tooltip {}", i)));
            }
        });
    });

    group.bench_function("tooltip_lookup", |b| {
        let mut ctx = LocustContext::default();
        for i in 0..1000 {
            ctx.tooltips
                .register(i, TooltipContent::new(&format!("Tooltip {}", i)));
        }

        b.iter(|| {
            ctx.tooltips.get(black_box(500));
        });
    });

    group.finish();
}

fn benchmark_context_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_operations");

    group.bench_function("context_creation", |b| {
        b.iter(|| {
            let _ctx = LocustContext::default();
        });
    });

    group.bench_function("context_with_data", |b| {
        b.iter(|| {
            let mut ctx = LocustContext::default();
            for i in 0..10 {
                ctx.targets
                    .register(NavTarget::new(i, Rect::new(i * 10, 10, 10, 2)));
                ctx.tooltips
                    .register(i, TooltipContent::new(&format!("Tip {}", i)));
            }
        });
    });

    group.finish();
}

fn benchmark_tour_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("tour_operations");

    group.bench_function("create_tour_10_steps", |b| {
        b.iter(|| {
            let mut tour = Tour::new("benchmark");
            for i in 0..10 {
                tour = tour.add_step(TourStep::new(&format!("Step {}", i), "Description"));
            }
        });
    });

    group.bench_function("navigate_tour", |b| {
        let mut tour = Tour::new("benchmark");
        for i in 0..10 {
            tour = tour.add_step(TourStep::new(&format!("Step {}", i), "Description"));
        }
        tour.start();

        b.iter(|| {
            let mut t = tour.clone();
            while t.next_step() {
                // Navigate through tour
            }
        });
    });

    group.finish();
}

fn benchmark_plugin_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_init");

    group.bench_function("init_nav_plugin", |b| {
        b.iter(|| {
            let mut ctx = LocustContext::default();
            let mut plugin = NavPlugin::new();
            LocustPlugin::<TestBackend>::init(&mut plugin, &mut ctx);
        });
    });

    group.bench_function("init_all_plugins", |b| {
        b.iter(|| {
            let mut ctx = LocustContext::default();
            let mut nav = NavPlugin::new();
            let mut omnibar = OmnibarPlugin::new();
            let mut tooltip = TooltipPlugin::new();
            let mut highlight = HighlightPlugin::new();

            LocustPlugin::<TestBackend>::init(&mut nav, &mut ctx);
            LocustPlugin::<TestBackend>::init(&mut omnibar, &mut ctx);
            LocustPlugin::<TestBackend>::init(&mut tooltip, &mut ctx);
            LocustPlugin::<TestBackend>::init(&mut highlight, &mut ctx);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_plugin_event_handling,
    benchmark_overlay_rendering,
    benchmark_target_operations,
    benchmark_tooltip_operations,
    benchmark_context_operations,
    benchmark_tour_operations,
    benchmark_plugin_initialization,
);
criterion_main!(benches);
