use clap::{Parser, Subcommand};
use std::fs; 

#[derive(Debug, Subcommand)]
enum Command{
    Init
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command
}

fn initialize(){
    fs::create_dir(".git").expect("SHOULD NOT FAIL"); 
    fs::create_dir(".git/objects").expect("SHOULD NOT FAIL"); 
    fs::create_dir(".git/refs").expect("SHOULD NOT FAIL");
    fs::write("git/HEAD", "ref: refs/head/main\n").unwrap(); 
    println!("Initialized git directory"); 
}

fn main() {
    let args = Args::parse();

    match args.command{
        Command::Init =>  initialize(),
    }
}