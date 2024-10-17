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

    let (tx,rx) = mpsc::channel();

    let threadProducer = thread::spawn( move || {
        let mut tempPipe = TempPipe::new("oxy_pipe");
        println!("Listening commands on: {}", tempPipe.get_path().display());
        for line in std::io::BufReader::new(tempPipe.get_pipe()).lines(){
            let val = String::from(line.unwrap());
            tx.send(val.clone()).unwrap();
            println!("{}",val);
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

    let _ = threadProducer.join();
}
