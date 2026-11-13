use thread_pool::JobStealer;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let cores = std::thread::available_parallelism().unwrap().get();
    println!("Booting up the Factory with {} Worker Threads...", cores);

    let pool = JobStealer::new(cores);
    let total_jobs = 1_000_000;
    let completed_tasks = Arc::new(AtomicUsize::new(0));

    println!("Submitting {} CPU-bound jobs to the Global Inbox...", total_jobs);

    let start_time = Instant::now();

    for id in 0..total_jobs {
        let counter_clone = completed_tasks.clone();

        pool.execute(move || {
            let mut _dummy = 0;
            for j in 0..50 {
                _dummy += (j * id) % 255;
            }
            std::hint::black_box(_dummy);
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });
    }

    let submission_time = start_time.elapsed();
    println!("Finished submitting all jobs in {:?}", submission_time);
    println!("Waiting for Worker Threads to drain the queues...");

    while completed_tasks.load(Ordering::Relaxed) < total_jobs {
        std::thread::yield_now();
    }

    let total_time = start_time.elapsed();
    let jobs_per_second = (total_jobs as f64) / total_time.as_secs_f64();

    println!("\n==================================================");
    println!("BENCHMARK COMPLETE");
    println!("Total Execution Time : {:?}", total_time);
    println!("Throughput           : {:.2} jobs / second", jobs_per_second);
    println!("==================================================\n");
}
