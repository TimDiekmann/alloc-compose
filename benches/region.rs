#![feature(allocator_api)]

use alloc_compose::{region::*, AllocateAll};
use core::{
    alloc::{AllocRef, Layout},
    mem::MaybeUninit,
};

use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};

fn regions(c: &mut Criterion) {
    let mut group = c.benchmark_group("region");
    let mut data = [MaybeUninit::uninit(); 1024 * 1024];

    #[inline]
    fn run(region: impl AllocRef + AllocateAll, b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..16 {
                region.alloc(black_box(Layout::new::<[u8; 16]>())).unwrap();
            }
            region.deallocate_all();
        })
    }

    group.bench_function("Region", |b| run(Region::new(&mut data), b));
    group.bench_function("SharedRegion", |b| run(SharedRegion::new(&mut data), b));
    group.bench_function("IntrusiveRegion", |b| {
        run(IntrusiveRegion::new(&mut data), b)
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(1000).measurement_time(std::time::Duration::from_secs(3));
    targets = regions
}
criterion_main!(benches);
