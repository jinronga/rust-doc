use std::thread;

pub fn parallel_compute() {
    let data = vec![10, 20, 30, 40, 50];

    let sum = thread::scope(|s| {
        let mut handles = vec![];

        // data.chunks(2) 表示把 data 按照 2 个一组进行切分，返回一个迭代器，每次迭代会得到一个长度最多为 2 的切片（slice）。
        // 例如 data = [10, 20, 30, 40, 50]，chunks(2) 会产生 [10, 20]、[30, 40]、[50] 三组。
        for chunk in data.chunks(2) {
            handles.push(s.spawn(move || chunk.iter().sum::<i32>()));
        }

        handles.into_iter().map(|h| h.join().unwrap()).sum::<i32>()
    });

    println!("sum = {}", sum);
    println!("all done! ✅ ✅ ✅ ");
}
