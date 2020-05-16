use kcpu::{
    assets,
    cli::{command, suite},
};

#[test]
fn run_suite_test() -> Result<(), kcpu::assembler::Error> {
    assert!(suite::run_suite(
        &std::ffi::OsString::from("test"),
        &assets::default_suite_dir(),
        None,
        command::ClockLimit::default().into_option()
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
        command::ClockLimit::default().into_option()
    )?);
    Ok(())
}
