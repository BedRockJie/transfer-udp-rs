// lib.rs
pub mod types;
pub mod utils;
pub mod layer1;
pub mod layer2;
pub mod layer3;

// 导出需要公开的类型和函数
pub use crate::layer1::{Layer1Protocol, FrameType, Priority, CheckType};
pub use crate::layer2::{Layer2Protocol, ReqRsp, DeviceType, RequestBodyType};
pub use crate::layer3::{Layer3Payload, ProtocolBody, RegisterProtocol, TlvProtocol};

use crate::types::ProtocolResult;

// 封包函数，从第三层开始封装到第一层
pub fn encapsulate_data(
    frame_type: FrameType,
    priority: Priority,
    check_type: CheckType,
    req_rsp: ReqRsp,
    device_type: DeviceType,
    device_index: u16,
    request_body_type: RequestBodyType,
    group: [u8; 8],
    address_or_command: u32,
    error_code: u16,
    payload: Vec<u8>,
) -> Vec<u8> {
    // 创建第三层协议数据
    let layer3_bytes = match request_body_type {
        RequestBodyType::RegisterProtocol => {
            let register_protocol = RegisterProtocol {
                register_address: address_or_command, // 实际使用时应设置正确的值
                error_code,
                data_length: payload.len() as u16,
                data: payload,
            };
            register_protocol.serialize()
        }
        RequestBodyType::TlvProtocol => {
            let tlv_protocol = TlvProtocol {
                command_code: address_or_command, // 实际使用时应设置正确的值
                error_code,
                data_length: payload.len() as u16,
                user_data: payload,
            };
            tlv_protocol.serialize()
        }
    };
    // Rust 不允许返回两个不同的类型过去，所以需要直接转化调用序列化，否则就要实现Layer3Payload的trait对象，可以支持灵活的变化。
    // let layer3 = match layer3_payload {
    //     ProtocolBody::Register(reg) => Layer3Payload::<RegisterProtocol>::new(reg),
    //     ProtocolBody::Tlv(tlv) => Layer3Payload::<TlvProtocol>::new(tlv),
    // };


    let layer2 = Layer2Protocol {
        req_rsp,
        is_need_reply: false, // 这里可根据实际情况初始化
        code: false, // 这里可根据实际情况初始化
        flag: false, // 这里可根据实际情况初始化
        request_body_type,
        device_type,
        device_index,
        group,
        payload: layer3_bytes,
    };
    let layer2_serialized = layer2.serialize();

    let layer1 = Layer1Protocol {
        frame_delimiter_0: 0x55, // 这里可根据实际情况初始化请求包的值
        frame_delimiter_1: 0xBB, // 这里可根据实际情况初始化请求包的值
        version: 1, // 这里可根据实际情况初始化
        priority,
        check_type,
        frame_type,
        frame_seq_number: 1, // 这里可根据实际情况初始化
        frame_length: 0, // 后续会计算填充
        payload: layer2_serialized,
        checksum: 0, // 后续会计算填充
    };
    layer1.serialize()
}

// 解包函数，从第一层开始解析到第三层
pub fn decapsulate_data(buf: &[u8]) -> Option<(Layer1Protocol, Layer2Protocol, ProtocolBody)> {
    // 解析第一层协议
    let layer1_result = Layer1Protocol::deserialize(buf);
    if layer1_result.is_err() {
        return None;
    }
    let layer1 = layer1_result.unwrap();

    // 解析第二层协议
    let layer2_result = Layer2Protocol::deserialize(&layer1.payload);
    if layer2_result.is_err() {
        return None;
    }
    let layer2 = layer2_result.unwrap();

    // 解析第三层协议
    let layer3_result: ProtocolResult<ProtocolBody>;
    // 这里比较关键，首先调用了接口本身的解析方法返回了当前的数据结构，然后又使用map将其转化为泛型 P
    match layer2.request_body_type {
        RequestBodyType::RegisterProtocol => {
            layer3_result = Layer3Payload::<RegisterProtocol>::deserialize(&layer2.payload)
                .map(|p| { ProtocolBody::Register(p.body)});
        }
        RequestBodyType::TlvProtocol => {
            layer3_result = Layer3Payload::<TlvProtocol>::deserialize(&layer2.payload)
                .map(|p| { ProtocolBody::Tlv(p.body)});
        }
    }
    if layer3_result.is_err() {
        return None;
    }
    let layer3 = layer3_result.unwrap();

    Some((layer1, layer2, layer3))
}


