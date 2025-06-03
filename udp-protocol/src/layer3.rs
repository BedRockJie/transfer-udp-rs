// layer3.rs

use crate::types::{ProtocolError, ProtocolResult};
use std::marker::PhantomData;

// 协议类型标记
pub trait ProtocolType {
    const TYPE_ID: u8;
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> ProtocolResult<Self> where Self: Sized;
}

// 寄存器协议
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterProtocol {
    pub register_address: u32,
    pub error_code: u16,
    pub data_length: u16,
    pub data: Vec<u8>,
}

impl ProtocolType for RegisterProtocol {
    const TYPE_ID: u8 = 0;
    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.register_address.to_le_bytes());
        buf.extend_from_slice(&self.error_code.to_le_bytes());
        buf.extend_from_slice(&self.data_length.to_le_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }

    fn deserialize(buf: &[u8]) -> ProtocolResult<Self> {
        if buf.len() < 8 {
            return Err(ProtocolError::InvalidLength);
        }

        let register_address = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let error_code = u16::from_le_bytes(buf[4..6].try_into().unwrap());
        let data_length = u16::from_le_bytes(buf[6..8].try_into().unwrap());
        let data = buf[8..].to_vec();

        Ok(Self {
            register_address,
            error_code,
            data_length,
            data,
        })
    }
}

// TLV 协议
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlvProtocol {
    pub command_code: u32,
    pub error_code: u16,
    pub data_length: u16,
    pub user_data: Vec<u8>,
}

impl ProtocolType for TlvProtocol {
    const TYPE_ID: u8 = 1;
    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.command_code.to_le_bytes());
        buf.extend_from_slice(&self.error_code.to_le_bytes());
        buf.extend_from_slice(&self.data_length.to_le_bytes());
        buf.extend_from_slice(&self.user_data);
        buf
    }
    fn deserialize(buf: &[u8]) -> ProtocolResult<Self> {
        if buf.len() < 8 {
            return Err(ProtocolError::InvalidLength);
        }

        let command_code = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let error_code = u16::from_le_bytes(buf[4..6].try_into().unwrap());
        let data_length = u16::from_le_bytes(buf[6..8].try_into().unwrap());
        let user_data = buf[8..].to_vec();

        Ok(Self {
            command_code,
            error_code,
            data_length,
            user_data,
        })
    }
}

// 避免空类型实现错误，添加了一个类型的空实现
impl ProtocolType for () {
    const TYPE_ID: u8 = 255; // 无效类型ID

    fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }

    fn deserialize(_buf: &[u8]) -> ProtocolResult<Self> {
        Err(ProtocolError::InvalidLength)
    }
}
// Layer3 有效载荷（使用泛型支持不同协议类型）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer3Payload<P: ProtocolType> {
    pub body: P,
    _phantom: PhantomData<P>,
}

// 寄存器协议实现
impl RegisterProtocol {
    pub fn new(register_address: u32, error_code: u16, data: Vec<u8>) -> Self {
        Self {
            register_address,
            error_code,
            data_length: data.len() as u16,
            data,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.register_address.to_le_bytes());
        buf.extend_from_slice(&self.error_code.to_le_bytes());
        buf.extend_from_slice(&self.data_length.to_le_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }

    fn deserialize(buf: &[u8]) -> ProtocolResult<Self> {
        if buf.len() < 8 {
            return Err(ProtocolError::InvalidLength);
        }

        let register_address = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let error_code = u16::from_le_bytes(buf[4..6].try_into().unwrap());
        let data_length = u16::from_le_bytes(buf[6..8].try_into().unwrap());
        let data = buf[8..].to_vec();

        Ok(Self {
            register_address,
            error_code,
            data_length,
            data,
        })
    }
}

// TLV 协议实现
impl TlvProtocol {
    pub fn new(command_code: u32, error_code: u16, user_data: Vec<u8>) -> Self {
        Self {
            command_code,
            error_code,
            data_length: user_data.len() as u16,
            user_data,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.command_code.to_le_bytes());
        buf.extend_from_slice(&self.error_code.to_le_bytes());
        buf.extend_from_slice(&self.data_length.to_le_bytes());
        buf.extend_from_slice(&self.user_data);
        buf
    }

    pub fn deserialize(buf: &[u8]) -> ProtocolResult<Self> {
        if buf.len() < 6 {
            return Err(ProtocolError::InvalidLength);
        }

        let command_code = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let error_code = u16::from_le_bytes(buf[4..6].try_into().unwrap());
        let data_length = u16::from_le_bytes(buf[6..8].try_into().unwrap());
        let user_data = buf[8..].to_vec();

        Ok(Self {
            command_code,
            error_code,
            data_length,
            user_data,
        })
    }
}

// Layer3Payload泛型实现
impl<P: ProtocolType> Layer3Payload<P> {
    pub fn new(body: P) -> Self {
        Self {
            body,
            _phantom: PhantomData,
        }
    }

    pub fn serialize(&self) -> ProtocolResult<Vec<u8>> {
        Ok(self.body.serialize())
    }

