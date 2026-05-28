use criterion::{criterion_group, criterion_main, Criterion};
use sentinel::runtime::events::LineEvent;
use sentinel::runtime::line_assembler::LineAssembler;
use std::io;

fn bench_single_line_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency");
    let line = b"2024-01-01 ERROR something went wrong\n";

    group.bench_function("single_line_push", |b| {
        b.iter(|| {
            let mut assembler = LineAssembler::new();
            assembler
                .push(line, |_event: LineEvent<'_>| -> io::Result<()> { Ok(()) })
                .unwrap();
        });
    });
    group.finish();
}

criterion_group!(benches, bench_single_line_latency);
criterion_main!(benches);
