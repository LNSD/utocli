//! Simple test to verify clap derive detection works

use clap::Parser;
use utocli::clap::OpenCli;

#[derive(Parser, OpenCli)]
#[command(name = "test")]
struct TestCli {
    #[arg(short)]
    verbose: bool,
}

#[test]
fn can_call_opencli_method() {
    let _commands = TestCli::opencli();
}
