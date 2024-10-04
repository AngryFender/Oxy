use std::env;
use std::io::BufRead;
use ipipe::{pprint, Pipe};
use std::io::Write;
use std::process::Command;

fn main() {
    if env::consts::OS != "linux" {
        return;
    }
    println!("Starting, Oxy!");

    let args: Vec<String> = env::args().collect();
    if args.len() <3{
        return;
    }

    if args[1] == "run" {
        let mut pipe = Pipe::with_name("oxy_pipe").unwrap();
        writeln!(&mut pipe,"{}", args[2].to_string());

    }

    let mut outputPipe = Pipe::with_name("oxy_pipe_output").unwrap();
    for line in std::io::BufReader::new(outputPipe).lines(){
        println!("{}",line.unwrap());
    }
}
