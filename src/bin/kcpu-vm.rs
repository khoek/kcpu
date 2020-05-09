use kcpu::frontend::command;
use structopt::StructOpt;

fn main() {
    command::terminal_init();
    command::vm(command::SubcommandVm::from_args());
}
