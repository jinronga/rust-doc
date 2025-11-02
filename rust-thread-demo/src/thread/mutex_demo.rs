use std::sync::{Arc, Mutex};
use std::thread;

pub fn mutex_demo() {

    let data = Arc::new(Mutex::new(vec![]));

    thread::scope(|s| {
        for i in 0..10 {
        
            let data_clone: Arc<Mutex<Vec<i32>>>    = Arc::clone(&data);
            // 使用 move 关键字可以将外部作用域中捕获的变量 data_clone 的所有权转移到新线程闭包里
            // 这样每个线程都能拥有 data_clone 的所有权副本，所以不会出现数据竞争。
            // 因为 data_clone 是 Arc，所以每个线程都能拥有 data_clone 的所有权副本，所以不会出现数据竞争。
            s.spawn(move || {
                let mut v = data_clone.lock().unwrap();
                v.push(i);
                println!("v: {:?}-> {:?}", v, thread::current().id());
            });
        }
    });
    println!("data: {:?}", data.lock().unwrap());
    println!("all done! ✅ ✅ ✅ ");
}