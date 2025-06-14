use udp_core::UdpClient;
use clap::Parser;
use std::net::SocketAddr;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use rand::{RngCore, rngs::OsRng};

#[derive(Parser, Clone, Debug)]
#[command(name = "udp-loop")]
#[command(about = "UDP 回环测试工具 (多端口多线程)", long_about = None)]
struct Args {
    #[arg(short, long)]
    addr: String,
    #[arg(short, long, default_value_t = false)]
    ignore_errors: bool,
    #[arg(short, long)]
    duration: Option<u64>,
    /// 并发线程数
    #[arg(short = 'n', long, default_value_t = 4)]
    threads: usize,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let target: SocketAddr = args.addr.parse().expect("Invalid address format");

    let start_time = Instant::now();
    let total_sent = Arc::new(AtomicU64::new(0));
    let total_bytes = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();

    for _ in 0..args.threads {
        let total_sent = total_sent.clone();
        let total_bytes = total_bytes.clone();
        let error_count = error_count.clone();
        let args = args.clone();
        let start_time = start_time.clone();
        let target = target.clone();

        handles.push(thread::spawn(move || {
            let client = UdpClient::new().expect("创建UDP socket失败");
            println!("线程绑定端口: {}", client.local_addr().unwrap());
            loop {
                if let Some(duration) = args.duration {
                    if start_time.elapsed() >= Duration::from_secs(duration) {
                        break;
                    }
                }

                let mut data = vec![0u8; 4096];
                OsRng.fill_bytes(&mut data);

                match client.send_and_receive(target, &data, Duration::from_secs(1)) {
                    Ok(resp) => {
                        if resp != data {
                            error_count.fetch_add(1, Ordering::Relaxed);
                            if !args.ignore_errors {
                                eprintln!("数据校验错误 (多端口多线程)");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error_count.fetch_add(1, Ordering::Relaxed);
                        if !args.ignore_errors {
                            eprintln!("发送/接收失败: {}", e);
                            break;
                        }
                    }
                }
                total_sent.fetch_add(1, Ordering::Relaxed);
                total_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
            }
        }));
    }

    for h in handles { let _ = h.join(); }

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("=== 测试完成 (多端口多线程) ===");
    println!("总包数: {}", total_sent.load(Ordering::Relaxed));
    println!("错误包数: {}", error_count.load(Ordering::Relaxed));
    println!("总发送数据: {:.2} MB", total_bytes.load(Ordering::Relaxed) as f64 / 1024.0 / 1024.0);
    println!("平均带宽: {:.2} MB/s", total_bytes.load(Ordering::Relaxed) as f64 / 1024.0 / 1024.0 / elapsed);

    Ok(())
}