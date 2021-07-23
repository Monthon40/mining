use criterion::*;
#[path = "../src/mining.rs"]
mod mining;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("onetime_rank", |b| {
        b.iter(|| {
            mining::ontime_rank(
                // "/Desktop/Functional and Parallel Programing/mining",
                "C:/Users/Admin/Desktop/mining/2008.csv",
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);