use udp_core::UdpServer;
use clap::Parser;
use std::time::Duration;
use std::thread;

/// 命令行参数解析
#[derive(Parser, Debug)]
#[command(name = "udp-echo-server")]
#[command(about = "UDP 回显服务器", long_about = None)]
struct Args {
    /// 监听端口
    #[arg(short, long, default_value_t = 12345)]
    port: u16,

    /// 是否启用延迟响应（用于测试 RTT）
    #[arg(long)]
    delay_ms: Option<u64>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let server = UdpServer::bind(args.port)?;
    println!("UDP Echo Server 已启动，监听端口 {}", args.port);

    let socket = server.socket.try_clone()?; // 为线程使用 clone 一个 socket 实例

    server.start_async(move |src_addr, data| {
        if let Some(delay) = args.delay_ms {
            thread::sleep(Duration::from_millis(delay)); // 模拟响应延迟
        }

        // 将收到的数据直接回传
        if let Err(e) = socket.send_to(data, src_addr) {
            eprintln!("发送回应失败: {}", e);
        }
    })?;

    // 主线程保持运行
    loop {
        thread::sleep(Duration::from_secs(10));
    }
}
