mod temppipe;
use temppipe::TempPipe;
use std::env;
use std::io::BufRead;
use ipipe::{Pipe};
use std::io::Write;
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

    let mut current_pid = String::new();
    match get_current_pid() {
        Ok(pid) => {
            current_pid = format!("{}", pid);
            println!("current pid: {}", current_pid);
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
                writeln!(p, "{}{}{}", args[2], ";;", current_pid).unwrap();
                println!("{}{}{}", args[2], ";;", current_pid);
            }
        },
        "status" => {
            pipe = Pipe::with_name("oxy_instruction_pipe").ok();
            if let Some(ref mut p) = pipe {
                println!("{}{}{}", "status", ";;", current_pid);
                writeln!(p, "{}{}{}", "status", ";;", current_pid).unwrap();
            }
        },
        "current" => {
            pipe = Pipe::with_name("oxy_instruction_pipe").ok();
            if let Some(ref mut p) = pipe {
                println!("{}{}{}", "current", ";;", current_pid);
                writeln!(p, "{}{}{}", "current", ";;", current_pid).unwrap();
            }
        },        _ =>{
            println!("unknown command");
        }
    }

    let output_pipe_name: String = "oxy_pip_output_".to_string() + &current_pid;
    let mut output_pipe = TempPipe::new(&output_pipe_name);
    for line in std::io::BufReader::new(output_pipe.get_pipe()).lines(){
        let line_output: String = line.unwrap();
        match line_output == "Oxy-over" {
            true => {
                break; }
            false => {
                println!(" ↳ {}", line_output);
            }
        }
    }
}
