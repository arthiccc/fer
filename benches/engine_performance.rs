use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use telco_core::{TelcoSimulator, QuotaType};
use std::time::Duration;

fn bench_engine(c: &mut Criterion) {
    let temp_db = "/tmp/bench.db";
    let _ = std::fs::remove_file(temp_db);
    let sim = TelcoSimulator::new("bench_user".to_string(), temp_db.to_string()).unwrap();
    
    // Warm up with a topping
    let _ = sim.handle_command("General 10GB".to_string());

    c.bench_function("Rust Core: Multi-Quota Deduction + SQL Persistence", |b| {
        b.iter(|| {
            sim.simulate_usage(black_box(1024 * 1024), black_box(QuotaType::General))
        })
    });

    c.bench_function("Legacy Bridge Simulator: Mock 10ms Lag", |b| {
        b.iter(|| {
            // Simulate the typical JSON serialization + JS context switch lag
            std::thread::sleep(Duration::from_millis(10));
            sim.simulate_usage(black_box(1024 * 1024), black_box(QuotaType::General))
        })
    });
}

criterion_group!(benches, bench_engine);
criterion_main!(benches);