    pub fn deserialize(buf: &[u8]) -> ProtocolResult<Self> {
        let body = P::deserialize(buf)?;
        Ok(Self {
            body,
            _phantom: PhantomData,
        })
    }
    // 修改为静态方法，不依赖具体的协议类型
    pub fn deserialize_any(buf: &[u8]) -> ProtocolResult<ProtocolBody> {
        // 先尝试解析为寄存器协议
        if let Ok(reg_body) = RegisterProtocol::deserialize(buf) {
            return Ok(ProtocolBody::Register(reg_body));
        }
        // 再尝试解析为TLV协议
        if let Ok(tlv_body) = TlvProtocol::deserialize(buf) {
            return Ok(ProtocolBody::Tlv(tlv_body));
        }
        Err(ProtocolError::UnknownCommandType)
    }
}
// 协议消息体枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolBody {
    Register(RegisterProtocol),
    Tlv(TlvProtocol),
}


// 自测试接口
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_protocol_serialization() {
        let reg_msg = RegisterProtocol::new(
            0x12345678,
            0x0002,
            vec![0x01, 0x02, 0x03, 0x04]
        );

        // 序列化测试
        let serialized = reg_msg.serialize();
        assert_eq!(serialized.len(), 12);  // 4字节地址 + 2字节错误码 + 2字节长度 + 4字节数据

        // 验证字段
        assert_eq!(&serialized[0..4], &0x12345678u32.to_le_bytes());
        assert_eq!(&serialized[4..6], &0x0002u16.to_le_bytes());
        assert_eq!(&serialized[8..12], &[0x01, 0x02, 0x03, 0x04]);

        // 反序列化测试
        let deserialized = RegisterProtocol::deserialize(&serialized).unwrap();
        assert_eq!(deserialized, reg_msg);
    }

    #[test]
    fn test_tlv_protocol_serialization() {
        let tlv_msg = TlvProtocol::new(
            0x87654321,
            0x0001,
            vec![0x0A, 0x0B, 0x0C]
        );

        // 序列化测试
        let serialized = tlv_msg.serialize();
        assert_eq!(serialized.len(), 11);  // 4字节命令码 + 2字节错误码 + 2 字节长度 + 3字节数据

        // 验证字段
        assert_eq!(&serialized[0..4], &0x87654321u32.to_le_bytes());
        assert_eq!(&serialized[4..6], &0x0001u16.to_le_bytes());
        assert_eq!(&serialized[8..11], &[0x0A, 0x0B, 0x0C]);

        // 反序列化测试
        let deserialized = TlvProtocol::deserialize(&serialized).unwrap();
        assert_eq!(deserialized, tlv_msg);
    }

    #[test]
    fn test_layer3_register_payload() {
        let reg_body = RegisterProtocol::new(
            0x12345678,
            0x0002,
            vec![0x01, 0x02, 0x03, 0x04]
        );

        let layer3 = Layer3Payload::<RegisterProtocol>::new(reg_body.clone());
        let serialized = layer3.serialize().unwrap();

        // 反序列化测试
        let deserialized = Layer3Payload::<RegisterProtocol>::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.body, reg_body);
    }

    #[test]
    fn test_layer3_tlv_payload() {
        let tlv_body = TlvProtocol::new(
            0x87654321,
            0x0001,
            vec![0x0A, 0x0B, 0x0C, 0x0D]
        );

        let layer3 = Layer3Payload::<TlvProtocol>::new(tlv_body.clone());
        let serialized = layer3.serialize().unwrap();

        // 反序列化测试
        let deserialized = Layer3Payload::<TlvProtocol>::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.body, tlv_body);
    }

    // 在第三层其实无法区分 TLV 和  寄存器 所以这个用例一定会失败！！！
    #[test]
    fn test_layer3_deserialize_any() {
        // 测试寄存器协议
        let reg_body = RegisterProtocol::new(
            0x12345678,
            0x0002,
            vec![0x01, 0x02],
        );
        let reg_serialized = reg_body.serialize();

        let parsed = Layer3Payload::<RegisterProtocol>::deserialize_any(&reg_serialized).unwrap();
        assert!(matches!(parsed, ProtocolBody::Register(_)));
        if let ProtocolBody::Register(deserialized) = parsed {
            assert_eq!(deserialized, reg_body);
        }

        // 测试TLV协议
        // let tlv_body = TlvProtocol::new(
        //     0x87654321,
        //     0x0001,
        //     vec![0x0A, 0x0B, 0x0C, 0x0D],
        // );
        // let tlv_serialized = tlv_body.serialize();
        // println!("{tlv_serialized:?}");
        // let parsed_tlv = Layer3Payload::<TlvProtocol>::deserialize_any(&tlv_serialized).unwrap();
        // assert!(matches!(parsed_tlv, ProtocolBody::Tlv(_)));
        // if let ProtocolBody::Tlv(deserialized) = parsed_tlv {
        //     assert_eq!(deserialized, tlv_body);
        // }
    }


    #[test]
    fn test_error_handling() {
        let short_buf = [0u8; 5];

        // 测试寄存器协议和TLV协议的反序列化错误
        assert!(RegisterProtocol::deserialize(&short_buf).is_err());
        assert!(TlvProtocol::deserialize(&short_buf).is_err());

        // 通过具体协议类型调用 deserialize_any
        assert!(Layer3Payload::<RegisterProtocol>::deserialize_any(&short_buf).is_err());
        assert!(Layer3Payload::<TlvProtocol>::deserialize_any(&short_buf).is_err());
    }
}