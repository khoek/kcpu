use kcpu::{
    assets,
    cli::suite,
};

#[test]
fn run_suite_test() -> Result<(), kcpu::assembler::Error> {
    assert!(suite::run_suite(
        &std::ffi::OsString::from("test"),
        &assets::default_suite_dir(),
        None,
        Some(50_000_000)
    )?);
    Ok(())
}

#[test]
#[cfg_attr(not(feature = "big_tests"), ignore)]
fn run_suite_bench() -> Result<(), kcpu::assembler::Error> {
    assert!(suite::run_suite(
        &std::ffi::OsString::from("bench"),
        &assets::default_suite_dir(),
        None,
        Some(50_000_000)
    )?);
    Ok(())
}
