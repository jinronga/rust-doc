# Tokio 学习指南 - 第二阶段：从简单示例开始

> 本指南面向有 Go/Java/Python 背景的 Rust 初学者

## 📚 学习目标

通过 4 个渐进式示例，掌握 Tokio 异步编程的核心概念：
- `async/await` 语法
- 异步网络编程
- 并发任务处理
- Stream/Sink 模式

## 🗺️ 学习路径

### 示例 1：`hello_world.rs` - 最简单的异步客户端 ⭐

**预计时间：30分钟**

#### 学习目标
- 理解 `#[tokio::main]` 宏的作用
- 理解 `async fn` 和 `.await` 语法
- 理解异步网络连接

#### 代码解析

```rust
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    // 连接到服务器
    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;
    
    // 写入数据
    stream.write_all(b"hello world\n").await?;
    
    Ok(())
}
```

#### 关键概念

| 概念 | 说明 | Go 对比 |
|------|------|---------|
| `#[tokio::main]` | 创建异步运行时入口 | Go 的 `main()` 函数 |
| `async fn` | 异步函数，返回 `Future` | Go 的普通函数（goroutine 在内部） |
| `.await` | 等待异步操作完成 | Go 的阻塞调用（但不阻塞线程） |
| `?` 操作符 | 错误传播 | Go 的 `if err != nil { return err }` |

#### 运行步骤

```bash
# 终端1：启动一个简单的服务器
ncat -l 6142
# 或者使用 nc
nc -l 6142

# 终端2：运行客户端
cargo run --example hello_world
```

#### 实践练习

1. **修改消息内容**
   ```rust
   stream.write_all(b"你好，世界！\n").await?;
   ```

2. **添加错误处理**
   ```rust
   match stream.write_all(b"hello world\n").await {
       Ok(_) => println!("写入成功"),
       Err(e) => eprintln!("写入失败: {}", e),
   }
   ```

3. **读取服务器响应**（需要服务器支持）
   ```rust
   use tokio::io::AsyncReadExt;
   
   let mut buf = [0; 1024];
   let n = stream.read(&mut buf).await?;
   println!("收到响应: {}", String::from_utf8_lossy(&buf[..n]));
   ```

#### 检查清单
- [ ] 理解 `#[tokio::main]` 的作用
- [ ] 理解 `.await` 不会阻塞线程
- [ ] 能运行并看到输出
- [ ] 能修改代码并理解结果

---

### 示例 2：`echo-tcp.rs` - TCP 回显服务器 ⭐⭐

**预计时间：1-2小时**

#### 学习目标
- 理解 `TcpListener` 和 `accept()`
- 理解 `tokio::spawn` 并发处理
- 理解异步 I/O 循环

#### 代码解析

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 绑定地址并监听
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    loop {
        // 接受新连接
        let (mut socket, _) = listener.accept().await?;
        
        // 为每个连接创建新任务
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            
            loop {
                // 读取数据
                let n = socket.read(&mut buf).await?;
                
                if n == 0 {
                    return; // 连接关闭
                }
                
                // 回显数据
                socket.write_all(&buf[0..n]).await?;
            }
        });
    }
}
```

#### 关键概念

| 概念 | 说明 | Go 对比 |
|------|------|---------|
| `TcpListener` | TCP 服务器监听器 | `net.Listen()` |
| `accept().await` | 异步接受连接 | `listener.Accept()`（阻塞） |
| `tokio::spawn` | 创建并发任务 | `go func()` |
| `async move` | 移动所有权到任务 | Go 自动处理 |

#### 数据流图

```
客户端连接
    ↓
listener.accept().await  ← 等待新连接（不阻塞线程）
    ↓
tokio::spawn()  ← 创建新任务（类似 goroutine）
    ↓
socket.read().await  ← 等待数据
    ↓
socket.write_all().await  ← 回显数据
    ↓
继续循环或返回（连接关闭）
```

#### 运行步骤

```bash
# 终端1：启动服务器
cargo run --example echo-tcp

# 终端2：使用客户端测试
cargo run --example connect-tcp 127.0.0.1:8080

# 或者使用 telnet
telnet 127.0.0.1 8080

