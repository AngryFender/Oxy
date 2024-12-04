mod temppipe;
mod childprocess;
mod commandentry;

use std::collections::{HashSet, VecDeque};
use temppipe::TempPipe;
use ipipe::{Pipe};
use std::io::{BufRead};
use std::thread;
use std::io::Write;
use std::ops::DerefMut;
use std::process::{Child};
use crossbeam_channel::{unbounded};
use std::sync::{mpsc, Arc, Mutex};
use childprocess::spawn_child_process;
use crate::commandentry::CommandEntry;

// ANSI escape codes for colors
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

fn main()  {
    println!("Starting oxyd service...");

    let (command_tx,command_rx) = unbounded();
    let (instruction_tx,instruction_rx) = mpsc::channel();
    let command_entries = Arc::new(Mutex::new(VecDeque::<CommandEntry>::new()));
    let ban_entries = Arc::new(Mutex::new(HashSet::new()));
    let current_command_output = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let last_command_output = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let child_arc: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

    let thread_instruction_producer = thread::spawn( move  || {
        let mut instruction_pipe = TempPipe::new("oxy_instruction_pipe");
        println!("Listening instructions on: {}", instruction_pipe.get_path().display());
        for line in std::io::BufReader::new(instruction_pipe.get_pipe()).lines(){
            let instruction = String::from(line.unwrap());
            instruction_tx.send(instruction.clone()).unwrap();
        }
    });

    let current_command_stdout_output = Arc::clone(&current_command_output);
    let command_entry_manage = Arc::clone(&command_entries);
    let ban_entries_udpate = Arc::clone(&ban_entries);
    let child_arc_copy = Arc::clone(&child_arc);
    let thread_instruct = thread::spawn(move ||{
       for instruction in instruction_rx {
           let args: Vec<&str> = instruction.split(";;").collect();

           if args.len()<1 {
               continue;
           }

           let pid = args[0];
           let instruct = args[1];

           println!("instruction ← {}", args[1]);

           let output_pipe_name: String = "oxy_pip_output_".to_string() + pid;
           let mut output_pipe = Pipe::with_name(&output_pipe_name).unwrap();
           let mut entries = command_entry_manage.lock().unwrap();

           match instruct{
               "status" => {
                   writeln!(&mut output_pipe, "\nTotal commands: {}", entries.len()).unwrap();
                   writeln!(&mut output_pipe, "==========================================").unwrap();

                   let mut count = 0;
                   for entry in entries.iter(){
                       if count == 0 {
                           let formatted = format!("{}{}  PID:{}  \"{}\"{}", GREEN, count+1, entry.get_pid(),entry.get_command(), RESET);
                           writeln!(&mut output_pipe, "{}", formatted).unwrap();
                       }else{
                           writeln!(&mut output_pipe, "{}  PID:{}  \"{}\"", count+1, entry.get_pid(),entry.get_command()).unwrap();
                       }
                       count += 1;
                   }
               },
               "current" => {
                   let current_output = current_command_stdout_output.lock().unwrap();
                   writeln!(&mut output_pipe, "\n Current stdout: {}", entries.len()).unwrap();
                   writeln!(&mut output_pipe, "==========================================").unwrap();

                   for line in current_output.iter() {
                       writeln!(&mut output_pipe, "{}", line).unwrap();
                   }
               },
               "last" => {

               },
               "kill" => {
                   if entries.len() > 0 {
                       println!("Killing child process...");
                       let mut child_arc_guard = child_arc_copy.lock().unwrap();
                       child_arc_guard.deref_mut().take().unwrap().kill().expect("Failed to kill child process");

                       let remove_pipe_name: String = "oxy_pip_output_".to_string() + entries.get(0).unwrap().get_pid();
                       let mut remove_pipe = Pipe::with_name(&remove_pipe_name).unwrap();
                       writeln!(&mut remove_pipe, "{}", "Oxy-over").unwrap();
                       println!("Killing {}", entries.get(0).unwrap().get_pid());
                   }
               },
               "remove" => {
                   let remove_list = args[2].split(",").collect::<Vec<&str>>();
                   for rpid in remove_list.iter(){
                       println!("Removing child process {}", rpid);
                       if let Some(index) = entries.iter().position(|entry| entry.get_pid() == *rpid){
                           entries.remove(index);
                           let remove_pipe_name: String = "oxy_pip_output_".to_string() + *rpid;
                           let mut remove_pipe = Pipe::with_name(&remove_pipe_name).unwrap();
                           writeln!(&mut remove_pipe, "{}", "Oxy-over").unwrap();
                       }
                       let mut ban_update = ban_entries_udpate.lock().unwrap();
                       ban_update.insert(rpid.to_string());
                   }
               },
               _ => {}
           }

           writeln!(&mut output_pipe, "==========================================").unwrap();
           writeln!(&mut output_pipe, "{}", "Oxy-over").unwrap();       }
    });

    let command_entry_add = Arc::clone(&command_entries);
    let thread_command_producer = thread::spawn( move  || {
        let mut command_pipe = TempPipe::new("oxy_pipe");
        println!("Listening commands on: {}", command_pipe.get_path().display());
        let command_tx_clone = command_tx.clone();
        for line in std::io::BufReader::new(command_pipe.get_pipe()).lines(){
            let line = String::from(line.unwrap());
            let args: Vec<&str> = line.split(";;").collect();

            if args.len()!=2 {
                continue;
            }

            let mut command_entry_add_guard = command_entry_add.lock().unwrap();
            command_entry_add_guard.push_back(CommandEntry::new(args[1].to_string(), args[0].to_string()));
            command_tx_clone.send(line.clone()).unwrap();
        }
    });

    let command_entry_pop = Arc::clone(&command_entries);
    let current_command_output_update = Arc::clone(&current_command_output);
    let child_arc_clone = Arc::clone(&child_arc);
    let ban_entries_consume = ban_entries.clone();
    let thread_consumer = thread::spawn(move ||
        {
            match spawn_child_process(child_arc_clone, command_rx, current_command_output_update, command_entry_pop, ban_entries_consume){
                Ok(_)=>println!(""),
                Err(e)=>eprintln!("Error: {}", e),
            }
        });
    let _ = thread_instruction_producer.join();
    let _ = thread_command_producer.join();
    let _ = thread_consumer.join();
    let _ = thread_instruct.join();

}
