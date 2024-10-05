use std::env;
use std::io::BufRead;
use ipipe::{pprint, Pipe};
use std::io::Write;
use std::process::Command;
use sysinfo::get_current_pid;

fn main() {
    if env::consts::OS != "linux" {
        return;
    }
    println!("Starting, Oxy!");

    let args: Vec<String> = env::args().collect();
    if args.len() <3{
        return;
    }

    let mut currentPid = String::new();
    match get_current_pid() {
        Ok(pid) => {
            currentPid = format!("{}",pid);
            println!("current pid: {}", currentPid);
        }
        Err(e) => {
            println!("failed to get current pid: {}", e);
        }
    }

    if args[1] == "run" {
        let mut pipe = Pipe::with_name("oxy_pipe").unwrap();
        writeln!(&mut pipe,"{} {}", args[2].to_string(), currentPid);
    }

//    let outputPipeName: String = "oxy_pip_output_" + currentPid;
    let mut outputPipe = Pipe::with_name("oxy_output_pipe").unwrap();
    for line in std::io::BufReader::new(outputPipe).lines(){
        let lineOutput: String = line.unwrap();
        println!("{}",lineOutput);
        match lineOutput == "Oxy-over" {
            true => { break; }
            false => {}
        }
    }
}
