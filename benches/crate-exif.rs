use criterion::{Criterion, criterion_group, criterion_main};

fn thumbnailer(c: &mut Criterion) {
    let mut group = c.benchmark_group("apple-iphone6.exif.le.tiff");

    group.bench_function("gufo", |b| {
        b.iter_batched_ref(
            || std::fs::read("../apple-iphone6.exif.le.tiff").unwrap(),
            |data| {
                let data = gufo_exif::ExifInternal::for_mut_slice(data).unwrap();
                data.gps_location().unwrap();
            },
            criterion::BatchSize::LargeInput,
        )
    });

    group.bench_function("kamadak-exif", |b| {
        b.iter_batched_ref(
            || std::fs::read("../apple-iphone6.exif.le.tiff").unwrap(),
            |data| {
                exif::parse_exif(data).unwrap();
            },
            criterion::BatchSize::LargeInput,
        )
    });
}

criterion_main!(benches);
criterion_group!(
    name = benches;
    config = Criterion::default().with_plots();
    targets = thumbnailer
);
