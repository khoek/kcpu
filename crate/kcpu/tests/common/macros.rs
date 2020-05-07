use kcpu::assembler;
use kcpu::frontend::{
    assemble,
    execute::{self, AbortAction, BreakMode, Config, ExecFlags, Verbosity},
};
use kcpu::vm::State;

pub fn run_test(bios_src: Option<&str>, prog_src: &str) -> Result<(), assembler::Error> {
    let bios = bios_src.as_deref().map(assemble::assemble).transpose()?;
    let prog = assemble::assemble(&prog_src)?;

    let summary = execute::execute(
        Config {
            flags: ExecFlags {
                headless: true,
                max_clocks: Some(5_000_000),
                mode: BreakMode::Noninteractive,
                abort_action: AbortAction::Stop,
            },

            verbosity: Verbosity::Silent,
            print_marginals: true,
        },
        bios.as_deref(),
        Some(&prog),
    )
    .unwrap();

    assert_eq!(summary.state, State::Halted);
    Ok(())
}

// RUSTFIX TODO DEFINE THE MACROS WE WANT HERE
