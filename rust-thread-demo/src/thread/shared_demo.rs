use std::cell::OnceCell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc, Condvar, Mutex, OnceLock, RwLock};
use std::thread;
use std::time::Duration;

// # 多线程数据共享方式对比
//
// | 共享方式             | 是否可变  | 是否加锁   | 是否线程安全 | 特点            |
// | ---------------- | ----- | ------ | ------ | ------------- |
// | `Arc<T>`         | 否     | 否      | ✅      | 多线程共享只读数据     |
// | `Arc<Mutex<T>>`  | 是     | 是（互斥）  | ✅      | 多线程共享可变数据（写多） |
// | `Arc<RwLock<T>>` | 是     | 是（读写锁） | ✅      | 读多写少的场景       |
// | `mpsc::channel`  | 否（传值） | 否      | ✅      | 通过消息传递共享数据    |
// | `Atomic*` 类型     | 是     | 否      | ✅      | 无锁高性能操作基础类型   |

// 多个线程读取同一份数据 需要使用 Arc 和 Mutex 来保证数据的安全
pub fn shared_demo() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));

    let mut handles = vec![];

    for i in 0..3 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            println!("Thread {}: {:?}", i, data.lock().unwrap());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("all done! ✅ ✅ ✅ ");
}


// 多线程累加计数器 
pub fn counter_demo() {
    let counter = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut count = counter.lock().unwrap();
            *count += 1;
            println!("counter = {}", *count);
        });
        handles.push(handle);   // 将 handle 添加到 handles 向量中，以便在最后等待所有线程完成  
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("counter = {}", counter.lock().unwrap());
    println!("all done! ✅ ✅ ✅ ");
}

// 多线程同时读数据、偶尔写
// read_write_demo 演示多线程下如何用 RwLock 实现“多读少写”场景下的数据安全共享。
// RwLock（读写锁）允许多个线程同时读数据，但写数据时只能有一个线程且不能有其他线程在读。
pub fn read_write_demo() {
    // 用 Arc<RwLock<T>> 包装数据以支持多线程安全共享，这里数据初始值为 5。
    let data = Arc::new(RwLock::new(5));

    // 1. 读线程示例
    // 克隆一份 Arc，为读线程准备数据引用。
    let data_r = Arc::clone(&data);
    // 启动一个线程，尝试读数据。读的时候会获得一个读锁。
    let reader_handle = thread::spawn(move || {
        let data = data_r.read().unwrap(); // 获取读锁
        println!("reader data = {}", *data); // 打印读到的内容
    });

    // 2. 写线程示例
    // 再克隆一份 Arc，为写线程准备数据引用。
    let data_w = Arc::clone(&data);
    // 启动一个线程，尝试写数据。写的时候会获得一个写锁，此时不能被其他线程读或写。
    let writer_handle = thread::spawn(move || {
        let mut data = data_w.write().unwrap(); // 获取写锁
        *data += 1; // 修改数据内容
        println!("writer data = {}", *data); // 打印新内容
    });

    // 等待两个线程都结束
    reader_handle.join().unwrap();
    writer_handle.join().unwrap();

    // 主线程打印最终的内容（此时会获得一个读锁）
    println!("Final value: {}", *data.read().unwrap());

    println!("all done! ✅ ✅ ✅ ");
}


// 多生产者单消费者通道
// mpsc（Multiple Producer, Single Consumer）是 Rust 标准库提供的一种多生产者单消费者的消息传递机制。
// 它允许多个线程同时发送消息，但只能有一个线程接收消息。
// 适用于需要多个线程同时发送消息，但只需要一个线程接收消息的场景。
pub  fn mpsc_demo() {
    let (tx, rx) = mpsc::channel();

    for i in 0..10 {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            println!("sent message {:?}-> {:?}", i, thread::current().id());
            tx_clone.send(i).unwrap();
        });
    }

    drop(tx); // 关闭发送端，防止发送端阻塞等待接收端接收
    // 等待所有生产者线程完成

    for received in rx {
        println!("mspc received message {:?}-> {:?}", received, thread::current().id());
    }

    println!("all done! ✅ ✅ ✅ ");
}

// 原子计数器 
pub fn atomic_counter_demo() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    for _ in 0..10 {
        let counter_clone   = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let current = counter_clone.load(Ordering::SeqCst);
            println!("atomic counter = {}", current);
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("atomic counter = {}", counter.load(Ordering::SeqCst));
 
    println!("all done! ✅ ✅ ✅ ");
}

