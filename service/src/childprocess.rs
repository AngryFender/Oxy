use std::collections::{HashSet, VecDeque};
use std::io::{Write, BufRead, BufReader};
use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use crossbeam_channel::Receiver;
use ipipe::Pipe;
use crate::{RED, RESET};
use crate::commandentry::CommandEntry;

pub(crate) fn spawn_child_process(child_arc_clone: Arc<Mutex<Option<Child>>>, command_rx:Receiver<String>, current_command_output_update: Arc<Mutex<VecDeque<String>>>, command_entry_pop: Arc<Mutex<VecDeque<CommandEntry>>>, ban_entries_consume: Arc<Mutex<HashSet<String>>>, last_command_output_update: Arc<Mutex<VecDeque<String>>>) -> std::io::Result<()> {
    for line in command_rx.clone() {
        let args: Vec<&str> = line.split(";;").collect();

        if args.len()!=2 {
            continue;
        }
        let pid = args[0];
        let command = args[1];


        {
            let mut ban_entries_consume = ban_entries_consume.lock().unwrap();
            if ban_entries_consume.contains(pid) {
                ban_entries_consume.remove(pid);
                continue;
            }
        }

        println!("Client@{} command ← {}", pid, command);

        let mut process = Command::new("sh")
            .arg("-c")
            .arg(command)
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

        let output_pipe_name: String = "oxy_pip_output_".to_string() + pid;
        let mut output_pipe = Pipe::with_name(&output_pipe_name).unwrap();

        stdout_reader.for_each(|line|{
            let mut current_output = current_command_output_update.lock().unwrap();
            let str_line = String::from(line.unwrap());
            current_output.push_back(str_line.clone());
            writeln!(&mut output_pipe, "{}", str_line).unwrap();
        });
        stderr_reader.for_each(|line|{
            let mut current_output = current_command_output_update.lock().unwrap();
            let str_line = String::from(line.unwrap());
            current_output.push_back(str_line.clone());
            writeln!(&mut output_pipe, "{}{}{}",RED, str_line,RESET).unwrap();
        });
        writeln!(&mut output_pipe, "{}", "Oxy-over").unwrap();
        {
            let mut current_output = current_command_output_update.lock().unwrap();
            let mut last_output = last_command_output_update.lock().unwrap();
            last_output.append(&mut current_output);
            while last_output.len() > 500 {
                last_output.pop_front();
            }
            current_output.clear();
        }

        if let Ok(mut list )= command_entry_pop.lock(){
            list.pop_front();
        }
    };

    Ok(())
}