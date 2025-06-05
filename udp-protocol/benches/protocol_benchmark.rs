use criterion::{ criterion_main, criterion_group, Criterion};
use udp_protocol::{encapsulate_data, decapsulate_data};
use udp_protocol::{FrameType, Priority, CheckType, ReqRsp, DeviceType, RequestBodyType};

fn create_2k_payload() -> Vec<u8> {
    // 创建一个2KB的测试负载数据
    vec![0xAA; 2048]
}

fn bench_encapsulation(c: &mut Criterion) {
    let payload = create_2k_payload();

    c.bench_function("encapsulate_2k_data", |b| {
        b.iter(|| {
            encapsulate_data(
                FrameType::Type1,
                Priority::Medium,
                CheckType::CheckSum,
                ReqRsp::Request,
                DeviceType::MCU,
                10,
                RequestBodyType::RegisterProtocol,
                [0x01; 8],
                0x12345678,
                0x0002,
                payload.clone(),
            )
        })
    });
}

fn bench_decapsulation(c: &mut Criterion) {
    let payload = create_2k_payload();
    let serialized_data = encapsulate_data(
        FrameType::Type1,
        Priority::Medium,
        CheckType::CheckSum,
        ReqRsp::Request,
        DeviceType::MCU,
        10,
        RequestBodyType::RegisterProtocol,
        [0x01; 8],
        0x12345678,
        0x0002,
        payload,
    );

    c.bench_function("decapsulate_2k_data", |b| {
        b.iter(|| {
            decapsulate_data(&serialized_data)
        })
    });
}

criterion_group!(benches, bench_encapsulation, bench_decapsulation);
criterion_main!(benches);
