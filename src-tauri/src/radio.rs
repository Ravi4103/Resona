use serde::{Deserialize, Serialize};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioStation {
    pub id: String,
    pub name: String,
    pub url: String,
    pub genre: String,
    pub logo_url: String,
    pub country: String,
    pub language: String,
    pub bitrate: i64,
}

pub struct RadioBuffer {
    pub data: Vec<u8>,
    pub finished: bool,
}

pub struct SharedBuffer {
    pub inner: Mutex<RadioBuffer>,
    pub cond: Condvar,
}

pub struct StreamReader {
    pub shared: Arc<SharedBuffer>,
    pub pos: usize,
}

impl Read for StreamReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut rb = self.shared.inner.lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "lock poisoned")
        })?;
        loop {
            if self.pos < rb.data.len() {
                let available = rb.data.len() - self.pos;
                let to_read = buf.len().min(available);
                buf[..to_read].copy_from_slice(&rb.data[self.pos..self.pos + to_read]);
                self.pos += to_read;
                return Ok(to_read);
            }
            if rb.finished {
                return Ok(0);
            }
            rb = self.shared.cond.wait(rb).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::Other, "cond wait poisoned")
            })?;
        }
    }
}

impl std::io::Seek for StreamReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let rb = self.shared.inner.lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "lock poisoned")
        })?;
        let new_pos = match pos {
            std::io::SeekFrom::Start(offset) => {
                if offset <= rb.data.len() as u64 {
                    Some(offset as usize)
                } else {
                    None
                }
            }
            std::io::SeekFrom::Current(offset) => {
                let np = self.pos as i64 + offset;
                if np >= 0 && (np as usize) <= rb.data.len() {
                    Some(np as usize)
                } else {
                    None
                }
            }
            std::io::SeekFrom::End(offset) => {
                let np = rb.data.len() as i64 + offset;
                if np >= 0 && (np as usize) <= rb.data.len() {
                    Some(np as usize)
                } else {
                    None
                }
            }
        };
        match new_pos {
            Some(p) => {
                self.pos = p;
                Ok(p as u64)
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek out of bounds",
            )),
        }
    }
}

pub fn spawn_downloader(
    url: String,
    shared: Arc<SharedBuffer>,
    playing: Arc<AtomicBool>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let client = match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to create HTTP client: {e}");
                let mut rb = shared.inner.lock().unwrap();
                rb.finished = true;
                shared.cond.notify_all();
                return;
            }
        };

        let response = match client.get(&url).send() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Radio download connect error: {e}");
                let mut rb = shared.inner.lock().unwrap();
                rb.finished = true;
                shared.cond.notify_all();
                return;
            }
        };

        if !response.status().is_success() {
            eprintln!("Radio HTTP error: {}", response.status());
            let mut rb = shared.inner.lock().unwrap();
            rb.finished = true;
            shared.cond.notify_all();
            return;
        }

        let mut buf = [0u8; 8192];
        let mut response_reader = response;
        loop {
            if !playing.load(Ordering::Acquire) {
                break;
            }
            match response_reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let mut rb = shared.inner.lock().unwrap();
                    rb.data.extend_from_slice(&buf[..n]);
                    shared.cond.notify_all();
                }
                Err(e) => {
                    eprintln!("Radio download error: {e}");
                    break;
                }
            }
        }
        let mut rb = shared.inner.lock().unwrap();
        rb.finished = true;
        shared.cond.notify_all();
    })
}