# 或者使用 nc
echo "hello" | nc 127.0.0.1 8080
```

#### 实践练习

1. **添加连接日志**
   ```rust
   let (mut socket, addr) = listener.accept().await?;
   println!("新连接来自: {}", addr);
   ```

2. **添加数据日志**
   ```rust
   let data_str = String::from_utf8_lossy(&buf[0..n]);
   println!("收到数据: {}", data_str);
   ```

3. **统计连接数**
   ```rust
   use std::sync::atomic::{AtomicU64, Ordering};
   
   let connection_count = AtomicU64::new(0);
   
   loop {
       let (mut socket, _) = listener.accept().await?;
       let count = connection_count.fetch_add(1, Ordering::Relaxed);
       println!("总连接数: {}", count + 1);
       // ...
   }
   ```

#### 检查清单
- [ ] 理解服务器监听和接受连接的流程
- [ ] 理解 `tokio::spawn` 如何实现并发
- [ ] 能同时处理多个客户端连接
- [ ] 理解 `n == 0` 表示连接关闭

---

### 示例 3：`connect-tcp.rs` - TCP 客户端（Stream/Sink） ⭐⭐⭐

**预计时间：2-3小时**

#### 学习目标
- 理解 `FramedRead` 和 `FramedWrite`
- 理解 Stream/Sink 模式
- 理解双向数据传输

#### 代码解析

```rust
// 将 stdin/stdout 转换为 Stream/Sink
let stdin = FramedRead::new(stdin(), BytesCodec::new());
let stdin = stdin.map(|i| i.map(|bytes| bytes.freeze()));
let stdout = FramedWrite::new(stdout(), BytesCodec::new());

// 连接服务器
let mut stream = TcpStream::connect(addr).await?;
let (r, w) = stream.split();  // 分离读写

// 创建 Stream/Sink
let mut sink = FramedWrite::new(w, BytesCodec::new());
let mut stream = FramedRead::new(r, BytesCodec::new())
    .filter_map(|i| match i {
        Ok(i) => future::ready(Some(i.freeze())),
        Err(e) => {
            println!("错误: {}", e);
            future::ready(None)
        }
    })
    .map(Ok);

// 双向转发：stdin → socket, socket → stdout
future::join(
    sink.send_all(&mut stdin),
    stdout.send_all(&mut stream)
).await?;
```

#### 关键概念

| 概念 | 说明 | Go 对比 |
|------|------|---------|
| `FramedRead` | 将 `AsyncRead` 转为 `Stream` | `bufio.NewReader` |
| `FramedWrite` | 将 `AsyncWrite` 转为 `Sink` | `bufio.NewWriter` |
| `stream.split()` | 分离读写部分 | Go 的 Conn 天然支持 |
| `send_all()` | 转发整个 Stream | `io.Copy` |
| `future::join` | 并发执行多个 Future | 多个 goroutine |

#### 数据流图

```
stdin (用户输入)
    ↓
FramedRead → Stream<Bytes>
    ↓
sink.send_all() → TcpStream (写)
    
TcpStream (读)
    ↓
FramedRead → Stream<Bytes>
    ↓
stdout.send_all() → stdout (输出)
```

#### 运行步骤

```bash
# 终端1：启动 echo-tcp 服务器
cargo run --example echo-tcp

# 终端2：运行客户端
cargo run --example connect-tcp 127.0.0.1:8080

# 在客户端终端输入文字，应该会回显
```

#### 实践练习

1. **添加传输日志**
   ```rust
   let mut stream = FramedRead::new(r, BytesCodec::new())
       .inspect(|result| {
           if let Ok(bytes) = result {
               println!("收到 {} 字节", bytes.len());
           }
       })
       // ...
   ```

2. **添加超时处理**
   ```rust
   use tokio::time::{timeout, Duration};
   
   timeout(
       Duration::from_secs(30),
       future::join(sink.send_all(&mut stdin), stdout.send_all(&mut stream))
   ).await??;
   ```

#### 检查清单
- [ ] 理解 Stream 是异步版本的 Iterator
- [ ] 理解 Sink 是异步版本的写入器
- [ ] 理解 `FramedRead/FramedWrite` 的作用
- [ ] 理解双向数据传输的实现

---

### 示例 4：`connect-udp.rs` - UDP 客户端 ⭐⭐

**预计时间：1-2小时**

#### 学习目标
- 理解 UDP 与 TCP 的区别
- 理解 UDP 的 `connect()` 方法
- 理解 `tokio::try_join` 的使用

#### 代码解析

```rust
// UDP 绑定和连接
let socket = UdpSocket::bind("0.0.0.0:0").await?;
socket.connect(addr).await?;  // 设置默认目标

// 并发执行发送和接收
tokio::try_join!(
    send(stdin, &socket),
    recv(stdout, &socket)
)?;

// 发送函数
async fn send(
    mut stdin: impl Stream<Item = Result<Bytes, Error>> + Unpin,
    writer: &UdpSocket,
) -> Result<(), Error> {
    while let Some(item) = stdin.next().await {
        let buf = item?;
        writer.send(&buf[..]).await?;  // 发送 UDP 数据包
    }
    Ok(())
}

