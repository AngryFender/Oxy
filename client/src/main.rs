mod temppipe;
use temppipe::TempPipe;
use std::env;
use std::io::BufRead;
use ipipe::{Pipe};
use std::io::Write;
use sysinfo::get_current_pid;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version,
    about = "A cli tool to run bash commands synchronously",
    long_about = None,
)]
struct Cli {
    #[command[subcommand]]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    run {command: String},
    status {},
    current{},
    kill{},
}

fn main() {
    if env::consts::OS != "linux" {
        return;
    }

    let mut pipe:Option<Pipe> = None;
    let mut current_pid = String::new();
    match get_current_pid() {
        Ok(pid) => {
            current_pid = format!("{}", pid);
        }
        Err(e) => {
            println!("Failed to get current pid: {}", e);
            return;
        }
    }

    let cli = Cli::parse();
    match cli.command {
        Commands::run{command} => {
            pipe = Pipe::with_name("oxy_pipe").ok();
            if let Some(ref mut p) = pipe {
                writeln!(p, "{}{}{}", command, ";;", current_pid).unwrap();
            }
        }
        Commands::status{} => {
            pipe = Pipe::with_name("oxy_instruction_pipe").ok();
            if let Some(ref mut p) = pipe {
                writeln!(p, "{}{}{}", "status", ";;", current_pid).unwrap();
            }
        }
        Commands::current{} => {
            pipe = Pipe::with_name("oxy_instruction_pipe").ok();
            if let Some(ref mut p) = pipe {
                writeln!(p, "{}{}{}", "current", ";;", current_pid).unwrap();
            }
        }
        Commands::kill{} => {
            pipe = Pipe::with_name("oxy_instruction_pipe").ok();
            if let Some(ref mut p) = pipe {
                println!("{}{}{}", "kill", ";;", current_pid);
            }
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
