mod temppipe;

use std::collections::VecDeque;
use temppipe::TempPipe;
use ipipe::{Pipe};
use std::io::{BufRead, BufReader};
use std::thread;
use std::io::Write;
use std::process::{Command, Stdio};
use crossbeam_channel::{unbounded, Receiver};
use std::sync::{mpsc, Arc, Mutex};

// ANSI escape codes for colors
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

fn main()  {
    println!("Starting oxyd service...");

    let (command_tx,command_rx) = unbounded();
    let (instruction_tx,instruction_rx) = mpsc::channel();
    let command_list = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let current_command_output = Arc::new(Mutex::new(VecDeque::<String>::new()));

    let thread_instruction_producer = thread::spawn( move  || {
        let mut instruction_pipe = TempPipe::new("oxy_instruction_pipe");
        println!("Listening instructions on: {}", instruction_pipe.get_path().display());
        for line in std::io::BufReader::new(instruction_pipe.get_pipe()).lines(){
            let instruction = String::from(line.unwrap());
            instruction_tx.send(instruction.clone()).unwrap();
        }
    });

    let command_list_consume = Arc::clone(&command_list);
    let thread_instruct = thread::spawn(move ||{
       for instruction in instruction_rx {
           let args_collection: Vec<&str> = instruction.split(";;").collect();

           if args_collection.len()!=2 {
               continue;
           }

           println!("Requested instruction : {}", args_collection[0]);

           if args_collection[0] == "status"{
                let output_pipe_name: String = "oxy_pip_output_".to_string() + &args_collection[1];
                let mut output_pipe = Pipe::with_name(&output_pipe_name).unwrap();

                let command_list = command_list_consume.lock().unwrap();
                writeln!(&mut output_pipe, "\nTotal commands: {}", command_list.len()).unwrap();
                writeln!(&mut output_pipe, "==========================================").unwrap();

                let mut count = 0;
                for command in command_list.iter(){
                    let args_collection: Vec<&str> = command.split(";;").collect();

                    if args_collection.len()!=2 {
                        continue;
                    }

                    if count == 0 {
                        let formatted = format!("{}{}  PID:{}  \"{}\"{}", GREEN, count+1, args_collection[1], args_collection[0].to_string(), RESET);
                        writeln!(&mut output_pipe, "{}", formatted).unwrap();
                    }else{
                        writeln!(&mut output_pipe, "{}  PID:{}  \"{}\"", count+1, args_collection[1], args_collection[0].to_string()).unwrap();
                    }
                    count += 1;
                }
                writeln!(&mut output_pipe, "==========================================").unwrap();
                writeln!(&mut output_pipe, "{}", "Oxy-over").unwrap();
           }
       }

    });

    let command_list_update = Arc::clone(&command_list);
    let thread_command_producer = thread::spawn( move  || {
        let mut command_pipe = TempPipe::new("oxy_pipe");
        println!("Listening commands on: {}", command_pipe.get_path().display());
        let command_tx_clone = command_tx.clone();
        for line in std::io::BufReader::new(command_pipe.get_pipe()).lines(){
            let command = String::from(line.unwrap());
            command_tx_clone.send(command.clone()).unwrap();
            let mut command_list_guard = command_list_update.lock().unwrap();
            command_list_guard.push_back(command);
        }
    });



    let command_list_pop = Arc::clone(&command_list);
    let current_command_output_update = Arc::clone(&current_command_output);
    let thread_consumer = thread::spawn(move ||
        {
            match spawn_child_process(command_rx, current_command_output_update, command_list_pop){
                Ok(_)=>println!(""),
                Err(e)=>eprintln!("Error: {}", e),
            }
        });
    let _ = thread_instruction_producer.join();
    let _ = thread_command_producer.join();
    let _ = thread_consumer.join();
    let _ = thread_instruct.join();

}

fn spawn_child_process(command_rx:Receiver<String>,current_command_output_update: Arc<Mutex<VecDeque<String>>>, command_list_pop: Arc<Mutex<VecDeque<String>>>) -> std::io::Result<()> {
    let command_rx_clone = command_rx.clone();
    for command in command_rx_clone {
        let args_collection: Vec<&str> = command.split(";;").collect();

        if args_collection.len()!=2 {
            continue;
        }

        println!("Client pid: {} : Requested command : {}", args_collection[1], args_collection[0]);

        let process = Command::new("sh")
            .arg("-c")
            .arg(args_collection[0])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = process.stdout.expect("Failed to capture stdout");
        let reader = BufReader::new(stdout).lines();

        let output_pipe_name: String = "oxy_pip_output_".to_string() + &args_collection[1];
        let mut output_pipe = Pipe::with_name(&output_pipe_name).unwrap();

        reader.for_each(|line|{
            let mut current_output = current_command_output_update.lock().unwrap();
            let str_line = String::from(line.unwrap());
            current_output.push_back(str_line.clone());
            println!("{}",str_line);
            writeln!(&mut output_pipe, "{}", str_line).unwrap();
        });

        writeln!(&mut output_pipe, "{}", "Oxy-over").unwrap();

        if let Ok(mut list )= command_list_pop.lock(){
            list.pop_front();
        }
    };

    Ok(())
}