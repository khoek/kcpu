use kcpu::cli::command;
use structopt::StructOpt;

fn main() {
    command::terminal_init();
    command::root(command::CommandRoot::from_args());
}
