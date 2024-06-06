use crate::vstack::VStack;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::str::FromStr;
use std::sync::{Arc, Mutex, Weak};

const BATCH_SIZE: usize = 1024; // number of chars per job

pub struct VProducer {
    filenames: Vec<String>,
    // 0: file index, 1: position
    pos: Mutex<(usize, u64)>,
    lines: Arc<Mutex<VStack<String>>>,
}

impl VProducer {
    pub fn new(
        filelist: Vec<String>,
        lines: Arc<Mutex<VStack<String>>>,
    ) -> Self {
        let mut filenames = Vec::with_capacity(filelist.len());
        for filename in filelist {
            filenames.push(String::from_str(&filename).unwrap());
        }
        Self {
            filenames,
            lines,
            pos: Mutex::new((0, 0)),
        }
    }

    /// Starts reading the files and adding to the `lines`
    pub fn start(this: Weak<Self>) {
        loop {
            if !{
                if let Some(this) = this.upgrade() {
                    let mut pos = this.pos.lock().unwrap();
                    if pos.0 < this.filenames.len() {
                        let filename = &this.filenames[pos.0];
                        let mut f =
                            BufReader::new(File::open(filename.as_str()).expect("open failed"));
                        let mut read = 0;
                        if f.seek(SeekFrom::Start(pos.1)).is_ok() {
                            let mut buf = Vec::<u8>::new();
                            let mut line = Vec::<u8>::new();
                            let mut lines = this.lines.lock().unwrap();
                            loop {
                                let r = f.read_until(b'\n', &mut buf).expect("read_until failed");
                                if r != 0 {
                                    read += r;
                                    line.append(&mut buf);
                                    if read >= BATCH_SIZE {
                                        let s = String::from_utf8(line).expect("from_utf8 failed");
                                        lines.push(String::from_str(&s).unwrap());
                                        pos.1 += read as u64;
                                        break;
                                    }
                                } else {
                                    if !line.is_empty() {
                                        let s = String::from_utf8(line).expect("from_utf8 failed");
                                        lines.push(String::from_str(&s).unwrap());
                                    }
                                    pos.1 = 0;
                                    pos.0 += 1;
                                    break;
                                }
                            }
                        } else {
                            pos.1 = 0;
                            pos.0 += 1;
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            {
                return;
            }
        }
    }
}
