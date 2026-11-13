use crossbeam_deque::{Injector, Stealer, Worker, Steal};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Condvar;
use std::thread;
use std::thread::JoinHandle;


// The Type Alias for payload
pub type Job = Box<dyn FnOnce() + Send + 'static>;

pub(crate) struct SleepSignal {
    // The lock required to use the Condvar. 
    // It can also hold a boolean to signal when the engine is shutting down.
    is_active: Mutex<bool>,

    // The actual loudspeaker to wake sleeping threads
    cvar: Condvar,
}

pub struct JobStealer {
    // The Global Inbox. Wrapped in an Arc so workers can check it.
    global_queue: Arc<Injector<Job>>,

    // The Loudspeaker. Wrapped in an Arc so the Main Thread can yell into it.
    sleep_signal: Arc<SleepSignal>,

    // The OS Thread Handles. Required so the Main Thread can safely 
    // join/close them when the program shuts down.
    workers: Vec<JoinHandle<()>>,
}

impl JobStealer {
    pub fn new(num_threads: usize) -> Self {
        let sleep_signal = Arc::new(
            SleepSignal{
                is_active: Mutex::new(true),
                cvar: Condvar::new()
            }
        );
        let global_deque = Arc::new(Injector::new());

        let mut stealers = Vec::new();
        let mut workers = Vec::new();

        for _ in 0..num_threads {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers.push(worker);
        }

        let mut worker_handle = Vec::new();

        for i in 1..num_threads {
            let worker_con = WorkerContext{
                id: i,
                local_deque: workers.pop().unwrap(),
                stealers: stealers.clone(),
                global_deque: global_deque.clone(),
                sleep_signal: sleep_signal.clone()
            };

            let handle = thread::spawn( move || {
                worker_con.run();
            });

            worker_handle.push(handle);
        }

        JobStealer {
            global_queue: global_deque,
            sleep_signal: sleep_signal,
            workers: worker_handle
        }
    }

    pub fn execute<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        let payload = Box::new(job);
        self.global_queue.push(payload);
        let _lock = self.sleep_signal.is_active.lock().unwrap();
        self.sleep_signal.cvar.notify_one();
    }
}

impl Drop for JobStealer {
        fn drop(&mut self){
            {
                let mut keep_running = self.sleep_signal.is_active.lock().unwrap();
                *keep_running = false;
                self.sleep_signal.cvar.notify_all();
            }

            // 4. Wait for threads to pack their desks
        // (Use take() to pull the handles out of the struct)
            let handles = std::mem::take(&mut self.workers);
            for worker in handles{
                worker.join().unwrap();
            }
        }
    }


struct WorkerContext {
    id: usize,
    local_deque: Worker<Job>,
    stealers: Vec<Stealer<Job>>,
    global_deque: Arc<Injector<Job>>,
    sleep_signal: Arc<SleepSignal>,
}

impl WorkerContext {
    fn run(self){
        loop {
            if let Some(job) = self.local_deque.pop(){
                job();
                continue;
            }
            if let Steal::Success(job) = self.global_deque.steal_batch_and_pop(&self.local_deque) {
                job();
                continue;
            }

            let mut steal_job = false;
            for stealer in &self.stealers {
                if let Steal::Success(job) = stealer.steal_batch_and_pop(&self.local_deque){
                    steal_job = true;
                    job();
                    break;
                }
            }

            if steal_job {
                continue;
            }

            let mut keep_running_guard = self.sleep_signal.is_active.lock().unwrap();
            while *keep_running_guard && self.global_deque.is_empty(){
                keep_running_guard = self.sleep_signal.cvar.wait(keep_running_guard).unwrap();
            }

            if !*keep_running_guard{
                break;
            }

        }
    }
}