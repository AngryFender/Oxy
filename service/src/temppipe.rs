struct TempPipe {
    path: String,
}
impl TempPipe {
    fn new(path: &str) -> Self {
        Self{path : path.to_string()}

    }
}

impl Drop for TempPipe {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}