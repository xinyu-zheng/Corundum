#![allow(dead_code)]
//! # Word Count example with MapReduce model

mod vconsumer;
mod vhashmap;
mod vproducer;
mod vstack;

use vconsumer::VConsumer;
use vhashmap::*;
use vproducer::VProducer;
use vstack::*;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;

pub static mut PRINT: bool = true;

fn help() {
    println!("usage: grep [OPTIONS] list-file");
    println!();
    println!("OPTIONS:");
    println!("  -p num        Search pattern (Default '\\w+')");
    println!("  -r num        Number of reader (producer) threads (Default 1)");
    println!("  -c num        Number of consumer threads (Default 1)");
    println!("  -f file       Pool filename (Default ./wc.pool)");
    println!("  -N            Do not print output (perf test)");
    println!("  -D            Distribute text partitions to consumers (used with -I)");
    println!("  -I            Continue counting in isolation (used with -D)");
    println!("  -C            Continue from the previous run (used with -P)");
    println!("  -P            Prepare only (do not run threads, used with -C)");
    println!("  -h            Display help");
    println!();
    println!("The input list-file should contain a list files to read and count words.");
}

fn main() {
    let args: Vec<std::string::String> = env::args().collect();

    if args.len() < 2 {
        println!("usage: {} [OPTIONS] filename", args[0]);
        return;
    }

    let mut r = 1;
    let mut c = 1;
    let mut pool = "wc.pool".to_string();
    let mut filename = std::string::String::new();
    let mut i = 1;
    let mut cont = false;
    let mut prep = false;
    let mut dist = false;
    let mut isld = false;
    let mut pattern = "(\\w+)".to_string();
    while i < args.len() {
        let s = &args[i];
        if s == "-h" {
            help();
            return;
        } else if s == "-p" {
            if i == args.len() - 1 {
                panic!("-p requires an argument");
            }
            i += 1;
            pattern = format!("({})", args[i]);
        } else if s == "-r" {
            if i == args.len() - 1 {
                panic!("-r requires an argument");
            }
            i += 1;
            r = args[i].parse().expect("An integer expected");
            if r < 1 {
                panic!("Number of reader threads cannot be less than 1");
            }
        } else if s == "-c" {
            if i == args.len() - 1 {
                panic!("-c requires an argument");
            }
            i += 1;
            c = args[i].parse().expect("An integer expected");
            if c < 0 {
                panic!("Number of consumer threads cannot be less than 0");
            }
        } else if s == "-f" {
            if i == args.len() - 1 {
                panic!("-f requires an argument");
            }
            i += 1;
            pool = args[i].clone();
        } else if s == "-C" {
            cont = true;
        } else if s == "-N" {
            unsafe {PRINT = false;}
        } else if s == "-D" {
            dist = true;
        } else if s == "-I" {
            isld = true; cont = true;
        } else if s == "-P" {
            prep = true;
        } else if filename.is_empty() {
            filename = s.clone();
        } else {
            panic!("Unknown option `{}'", s);
        }
        i += 1;
    }

    struct Root {
        lines: Arc<Mutex<VStack<String>>>,
        words: Arc<Mutex<VHashMap<String, u64>>>,
        producers: RefCell<Vec<Arc<VProducer>>>,
        consumers: RefCell<Vec<Arc<VConsumer>>>,
    }
    
    let root = 
            Root {
                lines: Arc::new(Mutex::new(VStack::new())),
                words: Arc::new(Mutex::new(VHashMap::new())),
                producers: RefCell::new(Vec::new()),
                consumers: RefCell::new(Vec::new()),
            };

    {
        let mut producers = root.producers.borrow_mut();
        let mut consumers = root.consumers.borrow_mut();

        if !cont {
            producers.clear();
            consumers.clear();

            root.lines.lock().unwrap().clear();
            root.words.lock().unwrap().clear();

            let mut files = vec![];
            let f = BufReader::new(
                File::open(&filename).expect(&format!("cannot open `{}`", &filename)),
            );

            for line in f.lines() {
                files.push(line.unwrap());
            }
            let p = r.min(files.len());
            let b = files.len() / p;
            for i in 0..p + 1 {
                if i * b < files.len() {
                    producers.push(
                        Arc::new(
                            VProducer::new(
                                files[i * b..files.len().min((i + 1) * b)].to_vec(),
                                root.lines.clone()
                            )
                        )
                    );
                }
            }
            for _ in 0..c.max(1) {
                consumers.push(
                    Arc::new(
                        VConsumer::new(&pattern, root.lines.clone()),
                    )
                );
            }
        }
    }

    eprintln!(
        "Total remaining from previous run: {} ",
        root.lines.lock().unwrap().len()
    );

    if !prep {
        let producers = root.producers.borrow();
        let consumers = root.consumers.borrow();

        let mut p_threads = vec![];
        let mut c_threads = vec![];

        for p in &*producers {
            let p = Arc::downgrade(p);
            p_threads.push(thread::spawn(move || VProducer::start(p)))
        }

        if c == 0 {
            for thread in p_threads {
                thread.join().unwrap()
            }

            for consumer in &*consumers {
                consumer.activate();
            }

            if !dist {
                for c in &*consumers {
                    let c = Arc::downgrade(c);
                    c_threads.push(thread::spawn(move || VConsumer::start(c, isld)))
                }
            }
        } else {
            for consumer in &*consumers {
                consumer.activate();
            }

            if !dist {
                for c in &*consumers {
                    let c = Arc::downgrade(c);
                    c_threads.push(thread::spawn(move || VConsumer::start(c, isld)))
                }
            }

            for thread in p_threads {
                thread.join().unwrap()
            }
        }

        if !dist {
            // Notifying consumers that there is no more feeds
            for consumer in &*consumers {
                consumer.stop_when_finished();
            }
        } else {
            {
                let mut lines = root.lines.lock().unwrap();
                let mut work = true;
                while work {
                    for consumer in &*consumers {
                        work = consumer.take_one(&mut lines);
                    }
                }
            }
        }

        for thread in c_threads {
            thread.join().unwrap()
        }

        eprintln!();
        
        if dist {
            let mut i=0;
            for c in &*consumers {
                i += 1;
                eprintln!("c#{}.private_buf_size = {}", i, c.private_buf_size());
            }
        }

        // Display results
        if unsafe {PRINT} {
            {
                for c in &*consumers {
                    c.collect(root.words.clone());
                }
                let words = root.words.lock().unwrap();
                println!("{}", words);
            }
        }
    }
}
