
use std::thread;

pub fn scope_thread() {

    let v = vec![1, 2, 3, 4, 5];

    thread::scope(|s| {
        s.spawn(move || {
            println!("Hello from the thread! {:?}-> {:?}", v, thread::current().id());
        });
        println!("Hello from the main thread! {:?}", thread::current().id());
    });

   println!("all done! ✅ ✅ ✅ ");

}