mod temppipe;
use temppipe::TempPipe;
use ipipe::{Pipe};
use std::io::BufRead;
use std::thread;
use std::io::Write;
use std::process::{Command, Stdio};
use crossbeam_channel::unbounded;
use std::sync::mpsc;
use std::thread::spawn;
use log::error;

fn main() {
    println!("Starting oxyd service...");

    let (command_tx,command_rx) = mpsc::channel();
    let (instruction_tx,instruction_rx) = mpsc::channel();

    let thread_instruction_producer = thread::spawn( move  || {
        let mut instruction_pipe = TempPipe::new("oxy_instruction_pipe");
        println!("Listening instructions on: {}", instruction_pipe.get_path().display());
        for line in std::io::BufReader::new(instruction_pipe.get_pipe()).lines(){
            let instruction = String::from(line.unwrap());
            instruction_tx.send(instruction.clone()).unwrap();
        }
    });

    let instructThread = thread::spawn(move ||{
       for instruction in instruction_rx {
           let argsCollection: Vec<&str> = instruction.split(";;").collect();

           if(argsCollection.len()!=2) {
               continue;
           }

           println!("Client pid: {} : Requested instruction : {}",argsCollection[1], argsCollection[0]);

           if argsCollection[0] == "status"{
                //TODO: Print status
                // 1. Show total number of commands left
                // 2. Show stdout of the current process?
               let outputPipeName: String = "oxy_pip_output_".to_string() + &argsCollection[1];
               let mut outputPipe = Pipe::with_name(&outputPipeName).unwrap();
               //writeln!(&mut outputPipe,"No of commands: {}",command_tx. ).unwrap();
               //writeln!(&mut outputPipe,"{}", "Oxy-over").unwrap();
           }

       }

    });

    let thread_command_producer = thread::spawn( move  || {
        let mut command_pipe = TempPipe::new("oxy_pipe");
        println!("Listening commands on: {}", command_pipe.get_path().display());
        for line in std::io::BufReader::new(command_pipe.get_pipe()).lines(){
            let command = String::from(line.unwrap());
            command_tx.send(command.clone()).unwrap();
        }
    });

    let commands: Vec<&Command> ;
    let mut current_command:String = String::new();
    let threadConsumer = thread::spawn(move || {
        for command in command_rx {
            let argsCollection: Vec<&str> = command.split(";;").collect();

            if(argsCollection.len()!=2){
                continue;
            }

            println!("Client pid: {} : Requested command : {}",argsCollection[1], argsCollection[0]);

            current_command = command.clone();
            let process = Command::new("sh")
                .arg("-c")
                .arg(argsCollection[0])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();
            //let output=process.unwrap().wait_with_output().expect("failed to execute process");
            let output=process.unwrap().wait_with_output().expect("Failed to wait on process");

            let outputMessage =  String::from_utf8_lossy(&output.stdout);
            let errorMessage =  String::from_utf8_lossy(&output.stderr);
            //println!(" ↳ {}",outputMessage);

            let outputPipeName: String = "oxy_pip_output_".to_string() + &argsCollection[1];
            let mut outputPipe = Pipe::with_name(&outputPipeName).unwrap();
            writeln!(&mut outputPipe,"{}", outputMessage).unwrap();
            writeln!(&mut outputPipe,"{}", errorMessage).unwrap();
            writeln!(&mut outputPipe,"{}", "Oxy-over").unwrap();
        };

    });
    let _ = thread_instruction_producer.join();
    let _ = thread_command_producer.join();
}