// Rust 的原子类型（Atomic*）在某些场景下确实可能出现 ABA 问题 
// 这是因为原子操作的“比较并交换”（CAS）机制，它要求在操作前后内存中的值必须保持一致。
// 如果一个值在操作前后被其他线程修改了，那么CAS操作就会失败。
// 为了解决这个问题，Rust 提供了一些原子类型，它们可以保证在操作前后内存中的值保持一致。
// 例如，AtomicUsize 类型可以保证在操作前后内存中的值保持一致。
// 例如，AtomicUsize 类型可以保证在操作前后内存中的值保持一致。

// ABA 问题演示：演示原子操作中的 ABA 问题
// ABA 问题是指一个值从 A 变为 B，又变回 A，但 CAS 操作仍然会成功
pub fn atomic_aba_demo() {
    let val = Arc::new(AtomicUsize::new(1));
    let v1 = Arc::clone(&val);
    let v2 = Arc::clone(&val);

    // 线程 1 读取到 1
    let t1 = thread::spawn(move || {
        let old = v1.load(Ordering::SeqCst);
        thread::sleep(std::time::Duration::from_millis(100));
        // 此时另一个线程可能已将值改为 2 又改回 1
        // 使用 compare_exchange 替代已废弃的 compare_and_swap
        match v1.compare_exchange(old, 999, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => println!("Thread 1: CAS succeeded (but value may have changed A->B->A)"),
            Err(_) => println!("Thread 1: CAS failed"),
        }
    });

    // 线程 2 模拟 ABA 变化：1 -> 2 -> 1
    let t2 = thread::spawn(move || {
        v2.store(2, Ordering::SeqCst);
        thread::sleep(std::time::Duration::from_millis(50));
        v2.store(1, Ordering::SeqCst);
        println!("Thread 2: Changed value 1 -> 2 -> 1 (ABA pattern)");
    });

    t1.join().unwrap();
    t2.join().unwrap();

    println!("Final value = {}", val.load(Ordering::SeqCst));
    println!("all done! ✅ ✅ ✅ ");
}

// OnceCell 是 Rust 标准库提供的一种线程安全的单次初始化机制。
// 它允许多个线程同时访问一个单元格，但只能有一个线程初始化它。
// 适用于需要单次初始化一个值的场景。
pub fn once_cell_demo() {
    let cell = OnceCell::new();

    cell.set(42).unwrap(); // ✅ 首次设置成功
    assert_eq!(cell.get(), Some(&42));

    // cell.set(99).unwrap(); // ❌ 再次设置会 panic
    println!("once cell = {}", cell.get().unwrap());
    println!("all done! ✅ ✅ ✅ ");
}


// OnceLock 是 Rust 标准库提供的一种线程安全的单次初始化机制。
// 它允许多个线程同时访问一个单元格，但只能有一个线程初始化它。
// 适用于需要单次初始化一个值的场景。
pub fn once_lock_demo() {
    static ONCE_LOCK: OnceLock<usize> = OnceLock::new();

    let handle1 = thread::spawn(|| {
        // 第一个线程初始化
        ONCE_LOCK.set(42).unwrap_or_else(|_| {
            println!("Thread 1: Already initialized");
        });
    });

    let handle2 = thread::spawn(|| {
        // 第二个线程尝试初始化（会失败，因为已经被初始化）
        ONCE_LOCK.set(100).unwrap_or_else(|_| {
            println!("Thread 2: Already initialized, cannot set");
        });
    });

    // `join()` 会阻塞当前线程直到子线程执行完毕，
    // `unwrap()` 用于处理 join 返回的 Result（如果线程出现 panic 会导致 Err）。
    // 该调用确保 handle1 线程已经运行结束，并在出现错误时直接 panic。
    handle1.join().unwrap();
    handle2.join().unwrap();

    // 所有线程都可以读取
    println!("Value: {}", ONCE_LOCK.get().unwrap());
    println!("all done! ✅ ✅ ✅ ");
}

// Lazycell 是 Rust 标准库提供的一种线程安全的延迟初始化机制。
// 它允许多个线程同时访问一个单元格，但只能有一个线程初始化它。
// 适用于需要延迟初始化一个值的场景。
pub fn lazy_cell_demo() {
    let lazy_cell = std::cell::LazyCell::new(|| {
        println!("LazyCell initialized");
        42
    });
    println!("lazy cell = {}", *lazy_cell);
    println!("all done! ✅ ✅ ✅ ");
}

