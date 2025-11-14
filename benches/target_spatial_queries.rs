use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use locust::core::targets::{NavTarget, TargetPriority, TargetRegistry};
use ratatui::layout::Rect;

fn create_registry_with_targets(count: usize) -> TargetRegistry {
    let mut registry = TargetRegistry::new();
    for i in 0..count {
        registry.register(
            NavTarget::new(
                i as u64,
                Rect::new((i % 50) as u16 * 10, (i / 50) as u16 * 2, 8, 1),
            )
            .with_label(format!("Target {}", i))
            .with_priority(match i % 4 {
                0 => TargetPriority::Critical,
                1 => TargetPriority::High,
                2 => TargetPriority::Normal,
                _ => TargetPriority::Low,
            }),
        );
    }
    registry
}

fn bench_target_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("target_registration");

    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut registry = TargetRegistry::new();
                for i in 0..size {
                    registry.register(
                        NavTarget::new(
                            i as u64,
                            Rect::new((i % 50) as u16 * 10, (i / 50) as u16 * 2, 8, 1),
                        )
                        .with_label(format!("Target {}", i)),
                    );
                }
                black_box(registry);
            });
        });
    }

    group.finish();
}

fn bench_spatial_at_point(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_at_point");

    for size in [100, 500, 1000].iter() {
        let registry = create_registry_with_targets(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let targets = registry.at_point(black_box(100), black_box(20));
                black_box(targets);
            });
        });
    }

    group.finish();
}

fn bench_spatial_in_area(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_in_area");

    for size in [100, 500, 1000].iter() {
        let registry = create_registry_with_targets(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let area = Rect::new(0, 0, 200, 40);
                let targets = registry.in_area(black_box(area));
                black_box(targets);
            });
        });
    }

    group.finish();
}

fn bench_by_id_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("by_id_lookup");

    for size in [100, 500, 1000].iter() {
        let registry = create_registry_with_targets(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let target = registry.by_id(black_box((size / 2) as u64));
                black_box(target);
            });
        });
    }

    group.finish();
}

fn bench_closest_to(c: &mut Criterion) {
    let mut group = c.benchmark_group("closest_to");

    for size in [100, 500, 1000].iter() {
        let registry = create_registry_with_targets(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let closest = registry.closest_to(black_box(250), black_box(50));
                black_box(closest);
            });
        });
    }

    group.finish();
}

fn bench_filter_by_priority(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_by_priority");

    for size in [100, 500, 1000].iter() {
        let registry = create_registry_with_targets(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let targets = registry.by_priority(black_box(TargetPriority::High));
                black_box(targets);
            });
        });
    }

    group.finish();
}

fn bench_sorted_by_priority(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorted_by_priority");

    for size in [100, 500, 1000].iter() {
        let registry = create_registry_with_targets(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let sorted = registry.sorted_by_priority();
                black_box(sorted);
            });
        });
    }

    group.finish();
}

fn bench_sorted_by_area(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorted_by_area");

    for size in [100, 500, 1000].iter() {
        let registry = create_registry_with_targets(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let sorted = registry.sorted_by_area();
                black_box(sorted);
            });
        });
    }

    group.finish();
}

fn bench_clear(c: &mut Criterion) {
    c.bench_function("clear_1000_targets", |b| {
        b.iter_with_setup(
            || create_registry_with_targets(1000),
            |mut registry| {
                registry.clear();
                black_box(registry);
            },
        );
    });
}

criterion_group!(
    benches,
    bench_target_registration,
    bench_spatial_at_point,
    bench_spatial_in_area,
    bench_by_id_lookup,
    bench_closest_to,
    bench_filter_by_priority,
    bench_sorted_by_priority,
    bench_sorted_by_area,
    bench_clear
);

criterion_main!(benches);
