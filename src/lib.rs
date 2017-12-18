use std::thread;
use std::sync::{atomic, Arc};

pub trait Work: Send + Sized {
    fn work(self, baton: Baton<Self>) -> Baton<Self>;
}

pub trait WorkFactory: Sync + Send {
    type Work: Work;
    fn build(&self) -> Self::Work;
}
impl<W: Work, F: Fn() -> W + Sync + Send> WorkFactory for F {
    type Work = W;
    fn build(&self) -> W {
        self()
    }
}

pub struct Baton<W>
where
    W: Work + 'static,
{
    work_factory: Arc<WorkFactory<Work = W>>,
    desired: usize,
    count: Arc<atomic::AtomicUsize>,
}

impl<W> Baton<W>
where
    W: Work + 'static,
{
    fn spawn(self) {
        let work = self.work_factory.build();
        thread::spawn(move || {
            work.work(self);
        });
    }
}

struct Sentinel<W>
where
    W: Work + 'static,
{
    work_factory: Arc<WorkFactory<Work = W>>,
    desired: usize,
    count: Arc<atomic::AtomicUsize>,
}
impl<W> Sentinel<W>
where
    W: Work + 'static,
{
    pub fn new<F: WorkFactory<Work = W> + 'static>(desired: usize, work_factory: F) -> Self {
        let count = Arc::new(atomic::AtomicUsize::new(0));
        let arc_work_factory = Arc::new(work_factory);
        Sentinel { work_factory: arc_work_factory, desired, count }
    }

    fn balance(&self) {
        loop {
            let curr = self.count.load(atomic::Ordering::SeqCst);
            if curr >= self.desired {
                break;
            }
            let prev = self.count.compare_and_swap(curr, curr + 1, atomic::Ordering::SeqCst);
            if prev != curr {
                continue;
            }
            let work_factory = self.work_factory.clone();
            let desired = self.desired;
            let count = self.count.clone();
            let baton = Baton { work_factory, desired, count };
            baton.spawn();
        }
    }
}
impl<W> Drop for Baton<W>
where
    W: Work + 'static,
{
    fn drop(&mut self) {
        self.count.fetch_sub(1, atomic::Ordering::SeqCst);

        let work_factory = self.work_factory.clone();
        let desired = self.desired;
        let count = self.count.clone();
        let sentinel = Sentinel { work_factory, desired, count };
        sentinel.balance();
    }
}

pub fn spawn<F>(desired: usize, work_factory: F)
where
    F: WorkFactory + 'static
{
    Sentinel::new(desired, work_factory).balance();
}
