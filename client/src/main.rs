mod temppipe;
use temppipe::TempPipe;
use std::fs;
use std::io;
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
        println!("Requested command: {}",args[2]);
        writeln!(&mut pipe,"{}{}{}", args[2].to_string(),";;",currentPid);
    }

    let outputPipeName: String = "oxy_pip_output_".to_string() + &currentPid;
    let mut outputPipe = Pipe::with_name(&outputPipeName).unwrap();
    for line in std::io::BufReader::new(outputPipe).lines(){
        let lineOutput: String = line.unwrap();
        match lineOutput == "Oxy-over" {
            true => {
                // Attempt to remove the named pipe
                    let outputPipePath: String = "/tmp/".to_string()+ &outputPipeName;
                    match fs::remove_file(&outputPipePath) {
                    Ok(_) => println!("Successfully removed the named pipe: {}", outputPipePath),
                    Err(e) => eprintln!("Failed to remove the named pipe: {}", outputPipePath),
                }
                break; }
            false => {
                println!(" ↲ {}", lineOutput);
            }
        }
    }
}
