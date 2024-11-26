
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
}