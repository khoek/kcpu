use kcpu::frontend::command;
use structopt::StructOpt;

fn main() {
    command::root(command::CommandRoot::from_args());
}
