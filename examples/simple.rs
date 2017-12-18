extern crate rand;
extern crate worker_sentinel;

use std::thread;
use std::time;
use rand::Rng;
use worker_sentinel::*;

struct SimpleWork {
    random_sleep: u64,
}
impl Work for SimpleWork {
    fn work(self, baton: Baton<Self>) -> Baton<Self> {
        println!("spawned & sleeping: {}ms", self.random_sleep);
        thread::sleep(time::Duration::from_millis(self.random_sleep));
        println!("exited");
        baton
    }
}

fn main() {
    spawn(3, || {
        let mut rng = rand::OsRng::new().unwrap();
        SimpleWork {
            random_sleep: rng.gen_range(1000, 3000),
        }
    });
    thread::park();
}
