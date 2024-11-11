mod temppipe;

use std::collections::VecDeque;
use temppipe::TempPipe;
use ipipe::{Pipe};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::thread;
use std::io::Write;
use std::process::{Command, Stdio};
use crossbeam_channel::unbounded;
use std::sync::{mpsc, Arc, Mutex};

fn main() {
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
    let instructThread = thread::spawn(move ||{
       for instruction in instruction_rx {
           let argsCollection: Vec<&str> = instruction.split(";;").collect();

           if(argsCollection.len()!=2) {
               continue;
           }

           println!("Requested instruction : {}", argsCollection[0]);

               if argsCollection[0] == "status"{
                //TODO: Print status
                // 1. Show total number of commands left
                // 2. Show stdout of the current process?

                let outputPipeName: String = "oxy_pip_output_".to_string() + &argsCollection[1];
                let mut outputPipe = Pipe::with_name(&outputPipeName).unwrap();

                let mut command_list = command_list_consume.lock().unwrap();
                writeln!(&mut outputPipe,"==========================================").unwrap();
                for command in command_list.iter(){
                    let argsCollection: Vec<&str> = command.split(";;").collect();

                    if(argsCollection.len()!=2){
                        continue;
                    }
                    writeln!(&mut outputPipe,"PID:{}->\"{}\"",argsCollection[1], argsCollection[0].to_string()).unwrap();
                    println!("{}",&command);
                }
                writeln!(&mut outputPipe,"==========================================").unwrap();
                writeln!(&mut outputPipe,"{}", "Oxy-over").unwrap();
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
    let threadConsumer = thread::spawn(move || {
        let command_rx_clone = command_rx.clone();
        for command in command_rx_clone {
            let argsCollection: Vec<&str> = command.split(";;").collect();

            if(argsCollection.len()!=2){
                continue;
            }

            println!("Client pid: {} : Requested command : {}",argsCollection[1], argsCollection[0]);

            let process = Command::new("sh")
                .arg("-c")
                .arg(argsCollection[0])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            let stdout = process.unwrap().stdout.expect("Failed to capture stdout");
            let reader = BufReader::new(stdout).lines();

            let outputPipeName: String = "oxy_pip_output_".to_string() + &argsCollection[1];
            let mut outputPipe = Pipe::with_name(&outputPipeName).unwrap();

            reader.for_each(|line|{
                let mut current_output = current_command_output_update.lock().unwrap();
                let str_line = String::from(line.unwrap());
                    current_output.push_back(str_line.clone());
                    println!("{}",str_line);
                    writeln!(&mut outputPipe,"{}", str_line).unwrap();
            });

            writeln!(&mut outputPipe,"{}", "Oxy-over").unwrap();

            if let Ok(mut list )= command_list_pop.lock(){
                list.pop_front();
            }
        };
    });
    let _ = thread_instruction_producer.join();
    let _ = thread_command_producer.join();
}
