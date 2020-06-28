use criterion::{black_box, criterion_group, criterion_main, Benchmark, Criterion, Throughput};
use hessian_rs::from_slice;

static INPUT: &'static [u8] = include_bytes!("../tests/fixtures/map/custom_map_type.bin");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench(
        "decode",
        Benchmark::new("from_slice", |b| {
            b.iter(|| {
                from_slice(black_box(INPUT)).unwrap();
            })
        })
        .throughput(Throughput::Bytes(INPUT.len() as u64)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
