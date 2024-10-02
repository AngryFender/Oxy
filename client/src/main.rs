use std::env;
use ipipe::Pipe;
use std::io::Write;
use std::process::Command;

fn main() {
    if env::consts::OS != "linux" {
        return;
    }
    println!("Starting, Oxy!");

    let args: Vec<String> = env::args().collect();
    if args.len() <3{
        return;
    }

    if args[1] == "run" {
        let mut pipe = Pipe::with_name("oxy_pipe").unwrap();
        writeln!(&mut pipe,"{}", args[2].to_string());


       /* let output = Command::new("sh")
            .arg("-c")
            .arg(args[2].to_string())
            .output()
            .expect("failed to execute process");

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();*/
    }
}
