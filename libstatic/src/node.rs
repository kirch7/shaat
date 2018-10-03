// Copyright 2018 Cássio Kirch.

use super::{io, time};
use std::fs::File;
use std::io::{BufReader, Read};

pub struct Node {
    pub last_update: time::SystemTime,
    pub content: Result<Vec<u8>, String>,
}

impl Node {
    pub fn must_update(&self, file: &File) -> Result<bool, String> {
        let metadata = file.metadata().map_err(|e| e.to_string())?;

        let modified = metadata.modified().map_err(|e| e.to_string())?;

        Ok(modified > self.last_update)
    }

    pub fn update(&mut self, file: &File) {
        let metadata = file.metadata();
        if metadata.is_err() {
            self.content = Err(metadata.unwrap_err().to_string());
            return;
        }
        let metadata = metadata.unwrap();

        let modified = metadata.modified();
        if modified.is_err() {
            self.content = Err(modified.unwrap_err().to_string());
            return;
        }

        let modified = modified.unwrap();
        if modified > self.last_update {
            self.last_update = modified;
            self.content = get_file_content(&file).map_err(|e| e.to_string());
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node {
            last_update: time::UNIX_EPOCH,
            content: Err("Still unset".into()),
        }
    }
}

fn get_file_content(file: &File) -> io::Result<Vec<u8>> {
    let mut buf = BufReader::new(file);
    let mut v = Vec::default();
    buf.read_to_end(&mut v).map(|_| Ok(v))?
}
