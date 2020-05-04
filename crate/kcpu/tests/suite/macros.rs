use kcpu::vm::State;
use kcpu::assembler;
use kcpu::frontend::{assemble, execute::{self, BreakMode, Config, Verbosity, AbortAction}};

pub fn run_test(bios_src: Option<&str>, prog_src: &str) -> Result<(), assembler::Error> {
    let bios = bios_src
        .as_deref()
        .map(assemble::assemble)
        .transpose()?;
    let prog = assemble::assemble(&prog_src)?;

    let summary = execute::execute(
        Config {
            headless: true,
            max_clocks: Some(5_000_000),

            verbosity: Verbosity::Silent,
            mode: BreakMode::Noninteractive,
            abort_action: AbortAction::Stop,

            print_marginals: true,
        },
        bios.as_ref(),
        Some(&prog),
    );

    assert_eq!(summary.state, State::Halted);
    Ok(())
}

// RUSTFIX TODO DEFINE THE MACROS WE WANT HERE
