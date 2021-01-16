use std::fmt::Display;
use crate::hashmap::HashMap;
use crate::stack::Stack;
use corundum::default::*;
use corundum::sync::VWeak;
use regex::Regex;

type P = BuddyAlloc;

struct ConsumerData {
    buf: PString,
    local: HashMap<PString, u64>,
    active: bool,
}

pub struct Consumer {
    pattern: PString,
    data: PMutex<ConsumerData>,
    lines: Parc<PMutex<Stack<PString>>>,
    private_lines: PMutex<Stack<PString>>,
}

impl Consumer {
    pub fn new(
        pattern: &str,
        lines: Parc<PMutex<Stack<PString>>>,
        j: &Journal,
    ) -> Self {
        Self {
            pattern: PString::from_str(pattern, j),
            lines,
            private_lines: PMutex::new(Stack::new(), j),
            data: PMutex::new(
                ConsumerData {
                    buf: PString::new(j),
                    local: HashMap::new(j),
                    active: true,
                },
                j,
            ),
        }
    }

    /// Starts processing `lines` and updating `words`
    pub fn start(slf: VWeak<Consumer, P>, dist: bool, isolated: bool) {
        loop {
            // Read from global buffer to the local buffer
            if !P::transaction(|j| {
                if let Some(slf) = slf.promote(j) {
                    let mut this = slf.data.lock(j);
                    if this.buf.is_empty() {
                        let mut lines = if dist || !isolated {
                            slf.lines.lock(j)
                        } else {
                            slf.private_lines.lock(j)
                        };
                        let line = lines.pop(j);
                        if unsafe { crate::PRINT } {
                            if dist || !isolated {
                                eprint!(
                                    "\r\x1b[?25lRemaining: {:<12} Memory usage: {:<9} bytes \x1b[?25h",
                                    lines.len(),
                                    P::used()
                                );
                            } else {
                                eprint!(
                                    "\r\x1b[?25lMemory usage: {:<9} bytes \x1b[?25h",
                                    P::used()
                                );
                            } 
                        }
                        if let Some(line) = line {
                            this.buf = line;
                            true // Still working
                        } else {
                            this.active
                        }
                    } else {
                        true
                    }
                } else {
                    false
                }
            }).unwrap() {
                return;
            }

            // counting words
            P::transaction(|j| {
                if let Some(slf) = slf.promote(j) {
                    let mut this = slf.data.lock(j);
                    if !this.buf.is_empty() {
                        if dist {
                            let mut lines = slf.private_lines.lock(j);
                            lines.push(this.buf.pclone(j), j);
                        } else {
                            let buf = this.buf.to_string();
                            let re = Regex::new(slf.pattern.as_str()).unwrap();
    
                            for cap in re.captures_iter(&buf) {
                                let w = cap.get(1).unwrap().as_str().to_pstring(j);
                                this.local.update_with(&w, j, |v| v + 1);
                            }
                        }
                        this.buf.clear();
                    }
                }
            }).unwrap();
        }
    }

    pub fn collect(&self, words: Parc<PMutex<HashMap<PString, u64>>>, j: &Journal) {
        let mut this = self.data.lock(j);
        let mut words = words.lock(j);
        this.local.foreach(|k, v| {
            words.update_with(k, j, |v0| v0 + v);
        });
        this.local.clear(j);
    }

    pub fn stop_when_finished(&self) {
        P::transaction(|j| {
            let mut this = self.data.lock(j);
            this.active = false;
        }).unwrap();
    }

    pub fn activate(&self) {
        P::transaction(|j| {
            let mut this = self.data.lock(j);
            this.active = true;
        }).unwrap();
    }
}

impl Display for Consumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let s = P::transaction(move |j| {
            let data = self.data.lock(j);
            format!("buf=\x1b[0;31m{}\x1b[0m\nlocal:\n\x1b[0;31m{}\x1b[0m", data.buf, data.local)
        }).unwrap();
        writeln!(f, "{}", s)?;
        Ok(())
    }
}