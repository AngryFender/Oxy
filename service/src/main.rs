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

    let handle = thread::spawn(|| {
        let mut tempPipe = TempPipe::new("oxy_instructions");
        println!("Listening instructions on: {}", tempPipe.get_path().display());
        for line in std::io::BufReader::new(tempPipe.get_pipe()).lines(){
            println!("{}",line.unwrap());
        }
    });

    let mut tempPipe = TempPipe::new("oxy_pipe");
    println!("Listening on: {}", tempPipe.get_path().display());

    for line in std::io::BufReader::new(tempPipe.get_pipe()).lines(){
        let allAgrs: String = line.unwrap();
        let argsCollection: Vec<&str> = allAgrs.split(";;").collect();

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
    }
}
