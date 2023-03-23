use criterion::{
    black_box, criterion_group, criterion_main, Benchmark, BenchmarkGroup, Criterion, Throughput,
};
use hessian_rs::{from_slice, to_vec};

static INPUT: &[u8] = include_bytes!("../tests/fixtures/map/custom_map_type.bin");

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");
    group.throughput(Throughput::Bytes(INPUT.len() as u64));
    group.bench_function("from_slice", |b| {
        b.iter(|| {
            from_slice(black_box(INPUT)).unwrap();
        })
    });
    group.bench_function("from_slice_to_vec", |b| {
        b.iter(|| {
            to_vec(&from_slice(black_box(INPUT)).unwrap()).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
