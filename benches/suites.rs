use criterion::{criterion_group, criterion_main, Criterion};
use kcpu::cli::run::execute::{self, AbortAction, Config, Verbosity};
use std::path::PathBuf;

fn suite_test_primes(c: &mut Criterion) {
    // RUSTFIX proper error handling, instead of just calling `unwrap()`.
    let prog_bin = assemble::assemble_path(&PathBuf::from("asm/test/primes.ks")).unwrap();

    c.bench_function("sample", |b| {
        b.iter(|| {
            execute::execute(
                Config {
                    headless: true,
                    max_clocks: None,
                    abort_action: AbortAction::Stop,

                    verbosity: Verbosity::Silent,
                    print_marginals: false,
                },
                None,
                Some(&prog_bin),
            )
        })
    });
}

// RUSTFIX pre-compile and the run all of the units in the `test` suite (`bench` will take too long I think)

criterion_group!(benches, suite_test_primes);
criterion_main!(benches);
