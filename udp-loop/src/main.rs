use udp_core::UdpClient;
use clap::Parser;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use rand::{RngCore, rngs::OsRng};
// use hex::encode;

/// 命令行参数解析
#[derive(Parser, Debug)]
#[command(name = "udp-loop")]
#[command(about = "UDP 回环测试工具", long_about = None)]
struct Args {
    /// 目标地址 (如 127.0.0.1:12345)
    #[arg(short, long)]
    addr: String,

    /// 遇到校验错误时忽略继续发送
    #[arg(short, long, default_value_t = false)]
    ignore_errors: bool,

    /// 测试持续时间，单位秒（默认无限）
    #[arg(short, long)]
    duration: Option<u64>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let target: SocketAddr = args.addr.parse().expect("Invalid address format");

    let client = UdpClient::new()?;
    println!("本地绑定端口: {}", client.local_addr()?);

    let start_time = Instant::now();
    let mut total_sent = 0u64;
    let mut total_bytes = 0u64;
    let mut error_count = 0u64;

    loop {
        if let Some(duration) = args.duration {
            if start_time.elapsed() >= Duration::from_secs(duration) {
                break;
            }
        }

        // 生成 4KB 随机数据
        let mut data = vec![0u8; 4096];
        OsRng.fill_bytes(&mut data);

        // 发送并等待回复
        match client.send_and_receive(target, &data, Duration::from_secs(1)) {
            Ok(resp) => {
                if resp != data {
                    // println!("send:{} recv:{}", encode(&data), encode(&resp));
                    error_count += 1;
                    if !args.ignore_errors {
                        eprintln!("数据校验错误");
                        break;
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                if !args.ignore_errors {
                    eprintln!("发送/接收失败: {}", e);
                    break;
                }
            }
        }

        total_sent += 1;
        total_bytes += data.len() as u64;
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("=== 测试完成 ===");
    println!("总包数: {}", total_sent);
    println!("错误包数: {}", error_count);
    println!("总发送数据: {:.2} MB", total_bytes as f64 / 1024.0 / 1024.0);
    println!("平均带宽: {:.2} MB/s", total_bytes as f64 / 1024.0 / 1024.0 / elapsed);


    Ok(())
}
