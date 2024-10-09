use std::fs;
use ipipe::Pipe;
struct TempPipe {
    pipe: Pipe,
}
impl TempPipe {
    fn new(name: &str) -> Self {
        let pipe = Pipe::with_name(name).expect("Failed to create oxy_pipe");

        Self{
            pipe
        }
    }

    fn get_pipe(&self) -> &Pipe {
        &self.pipe
    }
}

impl Drop for TempPipe {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.pipe.path());
    }
}