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
    if args.len() <2{
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

    let mut pipe:Option<Pipe> = None;
    match args[1].as_str()
    {
        "run" => {
            pipe = Pipe::with_name("oxy_pipe").ok();
            if let Some(ref mut p) = pipe {
                writeln!(p, "{}{}{}", args[2], ";;", currentPid).unwrap();
                println!("{}{}{}", args[2], ";;", currentPid);
            }
        },
        "status" => {
            pipe = Pipe::with_name("oxy_instruction_pipe").ok();
            if let Some(ref mut p) = pipe {
                println!("{}{}{}", "status", ";;", currentPid);
                writeln!(p, "{}{}{}", "status", ";;", currentPid).unwrap();
            }
        },
        _ =>{
            println!("unknown command");
        }
    }

    let outputPipeName: String = "oxy_pip_output_".to_string() + &currentPid;
    let mut outputPipe = TempPipe::new(&outputPipeName);
    for line in std::io::BufReader::new(outputPipe.get_pipe()).lines(){
        let lineOutput: String = line.unwrap();
        match lineOutput == "Oxy-over" {
            true => {
                break; }
            false => {
                println!(" ↲ {}", lineOutput);
            }
        }
    }
}
