use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use std::io;
use std::sync::mpsc;
use std::thread;

const PORT_RANGE_START: u16 = 58052;
const PORT_RANGE_END: u16 = 58080;

pub struct UdpClient {
    pub socket: UdpSocket,
}

impl UdpClient {
    // 创建新的UDP客户端，绑定到随机可用端口（在指定范围内）
    pub fn new() -> io::Result<Self> {
        for port in PORT_RANGE_START..=PORT_RANGE_END {
            match UdpSocket::bind(("0.0.0.0", port)) {
                Ok(socket) => {
                    return Ok(UdpClient { socket });
                }
                Err(_) => continue,
            }
        }
        Err(io::Error::new(io::ErrorKind::AddrNotAvailable, "No available ports in range"))
    }

    // 发送消息并等待回包，带超时
    pub fn send_and_receive(&self, addr: SocketAddr, msg: &[u8], timeout: Duration) -> io::Result<Vec<u8>> {
        self.socket.send_to(msg, addr)?;

        // 设置接收超时
        self.socket.set_read_timeout(Some(timeout))?;

        let mut buf = vec![0; 8192];
        let (num_bytes, _) = self.socket.recv_from(&mut buf)?;
        buf.truncate(num_bytes);
        Ok(buf)
    }

    // 只发送消息，不等待回包
    pub fn send_only(&self, addr: SocketAddr, msg: &[u8]) -> io::Result<()> {
        self.socket.send_to(msg, addr)?;
        Ok(())
    }

    // 获取客户端绑定的本地地址
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }
}

pub struct UdpServer {
    pub socket: UdpSocket,
}

impl UdpServer {
    // 创建新的UDP服务器，监听指定端口
    pub fn bind(port: u16) -> io::Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", port))?;
        Ok(UdpServer { socket })
    }

    // 异步启动服务器，接收消息并通过回调处理
    pub fn start_async<F>(&self, callback: F) -> io::Result<()>
    where
        F: FnMut(SocketAddr, &[u8]) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let socket_clone = self.socket.try_clone()?;

        // 启动接收线程
        thread::spawn(move || {
            let mut buf = vec![0; 8192];
            loop {
                match socket_clone.recv_from(&mut buf) {
                    Ok((num_bytes, src_addr)) => {
                        // println!("Received {} bytes from {}", num_bytes, src_addr);
                        let data = &buf[..num_bytes];
                        tx.send((src_addr, data.to_vec())).unwrap();
                    }
                    Err(e) => {
                        eprintln!("Error receiving data: {}", e);
                        break;
                    }
                }
            }
        });

        // 启动处理线程
        thread::spawn(move || {
            let mut callback = callback;
            for (src_addr, data) in rx {
                callback(src_addr, &data);
            }
        });

        Ok(())
    }
}

// 示例使用
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::sync::{Arc, Mutex}; // 只在测试模块中导入需要的类型
    
    #[test]
    fn test_send_and_receive() {
        let server = UdpServer::bind(12345).unwrap();
        let client = UdpClient::new().unwrap();

        //克隆服务器socket一个实例，用于发送响应
        let server_socket = server.socket.try_clone().unwrap();

        // 启动服务器线程，这里使用闭包语法，线程会获取所有权，此时move强制闭包获取线程所有权
        let handle = thread::spawn(move || {
            server.start_async(move |src_addr, data| {
                let resp = format!("Echo: {}", String::from_utf8_lossy(data));
                // 这里创建了一个随机的实例给客户端回复数据，端口会随机
                // let server = UdpServer::bind(0).unwrap();
                server_socket.send_to(resp.as_bytes(), src_addr).unwrap();
                // server.socket.send_to(resp.as_bytes(), src_addr).unwrap();
            }).unwrap();

            // 保持线程运行一段时间
            thread::sleep(Duration::from_secs(2));
        });

        // 客户端发送消息并接收回复
        thread::sleep(Duration::from_millis(500));
        let response = client.send_and_receive("127.0.0.1:12345".parse().unwrap(),
                                               b"Hello, server!",
                                               Duration::from_secs(1)).unwrap();

        assert!(response.starts_with(b"Echo: "));

        handle.join().unwrap();
    }

    #[test]
    fn test_send_only() {
        let server = UdpServer::bind(12346).unwrap();
        let client = UdpClient::new().unwrap();

        // 使用 Arc 和 Mutex 共享状态
        let received = Arc::new(Mutex::new(false));
        let received_clone = Arc::clone(&received);

        // 启动服务器线程
        let handle = thread::spawn(move || {
            // 为回调创建一个单独的克隆
            let callback_received = Arc::clone(&received_clone);

            server.start_async(move |_src_addr, data| {
                assert_eq!(data, b"Fire and forget!");
                // 使用锁来修改共享状态
                *callback_received.lock().unwrap() = true;
            }).unwrap();

            // 保持线程运行一段时间，确保有足够时间接收消息
            thread::sleep(Duration::from_secs(2));

            // 检查是否接收到消息
            assert!(*received_clone.lock().unwrap(), "Message not received");
        });

        // 客户端发送消息
        client.send_only("127.0.0.1:12346".parse().unwrap(), b"Fire and forget!").unwrap();

        // 等待服务器线程完成
        handle.join().unwrap();
    }
}    