use std::sync::mpsc;
use std::thread;

pub fn channel_demo() {

   let (tx, rx) = mpsc::channel();
   
   thread::scope(|s| {
       for i in 0..10 {
           let tx_clone = tx.clone();
           // 这里使用move关键字是为了将tx_clone的所有权转移到新线程中。
           // 在Rust中，线程的闭包默认只借用外部变量的引用，但如果要在新线程中使用外部变量的所有权，
           // 就需要用move将这些变量移动进闭包，这样新线程可以拥有这些变量的所有权或副本。
           // 这里每个线程都要拥有一个发送端(tx_clone)，所以需要move把它带进闭包里。
           s.spawn(move || {
               println!("Sending: {:?}-> {:?}", i, thread::current().id());
               tx_clone.send(format!("Hello from the thread! {:?}", i)).unwrap();
           });
       }
   });

drop(tx); // 关闭发送端，防止发送端阻塞等待接收端接收

for received in rx {
    println!("Received: {}", received);
}

println!("all done! ✅ ✅ ✅ ");
}