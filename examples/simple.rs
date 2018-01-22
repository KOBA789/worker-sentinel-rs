extern crate rand;
extern crate worker_sentinel;

use std::thread;
use std::time;
use rand::Rng;
use worker_sentinel::*;

struct SimpleWork
{
    random_sleep: u64,
    count: u64,
}
impl Work for SimpleWork
{
    fn work(mut self) -> Option<Self> {
        println!("spawned & sleeping: {}ms", self.random_sleep);
        thread::sleep(time::Duration::from_millis(self.random_sleep));
        println!("exited");
        self.count += 1;
        if self.count < 3 {
            Some(self)
        } else {
            None
        }
    }
}

struct Factory {
    thread: thread::Thread,
}
impl WorkFactory for Factory {
    type Work = SimpleWork;
    fn build(&self) -> Self::Work {
        let mut rng = rand::OsRng::new().unwrap();
        SimpleWork {
            random_sleep: rng.gen_range(1000, 3000),
            count: 0,
        }
    }
}
impl Drop for Factory {
    fn drop(&mut self) {
        self.thread.unpark();
    }
}

fn main() {
    spawn(3, || {
        let mut rng = rand::OsRng::new().unwrap();
        SimpleWork {
            random_sleep: rng.gen_range(1000, 3000),
            count: 0,
        }
    });
    thread::sleep(time::Duration::from_secs(9));

    spawn(3, Factory { thread: thread::current() });
    thread::park();
}
