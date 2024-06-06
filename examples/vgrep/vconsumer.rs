use std::{fmt::Display, str::FromStr};
use crate::vhashmap::VHashMap;
use crate::vstack::VStack;
use regex::Regex;
use corundum::may_crash;
use std::sync::{Arc, Mutex, Weak};

struct VConsumerData {
    buf: String,
    local: VHashMap<String, u64>,
    active: bool,
    private_lines: VStack<String>,
}

pub struct VConsumer {
    pattern: String,
    data: Mutex<VConsumerData>,
    lines: Arc<Mutex<VStack<String>>>,
}

impl VConsumer {
    pub fn new(
        pattern: &str,
        lines: Arc<Mutex<VStack<String>>>,
    ) -> Self {
        Self {
            pattern: String::from_str(pattern).unwrap(),
            lines,
            data: Mutex::new(
                VConsumerData {
                    buf: String::new(),
                    private_lines: VStack::new(),
                    local: VHashMap::new(),
                    active: true,
                }
            ),
        }
    }

    /// Starts processing `lines` and updating `words`
    pub fn start(slf: Weak<VConsumer>, isolated: bool) {
        loop {
            // Read from global buffer to the local buffer
            if !{                                            may_crash!();
                if let Some(slf) = slf.upgrade() {                             may_crash!();
                    let mut this = slf.data.lock().unwrap();                            may_crash!();
                    if this.buf.is_empty() {                                    may_crash!();
                        let mut rem = 0;                                        may_crash!();
                        let line = if !isolated {                               may_crash!();
                            let mut lines = slf.lines.lock().unwrap();                  may_crash!();
                            rem = lines.len();                                  may_crash!();
                            lines.pop()
                        } else {                                                may_crash!();
                            this.private_lines.pop()
                        };
                        if let Some(line) = line {                              may_crash!();
                            this.buf = line;                                    may_crash!();
                            true // Still working
                        } else {                                                may_crash!();
                            this.active
                        }
                    } else {                                                    may_crash!();
                        true
                    }
                } else {
                    false
                }
            }
            {
                return;
            }

            // counting words
            {
                if let Some(slf) = slf.upgrade() {                             may_crash!();
                    let mut this = slf.data.lock().unwrap();                            may_crash!();
                    if !this.buf.is_empty() {                                   may_crash!();
                        let buf = this.buf.to_string();                         may_crash!();
                        let re = Regex::new(slf.pattern.as_str()).unwrap();     may_crash!();

                        for cap in re.captures_iter(&buf) {                     may_crash!();
                            let w = String::from_str(cap.get(1).unwrap().as_str()).unwrap(); may_crash!();
                            this.local.update_with(&w, |v| v + 1);           may_crash!();
                        }
                        this.buf.clear();                                       may_crash!();
                    }
                }
            }
        }
    }

    pub fn collect(&self, words: Arc<Mutex<VHashMap<String, u64>>>) {
        let mut this = self.data.lock().unwrap();
        let mut words = words.lock().unwrap();
        this.local.foreach(|k, v| {
            words.update_with(k,  |v0| v0 + v);
        });
        this.local.clear();
    }

    pub fn stop_when_finished(&self) {
        {
            let mut this = self.data.lock().unwrap();
            this.active = false;
        }
    }

    pub fn activate(&self) {
        {
            let mut this = self.data.lock().unwrap();
            this.active = true;
        }
    }

    pub fn take_one(&self, lines: &mut VStack<String>) -> bool {
        let mut this = self.data.lock().unwrap();
        if let Some(line) = lines.pop() {
            this.private_lines.push(line);
            true
        } else {
            false
        }
    }

    pub fn private_buf_size(&self) -> usize {
        {
            let this = self.data.lock().unwrap();
            this.private_lines.len()
        }
    }
}

impl Display for VConsumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let s = {
            let data = self.data.lock().unwrap();
            format!("local:\n\x1b[0;31m{}\x1b[0m", data.local)
        };
        writeln!(f, "{}", s)?;
        Ok(())
    }
}