// 在lib.rs文件末尾添加以下测试模块

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::{FrameType, Priority, CheckType};
    use crate::layer2::{ReqRsp, DeviceType, RequestBodyType};
    // use crate::layer3::{RegisterProtocol, TlvProtocol};

    #[test]
    fn test_protocol_encapsulation_and_decapsulation_register() {
        // 准备测试数据
        let test_payload = vec![0x01, 0x02, 0x03, 0x04];
        let register_address = 0x12345678;
        let error_code = 0x0002;

        // 封装过程
        let serialized_data = encapsulate_data(
            FrameType::Type1,
            Priority::Medium,
            CheckType::CheckSum,
            ReqRsp::Request,
            DeviceType::MCU,
            10,
            RequestBodyType::RegisterProtocol,
            [0x01; 8],  // Group数据
            register_address,
            error_code,
            test_payload.clone(),
        );

        // 解封装过程
        let result = decapsulate_data(&serialized_data);
        assert!(result.is_some(), "解封装失败");

        let (layer1, layer2, layer3) = result.unwrap();

        // 验证第一层协议
        assert_eq!(layer1.frame_delimiter_0, 0x55);
        assert_eq!(layer1.frame_delimiter_1, 0xBB);
        assert_eq!(layer1.version, 1);
        assert_eq!(layer1.priority, Priority::Medium);
        assert_eq!(layer1.check_type, CheckType::CheckSum);
        assert_eq!(layer1.frame_type, FrameType::Type1);

        // 验证第二层协议
        assert_eq!(layer2.req_rsp, ReqRsp::Request);
        assert_eq!(layer2.device_type, DeviceType::MCU);
        assert_eq!(layer2.device_index, 10);
        assert_eq!(layer2.request_body_type, RequestBodyType::RegisterProtocol);
        assert_eq!(layer2.group, [0x01; 8]);

        // 验证第三层协议
        if let ProtocolBody::Register(reg) = layer3 {
            assert_eq!(reg.register_address, register_address);
            assert_eq!(reg.error_code, error_code);
            assert_eq!(reg.data, test_payload);
        } else {
            panic!("期望得到RegisterProtocol类型，但得到了其他类型");
        }
    }

    #[test]
    fn test_protocol_encapsulation_and_decapsulation_tlv() {
        // 准备测试数据
        let test_payload = vec![0x0A, 0x0B, 0x0C, 0x0D];
        let command_code = 0x87654321;
        let error_code = 0x0001;

        // 封装过程
        let serialized_data = encapsulate_data(
            FrameType::Type0,
            Priority::High,
            CheckType::CheckSum,
            ReqRsp::Response,
            DeviceType::FPGA,
            5,
            RequestBodyType::TlvProtocol,
            [0x02; 8],  // Group数据
            command_code,
            error_code,
            test_payload.clone(),
        );

        // 解封装过程
        let result = decapsulate_data(&serialized_data);
        assert!(result.is_some(), "解封装失败");

        let (layer1, layer2, layer3) = result.unwrap();

        // 验证第一层协议
        assert_eq!(layer1.frame_delimiter_0, 0x55);
        assert_eq!(layer1.frame_delimiter_1, 0xBB);
        assert_eq!(layer1.version, 1);
        assert_eq!(layer1.priority, Priority::High);
        assert_eq!(layer1.check_type, CheckType::CheckSum);
        assert_eq!(layer1.frame_type, FrameType::Type0);

        // 验证第二层协议
        assert_eq!(layer2.req_rsp, ReqRsp::Response);
        assert_eq!(layer2.device_type, DeviceType::FPGA);
        assert_eq!(layer2.device_index, 5);
        assert_eq!(layer2.request_body_type, RequestBodyType::TlvProtocol);
        assert_eq!(layer2.group, [0x02; 8]);

        // 验证第三层协议
        if let ProtocolBody::Tlv(tlv) = layer3 {
            assert_eq!(tlv.command_code, command_code);
            assert_eq!(tlv.error_code, error_code);
            assert_eq!(tlv.user_data, test_payload);
        } else {
            panic!("期望得到TlvProtocol类型，但得到了其他类型");
        }
    }

    #[test]
    fn test_invalid_data_decapsulation() {
        // 测试无效数据的解封装
        let invalid_data = vec![0x01, 0x02, 0x03, 0x04]; // 明显不是有效的协议数据
        let result = decapsulate_data(&invalid_data);
        assert!(result.is_none(), "对无效数据解封装应该失败");
    }
}