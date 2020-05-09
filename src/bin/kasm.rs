use kcpu::frontend::command;
use structopt::StructOpt;

fn main() {
    command::terminal_init();
    command::asm(command::SubcommandAsm::from_args());
}
