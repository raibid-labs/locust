//! Benchmarks for fuzzy matching performance.
//!
//! These benchmarks ensure the fuzzy matcher meets performance requirements:
//! - <1ms for 1000 candidates
//! - Efficient scoring algorithm
//! - Minimal allocations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use locust::core::fuzzy::FuzzyMatcher;

fn bench_exact_match(c: &mut Criterion) {
    let matcher = FuzzyMatcher::new();

    c.bench_function("exact_match_short", |b| {
        b.iter(|| {
            black_box(matcher.score(black_box("test"), black_box("test")));
        });
    });

    c.bench_function("exact_match_long", |b| {
        let long_text = "this_is_a_very_long_file_name_for_testing_purposes";
        b.iter(|| {
            black_box(matcher.score(black_box(long_text), black_box(long_text)));
        });
    });
}

fn bench_fuzzy_match(c: &mut Criterion) {
    let matcher = FuzzyMatcher::new();

    c.bench_function("fuzzy_match_short", |b| {
        b.iter(|| {
            black_box(matcher.score(black_box("fm"), black_box("file_manager")));
        });
    });

    c.bench_function("fuzzy_match_long", |b| {
        let query = "sctrl";
        let text = "system_controller_implementation";
        b.iter(|| {
            black_box(matcher.score(black_box(query), black_box(text)));
        });
    });

    c.bench_function("fuzzy_match_no_match", |b| {
        b.iter(|| {
            black_box(matcher.score(black_box("xyz"), black_box("abcdefghijk")));
        });
    });
}

fn bench_find_matches_scaling(c: &mut Criterion) {
    let matcher = FuzzyMatcher::new();
    let mut group = c.benchmark_group("find_matches_scaling");

    for size in [10, 50, 100, 500, 1000].iter() {
        let candidates: Vec<String> = (0..*size)
            .map(|i| format!("candidate_{:04}_item", i))
            .collect();
        let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(matcher.find_matches(black_box("cand"), black_box(&candidate_refs)));
            });
        });
    }

    group.finish();
}

fn bench_find_matches_query_length(c: &mut Criterion) {
    let matcher = FuzzyMatcher::new();
    let mut group = c.benchmark_group("find_matches_query_length");

    let candidates: Vec<String> = (0..100)
        .map(|i| format!("some_long_candidate_name_{}", i))
        .collect();
    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();

    for query_len in [1, 2, 4, 8, 16].iter() {
        let query: String = "x".repeat(*query_len);

        group.bench_with_input(BenchmarkId::from_parameter(query_len), query_len, |b, _| {
            b.iter(|| {
                black_box(matcher.find_matches(black_box(&query), black_box(&candidate_refs)));
            });
        });
    }

    group.finish();
}

fn bench_realistic_scenarios(c: &mut Criterion) {
    let matcher = FuzzyMatcher::new();

    // Simulate file path matching
    c.bench_function("realistic_file_paths", |b| {
        let paths = vec![
            "src/core/fuzzy.rs",
            "src/core/mod.rs",
            "src/core/context.rs",
            "src/core/input.rs",
            "src/plugins/nav/mod.rs",
            "src/plugins/omnibar/state.rs",
            "tests/unit/fuzzy_matcher.rs",
            "tests/integration/navigation_flow.rs",
            "benches/fuzzy_matching.rs",
            "examples/omnibar_demo.rs",
        ];

        b.iter(|| {
            black_box(matcher.find_matches(black_box("fuzz"), black_box(&paths)));
        });
    });

    // Simulate command matching
    c.bench_function("realistic_commands", |b| {
        let commands = vec![
            "open_file",
            "save_file",
            "save_all",
            "close_window",
            "close_all",
            "find_replace",
            "find_in_files",
            "goto_definition",
            "goto_line",
            "toggle_comment",
            "format_document",
            "rename_symbol",
            "show_references",
        ];

        b.iter(|| {
            black_box(matcher.find_matches(black_box("sf"), black_box(&commands)));
        });
    });
}

fn bench_unicode_handling(c: &mut Criterion) {
    let matcher = FuzzyMatcher::new();

    c.bench_function("unicode_ascii", |b| {
        b.iter(|| {
            black_box(matcher.score(black_box("test"), black_box("test")));
        });
    });

    c.bench_function("unicode_emojis", |b| {
        b.iter(|| {
            black_box(matcher.score(black_box("🦀"), black_box("🦀🚀💻")));
        });
    });

    c.bench_function("unicode_mixed", |b| {
        b.iter(|| {
            black_box(matcher.score(black_box("h🦀"), black_box("hello🦀world")));
        });
    });

    c.bench_function("unicode_cjk", |b| {
        b.iter(|| {
            black_box(matcher.score(black_box("日本"), black_box("日本語テスト")));
        });
    });
}

fn bench_worst_case_scenarios(c: &mut Criterion) {
    let matcher = FuzzyMatcher::new();

    // Long text with match at the end
    c.bench_function("worst_case_long_text", |b| {
        let long_text = "a".repeat(1000) + "xyz";
        b.iter(|| {
            black_box(matcher.score(black_box("xyz"), black_box(&long_text)));
        });
    });

    // Many candidates with no matches
    c.bench_function("worst_case_no_matches", |b| {
        let candidates: Vec<String> = (0..1000).map(|i| format!("item_{}", i)).collect();
        let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();

        b.iter(|| {
            black_box(matcher.find_matches(black_box("xyz"), black_box(&candidate_refs)));
        });
    });

    // Repeated pattern matching
    c.bench_function("worst_case_repeated_chars", |b| {
        let text = "aaaaaaaaaa";
        b.iter(|| {
            black_box(matcher.score(black_box("aaa"), black_box(text)));
        });
    });
}

fn bench_custom_configuration(c: &mut Criterion) {
    let default_matcher = FuzzyMatcher::new();
    let custom_matcher = FuzzyMatcher::new()
        .with_consecutive_bonus(20.0)
        .with_word_boundary_bonus(10.0)
        .with_first_char_bonus(15.0);

    c.bench_function("config_default", |b| {
        b.iter(|| {
            black_box(default_matcher.score(black_box("test"), black_box("test")));
        });
    });

    c.bench_function("config_custom", |b| {
        b.iter(|| {
            black_box(custom_matcher.score(black_box("test"), black_box("test")));
        });
    });
}

criterion_group!(
    benches,
    bench_exact_match,
    bench_fuzzy_match,
    bench_find_matches_scaling,
    bench_find_matches_query_length,
    bench_realistic_scenarios,
    bench_unicode_handling,
    bench_worst_case_scenarios,
    bench_custom_configuration,
);

criterion_main!(benches);
