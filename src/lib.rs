use std::thread;
use std::sync::{atomic, Arc};

pub trait Work: Clone + Send {
    fn work(&mut self);
}

#[derive(Clone)]
struct Shared<W>
where
    W: Work + 'static,
{
    work: W,
    count: Arc<atomic::AtomicUsize>,
    desired: usize,
}

pub struct Sentinel<W>
where
    W: Work + 'static,
{
    shared: Shared<W>,
}
impl<W> Sentinel<W>
where
    W: Work + 'static,
{
    pub fn spawn(desired: usize, work: W) {
        let master = Sentinel {
            shared: Shared {
                work,
                desired,
                count: Arc::new(atomic::AtomicUsize::new(1)),
            },
        };
        master.balance();
    }

    fn balance(&self) {
        while self.shared.count.load(atomic::Ordering::SeqCst) < self.shared.desired {
            let mut copy = self.clone();
            thread::spawn(move || { copy.shared.work.work(); });
        }
    }
}
impl<W> Drop for Sentinel<W>
where
    W: Work,
{
    fn drop(&mut self) {
        self.shared.count.fetch_sub(1, atomic::Ordering::Relaxed);
        self.balance();
    }
}
impl<W> Clone for Sentinel<W>
where
    W: Work,
{
    fn clone(&self) -> Sentinel<W> {
        self.shared.count.fetch_add(1, atomic::Ordering::Relaxed);
        Sentinel { shared: self.shared.clone() }
    }
}
