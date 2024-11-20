use std::collections::VecDeque;
use std::io::{Write, BufRead, BufReader};
use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use crossbeam_channel::Receiver;
use ipipe::Pipe;
use crate::{RED, RESET};

pub(crate) fn spawn_child_process(child_arc_clone: Arc<Mutex<Option<Child>>>, command_rx:Receiver<String>, current_command_output_update: Arc<Mutex<VecDeque<String>>>, command_list_pop: Arc<Mutex<VecDeque<String>>>) -> std::io::Result<()> {
    let command_rx_clone = command_rx.clone();
    for command in command_rx_clone {
        let args_collection: Vec<&str> = command.split(";;").collect();

        if args_collection.len()!=2 {
            continue;
        }

        println!("Client pid: {} : Requested command : {}", args_collection[1], args_collection[0]);

        let mut process = Command::new("sh")
            .arg("-c")
            .arg(args_collection[0])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;


        let stdout = process.stdout.take().expect("Failed to capture stdout");
        let stderr = process.stderr.take().expect("Failed to capture stdout");
        {
            let mut child_arc_guard = child_arc_clone.lock().unwrap();
            *child_arc_guard = Some(process);
        }
        let stdout_reader = BufReader::new(stdout).lines();
        let stderr_reader = BufReader::new(stderr).lines();

        let output_pipe_name: String = "oxy_pip_output_".to_string() + &args_collection[1];
        let mut output_pipe = Pipe::with_name(&output_pipe_name).unwrap();

        stdout_reader.for_each(|line|{
            let mut current_output = current_command_output_update.lock().unwrap();
            let str_line = String::from(line.unwrap());
            current_output.push_back(str_line.clone());
            println!("{}",str_line);
            writeln!(&mut output_pipe, "{}", str_line).unwrap();
        });
        stderr_reader.for_each(|line|{
            let mut current_output = current_command_output_update.lock().unwrap();
            let str_line = String::from(line.unwrap());
            current_output.push_back(str_line.clone());
            println!("{}",str_line);
            writeln!(&mut output_pipe, "{}{}{}",RED, str_line,RESET).unwrap();
        });
        writeln!(&mut output_pipe, "{}", "Oxy-over").unwrap();

        if let Ok(mut list )= command_list_pop.lock(){
            list.pop_front();
        }
    };

    Ok(())
}