// 接收函数
async fn recv(
    mut stdout: impl Sink<Bytes, Error = Error> + Unpin,
    reader: &UdpSocket,
) -> Result<(), Error> {
    loop {
        let mut buf = vec![0; 1024];
        let n = reader.recv(&mut buf[..]).await?;
        if n > 0 {
            stdout.send(Bytes::copy_from_slice(&buf[..n])).await?;
        }
    }
}
```

#### 关键概念

| 概念 | 说明 | Go 对比 |
|------|------|---------|
| `UdpSocket::bind()` | 绑定本地地址 | `net.ListenUDP()` |
| `socket.connect()` | 设置默认目标（可选） | `net.DialUDP()` |
| `send/recv` | 已连接 UDP 的发送/接收 | `conn.Write/Read` |
| `try_join!` | 并发执行，任一失败即返回 | 多个 goroutine + error group |

#### UDP vs TCP

| 特性 | TCP | UDP |
|------|-----|-----|
| 连接 | 需要建立连接 | 无连接（connect 可选） |
| 可靠性 | 可靠传输 | 不保证 |
| 顺序 | 保证顺序 | 不保证 |
| 速度 | 较慢 | 较快 |
| 使用场景 | 文件传输、HTTP | 游戏、视频流 |

#### 运行步骤

```bash
# 终端1：启动 UDP 服务器（使用 netcat）
nc -u -l 8080

# 终端2：运行客户端
cargo run --example connect-udp 127.0.0.1:8080
```

#### 实践练习

1. **添加数据包大小限制**
   ```rust
   const MAX_PACKET_SIZE: usize = 1400;  // 以太网 MTU
   
   if buf.len() > MAX_PACKET_SIZE {
       eprintln!("数据包太大: {} 字节", buf.len());
       continue;
   }
   ```

2. **添加数据包计数**
   ```rust
   let mut packet_count = 0u64;
   while let Some(item) = stdin.next().await {
       // ...
       packet_count += 1;
       if packet_count % 100 == 0 {
           println!("已发送 {} 个数据包", packet_count);
       }
   }
   ```

#### 检查清单
- [ ] 理解 UDP 与 TCP 的区别
- [ ] 理解 UDP 的 `connect()` 作用
- [ ] 理解 `try_join!` 与 `join!` 的区别
- [ ] 能同时处理发送和接收

---

## 📊 学习进度跟踪

### 总体进度

- [ ] 示例 1：`hello_world.rs` - 基础异步语法
- [ ] 示例 2：`echo-tcp.rs` - 服务器和并发
- [ ] 示例 3：`connect-tcp.rs` - Stream/Sink 模式
- [ ] 示例 4：`connect-udp.rs` - UDP 编程

### 核心概念掌握度

| 概念 | 理解程度 | 备注 |
|------|---------|------|
| `async/await` | ⬜ 未开始 / ⬛ 理解 / ✅ 掌握 | |
| `tokio::spawn` | ⬜ 未开始 / ⬛ 理解 / ✅ 掌握 | |
| Stream/Sink | ⬜ 未开始 / ⬛ 理解 / ✅ 掌握 | |
| 错误处理 | ⬜ 未开始 / ⬛ 理解 / ✅ 掌握 | |

## 🎯 学习建议

### 时间安排

- **第1天**：完成示例 1 和 2
- **第2-3天**：完成示例 3（重点理解 Stream/Sink）
- **第4天**：完成示例 4，复习所有概念

### 学习方法

1. **先运行，再理解**：先让代码跑起来，再逐行分析
2. **多修改，多实验**：尝试修改代码，观察结果
3. **对比学习**：与 Go/Java/Python 的实现对比
4. **画流程图**：理解数据流向和任务调度

### 常见问题

**Q: 为什么需要 `#[tokio::main]`？**
A: Rust 的 `main` 函数是同步的，需要运行时来执行异步代码。`#[tokio::main]` 宏会生成代码来创建和运行 Tokio 运行时。

**Q: `.await` 会阻塞线程吗？**
A: 不会。`.await` 会挂起当前任务，让线程去处理其他任务，实现高并发。

**Q: `tokio::spawn` 和 Go 的 `go` 有什么区别？**
A: 概念类似，但 Rust 需要明确的所有权管理。`tokio::spawn` 需要 `async move` 来转移所有权。

**Q: 什么时候用 `join!` 什么时候用 `try_join!`？**
A: `join!` 等待所有 Future 完成；`try_join!` 如果任一 Future 失败，立即返回错误。

**Q: Stream 和 Iterator 有什么区别？**
A: Iterator 是同步的，Stream 是异步的。Stream 的 `next()` 返回 `Future`，需要用 `.await`。

## 🔗 相关资源

- [Tokio 官方教程](https://tokio.rs/tokio/tutorial)
- [Tokio API 文档](https://docs.rs/tokio)
- [Rust 异步编程指南](https://rust-lang.github.io/async-book/)
- [Futures 文档](https://docs.rs/futures)

## 📝 下一步

完成第二阶段后，可以继续学习：
- 第三阶段：理解核心概念（runtime、reactor 等）
- 第四阶段：进阶示例（chat.rs、tinyhttp.rs 等）
- 实践项目：实现自己的异步应用

