use std::thread;
use std::sync::{atomic, Arc};

pub trait Work: Send + Sized + 'static {
    fn work(self) -> Option<Self>;
}

pub trait WorkFactory: Sync + Send + Sized + 'static {
    type Work: Work + 'static;
    fn build(&self) -> Self::Work;
}
impl<W, F> WorkFactory for F
where
    W: Work,
    F: Fn() -> W + Sync + Send + 'static,
{
    type Work = W;
    fn build(&self) -> W {
        self()
    }
}

pub struct Baton<F>
where
    F: WorkFactory,
{
    sentinel: Arc<Sentinel<F>>,
}

struct Sentinel<F>
where
    F: WorkFactory,
{
    work_factory: F,
    desired: atomic::AtomicUsize,
    count: atomic::AtomicUsize,
}
impl<F> Sentinel<F>
where
    F: WorkFactory,
{
    fn new(desired: usize, work_factory: F) -> Self {
        let count = atomic::AtomicUsize::new(0);
        let desired = atomic::AtomicUsize::new(desired);
        Sentinel { work_factory, desired, count }
    }
}

fn balance<F>(sentinel: &Arc<Sentinel<F>>)
where
    F: WorkFactory,
{
    loop {
        let curr = sentinel.count.load(atomic::Ordering::SeqCst);
        if curr >= sentinel.desired.load(atomic::Ordering::Relaxed) {
            break;
        }
        let prev = sentinel.count.compare_and_swap(curr, curr + 1, atomic::Ordering::SeqCst);
        if prev != curr {
            continue;
        }
        let baton_sentinel = sentinel.clone();
        let work = sentinel.work_factory.build();
        thread::spawn(move || {
            let baton = Baton { sentinel: baton_sentinel };
            let mut work = work;
            loop {
                match work.work() {
                    Some(next_work) => work = next_work,
                    None => {
                        baton.sentinel.desired.store(0, atomic::Ordering::SeqCst);
                        return;
                    },
                }
            }
        });
    }
}

impl<F> Drop for Baton<F>
where
    F: WorkFactory,
{
    fn drop(&mut self) {
        self.sentinel.count.fetch_sub(1, atomic::Ordering::SeqCst);

        balance(&self.sentinel);
    }
}

pub fn spawn<F>(desired: usize, work_factory: F)
where
    F: WorkFactory
{
    let sentinel = Arc::new(Sentinel::new(desired, work_factory));
    balance(&sentinel);
}
