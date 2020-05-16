use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use kcpu::{
    cli::command,
    exec::{
        event_loop::headless, interactor::noninteractive, pipeline, poller, types::PipelineBuilder,
    },
};
use std::path::PathBuf;

fn suite_test_primes(c: &mut Criterion) {
    // RUSTFIX proper error handling, instead of just calling `unwrap()`.
    let prog_bin = command::assemble_path(&PathBuf::from("asm/test/primes.ks")).unwrap();

    c.bench_function("sample", |b| {
        b.iter_batched(
            || {
                pipeline::Run::new(None, None, noninteractive::Interactor)
                    .build()
                    .runner(poller::BlockingFactory, headless::EventLoop)
            },
            |runner| runner.run_with_binaries(None, Some(&prog_bin)).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

// RUSTFIX pre-compile and the run all of the units in the `test` suite (`bench` will take too long I think)

criterion_group!(benches, suite_test_primes);
criterion_main!(benches);
