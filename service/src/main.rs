use ipipe::{pprint, Pipe};
use std::io::BufRead;
use std::thread;
use std::io::Write;
use std::process::Command;

fn main() {
    println!("starting oxyd service...");

    let mut pipe = Pipe::with_name("oxy_pipe").unwrap();
    println!("Pipe path:{}", pipe.path().display());

    for line in std::io::BufReader::new(pipe).lines(){
        let output = Command::new("sh")
            .arg("-c")
            .arg(line.unwrap())
            .output()
            .expect("failed to execute process");

        print!("{}",String::from_utf8_lossy(&output.stdout));
    }
}
