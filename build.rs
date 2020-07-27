extern crate lalrpop;

use std::process::Command;
use std::io::Stderr;

fn main() {
    lalrpop::Configuration::new()
        .generate_in_source_tree()
        .process()
        .unwrap();
    println!("cargo:rerun-if-changed=src/classwrite.d");
    Command::new("dmd")
        .spawn();
}