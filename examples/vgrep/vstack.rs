use std::fmt::Display;
use std::sync::Arc;

struct VStackItem<T> {
    data: T,
    next: Option<Arc<VStackItem<T>>>,
}

pub struct VStack<T> {
    len: usize,
    head: Option<Arc<VStackItem<T>>>,
}

impl<T> VStack<T> {
    pub fn new() -> Self {
        Self { len: 0, head: None }
    }

    pub fn push(&mut self, data: T) {
        self.head = Some(Arc::new(
            VStackItem {
                data,
                next: self.head.clone(),
            }
        ));
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        if let Some(head) = &self.head {
            let d = head.data.clone();
            self.head = head.next.clone();
            self.len -= 1;
            Some(d)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.head = None;
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn print_top10(&self)
    where
        T: Display,
    {
        let mut curr = &self.head;
        for i in 0..10 {
            if let Some(c) = curr {
                print!("{:2>}: {}", i + 1, c.data);
                curr = &c.next;
            } else {
                break;
            }
        }
        println!(
            "----------------------------------------------------- Total: {}",
            self.len
        );
    }

    pub fn print_all(&self)
    where
        T: Display,
    {
        let mut curr = &self.head;
        for i in 0..self.len() {
            if let Some(c) = curr {
                print!("{:2>}: {}", i + 1, c.data);
                curr = &c.next;
            } else {
                break;
            }
        }
        println!(
            "----------------------------------------------------- Total: {}",
            self.len
        );
    }
}
