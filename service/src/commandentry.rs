use ipipe::Pipe;

pub(crate) struct CommandEntry {
    command: String,
    pid: String
}

impl CommandEntry {
    pub(crate) fn new(command: String, pid: String) -> Self {
        Self{
            command,
            pid
        }
    }

    pub(crate) fn get_command(&self) -> &String {
        &self.command
    }
    pub(crate) fn get_pid(&self) -> &String {
        &self.pid
    }
}