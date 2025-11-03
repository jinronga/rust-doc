mod thread;

fn main() {
    // thread::scope_thread::scope_thread();
    // thread::mutex_demo::mutex_demo();
    // thread::channel_demo::channel_demo();
    // thread::parallel_compute::parallel_compute();
    // thread::shared_demo::read_write_demo();
    // thread::shared_demo::mpsc_demo();
    // thread::shared_demo::atomic_counter_demo();
    // thread::shared_demo::atomic_aba_demo();
    // thread::shared_demo::once_cell_demo();
    // thread::shared_demo::lazy_lock_demo();
    // thread::shared_demo::thread_parking_demo();
    thread::shared_demo::thread_parking_demo_with_timeout_and_result();
}

