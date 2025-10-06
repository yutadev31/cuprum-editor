use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct EditorFile {
    file: File,
    path: PathBuf,
}

impl EditorFile {
    pub fn open(path: PathBuf) -> anyhow::Result<Self> {
        let file = OpenOptions::new().read(true).write(true).open(&path)?;
        Ok(Self { file, path })
    }

    pub fn read(&mut self) -> anyhow::Result<String> {
        self.file.seek(std::io::SeekFrom::Start(0))?;
        let mut buf = String::new();
        self.file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    pub fn write(&mut self, content: String) -> anyhow::Result<()> {
        self.file.seek(std::io::SeekFrom::Start(0))?;
        self.file.write_all(content.as_bytes())?;
        Ok(())
    }

    #[allow(dead_code)] // TODO
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    #[allow(dead_code)] // TODO
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path
    }
}
