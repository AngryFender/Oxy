use std::fs;
use ipipe::Pipe;

pub(crate) struct TempPipe {
    pipe: Pipe,
}
impl TempPipe {
    pub(crate) fn new(name: &str) -> Self {
        let pipe = Pipe::with_name(name).expect("Failed to create oxy_pipe");

        Self{
            pipe
        }
    }
    pub(crate) fn get_pipe(&mut self) -> &mut Pipe {
        &mut self.pipe
    }
    pub(crate) fn get_path(&self) -> &std::path::Path {
        &self.pipe.path()
    }
}

impl Drop for TempPipe {
    fn drop(&mut self) {
        match fs::remove_file(&self.pipe.path()) {
            Ok(_) => println!("Successfully removed the named pipe: {}", &self.pipe.path().display()),
            Err(_e) => eprintln!("Failed to remove the named pipe: {}", &self.pipe.path().display()),
        }
    }
}