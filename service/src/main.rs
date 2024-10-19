mod temppipe;
use temppipe::TempPipe;
use ipipe::{pprint, Pipe};
use std::io::BufRead;
use std::thread;
use std::io::Write;
use std::process::Command;
use std::sync::mpsc;

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

    let thread_command_producer = thread::spawn( move  || {
        let mut command_pipe = TempPipe::new("oxy_pipe");
        println!("Listening commands on: {}", command_pipe.get_path().display());
        for line in std::io::BufReader::new(command_pipe.get_pipe()).lines(){
            let command = String::from(line.unwrap());
            command_tx.send(command.clone()).unwrap();
        }
    });

    let threadConsumer = thread::spawn(move || {
        for command in rx {
            let argsCollection: Vec<&str> = command.split(";;").collect();

            if(argsCollection.len()!=2){
                continue;
            }

            println!("Requested command: {}",argsCollection[0]);
            println!("Requested pid: {}",argsCollection[1]);

            let output = Command::new("sh")
                .arg("-c")
                .arg(argsCollection[0])
                .output()
                .expect("failed to execute process");

            let outputMessage =  String::from_utf8_lossy(&output.stdout);
            println!(" ↳ {}",outputMessage);
            let outputPipeName: String = "oxy_pip_output_".to_string() + &argsCollection[1];
            let mut outputPipe = Pipe::with_name(&outputPipeName).unwrap();
            writeln!(&mut outputPipe,"{}", outputMessage).unwrap();
            writeln!(&mut outputPipe,"{}", "Oxy-over").unwrap();
        };

    });
    let _ = thread_instruction_producer.join();
    let _ = thread_command_producer.join();
}
