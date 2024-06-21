use bitmap::Bitmap;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_and(c: &mut Criterion) {
    let left = Bitmap::from_iter(&[0, 1, 2, 6, 9, 10, 25, 90, 91, 150, 2000]);
    let right = Bitmap::from_iter(&[0, 1, 3, 4, 9, 10, 29, 90, 91, 150, 3000]);
    let mut group = c.benchmark_group("small");
    group.bench_with_input("naive", &left, |b, left: &Bitmap| {
        b.iter(|| {
            let mut left = left.clone();
            left.intersection(black_box(&right));
            black_box(left)
        });
    });
    group.bench_with_input("simd", &left, |b, left: &Bitmap| {
        b.iter(|| {
            let mut left = left.clone();
            left.intersection_simd(black_box(&right));
            black_box(left)
        });
    });
    group.finish();

    let left = Bitmap::from_iter((0..200).chain(1000..2000).step_by(3).step_by(5));
    let right = Bitmap::from_iter((100..300).chain(1000..2000).step_by(5).step_by(3));
    let mut group = c.benchmark_group("medium");
    group.bench_with_input("naive", &left, |b, left: &Bitmap| {
        b.iter(|| {
            let mut left = left.clone();
            left.intersection(black_box(&right));
            black_box(left)
        });
    });
    group.bench_with_input("simd", &left, |b, left: &Bitmap| {
        b.iter(|| {
            let mut left = left.clone();
            left.intersection_simd(black_box(&right));
            black_box(left)
        });
    });
    group.finish();

    let left = Bitmap::from_iter(
        (0..20_000)
            .step_by(2)
            .step_by(3)
            .chain(50_000..60_000)
            .step_by(3)
            .step_by(5),
    );
    let right = Bitmap::from_iter(
        (0..20_000)
            .step_by(3)
            .step_by(2)
            .chain(50_000..60_000)
            .step_by(5)
            .step_by(3),
    );
    let mut group = c.benchmark_group("large");
    group.bench_with_input("naive", &left, |b, left: &Bitmap| {
        b.iter(|| {
            let mut left = left.clone();
            left.intersection(black_box(&right));
            black_box(left)
        });
    });
    group.bench_with_input("simd", &left, |b, left: &Bitmap| {
        b.iter(|| {
            let mut left = left.clone();
            left.intersection_simd(black_box(&right));
            black_box(left)
        });
    });
    group.finish();
}

criterion_group!(benches, bench_and);
criterion_main!(benches);
