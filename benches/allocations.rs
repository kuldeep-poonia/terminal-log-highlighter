use criterion::{criterion_group, criterion_main, Criterion};
use sentinel::runtime::events::LineEvent;
use sentinel::runtime::line_assembler::LineAssembler;
use std::io;

fn bench_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocations");
    // Use a large input that triggers the ring buffer growth path.
    let input = "a".repeat(200_000) + "\n"; // 200KB line
    let bytes = input.as_bytes();

    group.bench_function("large_line_growth", |b| {
        b.iter(|| {
            let mut assembler = LineAssembler::new();
            assembler
                .push(bytes, |_event: LineEvent<'_>| -> io::Result<()> { Ok(()) })
                .unwrap();
        });
    });
    group.finish();
}

criterion_group!(benches, bench_allocations);
criterion_main!(benches);