// LazyLock 是 Rust 标准库提供的一种线程安全的延迟初始化机制。
// 它允许多个线程同时访问一个单元格，但只能有一个线程初始化它。
// 适用于需要延迟初始化一个值的场景。
#[allow(dead_code)]
pub fn lazy_lock_demo() {
    let lazy_lock = std::sync::LazyLock::new(|| {
        println!("LazyLock initialized");
        42
    });
    println!("lazy lock = {}", *lazy_lock);
    println!("all done! ✅ ✅ ✅ ");
}

// ThreadLocal 演示：每个线程都有自己独立的数据副本
pub fn thread_local_demo() {
    thread_local! {
        static COUNTER: std::cell::RefCell<usize> = std::cell::RefCell::new(0);
    }

    let mut handles = vec![];

    for i in 0..3 {
        let handle = thread::spawn(move || {
            // 每个线程都有自己的 COUNTER
            COUNTER.with(|c| {
                *c.borrow_mut() += i;
                println!("Thread {:?}: counter = {}", thread::current().id(), c.borrow());
            });
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // 主线程也有自己的 COUNTER
    COUNTER.with(|c| {
        println!("Main thread counter = {}", c.borrow());
    });

    println!("all done! ✅ ✅ ✅ ");
}

// Condvar 演示：条件变量，用于线程间协调
// 通常与 Mutex 一起使用，实现等待/通知机制
pub fn condvar_demo() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair_clone = Arc::clone(&pair);

    // 工作线程：等待条件满足
    let handle = thread::spawn(move || {
        let (lock, cvar) = &*pair_clone;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
        println!("Thread: Condition met, starting work!");
    });

    // 主线程：模拟一些工作后通知
    thread::sleep(std::time::Duration::from_millis(100));
    {
        let (lock, cvar) = &*pair;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
        println!("Main: Notifying waiting thread");
    }

    handle.join().unwrap();
    println!("all done! ✅ ✅ ✅ ");
}

// Rayon 是一个并行计算库，提供简单易用的并行迭代器
// 注意：需要在 Cargo.toml 中添加 rayon 依赖
pub fn rayon_crate_demo() {
    // 这个函数需要 rayon crate
    // 使用示例（如果已添加依赖）:
    /*
    use rayon::prelude::*;
    
    let vec = (0..1000).collect::<Vec<_>>();
    let sum: usize = vec.par_iter().sum();
    println!("Sum: {}", sum);
    */
    println!("Rayon demo requires 'rayon' crate in Cargo.toml");
    println!("Add to Cargo.toml: rayon = \"1.8\"");
    println!("all done! ✅ ✅ ✅ ");
}


// thread parking 演示：线程挂起和唤醒
pub fn thread_parking_demo() {
  
    let t = thread::spawn(|| {
        println!("子线程开始执行...");
        thread::park(); // 暂停自己
        println!("子线程被唤醒，继续执行...");
    });

    thread::sleep(Duration::from_secs(2));
    println!("主线程唤醒子线程...");
    t.thread().unpark(); // 唤醒
    t.join().unwrap();
    println!("all done! ✅ ✅ ✅ ");
}



// 带超时的 park 演示
pub fn thread_parking_demo_with_timeout() {
    let handle = std::thread::spawn(move || {
        println!("等待 2 秒或被唤醒...");
        std::thread::park_timeout(std::time::Duration::from_secs(2));
        println!("继续执行");
    });

    std::thread::sleep(std::time::Duration::from_secs(1));
    handle.thread().unpark(); // 如果注释掉这行，2 秒后自动恢复
    handle.join().unwrap();
    println!("all done! ✅ ✅ ✅ ");
}

pub fn thread_parking_demo_with_timeout_and_result() {
    
        let tasks = Arc::new(Mutex::new(vec![]));
        let worker = {
            let tasks = Arc::clone(&tasks);
            thread::spawn(move || {
                loop {
                    let task_opt = {
                        let mut tasks = tasks.lock().unwrap();
                        tasks.pop()
                    };
                    if let Some(task) = task_opt {
                        println!("执行任务: {}", task);
                    } else {
                        println!("无任务，线程休眠...");
                        thread::park(); // 等待唤醒
                    }
                }
            })
        };
    
        thread::sleep(Duration::from_secs(2));
        {
            let mut tasks = tasks.lock().unwrap();
            tasks.push("任务A".to_string());
        }
        println!("主线程添加任务并唤醒工作线程");
        worker.thread().unpark();
    
        thread::sleep(Duration::from_secs(1));
        println!("主线程结束");
    }   