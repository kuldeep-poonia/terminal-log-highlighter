use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use sentinel::runtime::events::LineEvent;
use sentinel::runtime::line_assembler::LineAssembler;
use std::io;

fn bench_assembler_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_assembler");
    // Simulate a typical stream: many short lines.
    let input = "hello world\n".repeat(10_000);
    let bytes = input.as_bytes();

    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("push_10k_lines", |b| {
        b.iter(|| {
            let mut assembler = LineAssembler::new();
            assembler
                .push(bytes, |_event: LineEvent<'_>| -> io::Result<()> { Ok(()) })
                .unwrap();
        });
    });
    group.finish();
}

criterion_group!(benches, bench_assembler_throughput);
criterion_main!(benches);
