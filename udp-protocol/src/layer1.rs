// layer1.rs
pub use crate::types::{CheckType, FrameType, Priority, ProtocolError, ProtocolResult};
use crate::utils::{calc_checksum, verify_checksum};

// 定义第一层协议结构体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer1Protocol {
    pub frame_delimiter_0: u8,
    pub frame_delimiter_1: u8,
    pub version: u8,
    pub priority: Priority,
    pub check_type: CheckType,
    pub frame_type: FrameType,
    pub frame_seq_number: u16,
    pub frame_length: u16,
    pub payload: Vec<u8>,
    pub checksum: u16,
}

impl Layer1Protocol {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // 封装Frame Head部分
        buf.push(self.frame_delimiter_0);
        buf.push(self.frame_delimiter_1);
        buf.push(self.version);
        buf.push(self.priority as u8);
        buf.push(self.check_type as u8);
        buf.push(self.frame_type as u8);
        buf.extend_from_slice(&self.frame_seq_number.to_le_bytes());
        // 这里先预留Frame Length位置，后续计算填充
        buf.extend_from_slice(&[0u8; 2]);

        // 封装Payload
        buf.extend_from_slice(&self.payload);

        // 计算并填充Frame Length
        let payload_length = self.payload.len() as u16;
        let frame_length_value = payload_length + 2;  // 加上校验和长度
        buf[8..10].copy_from_slice(&frame_length_value.to_le_bytes());

        let sum = calc_checksum(&buf);
        buf.extend_from_slice(&sum.to_le_bytes());

        buf
    }

    pub fn deserialize(buf: &[u8]) -> ProtocolResult<Self> {
        if buf.len() < 12 {
            return Err(ProtocolError::InvalidLength);
        }

        // 解析Frame Head部分
        let frame_delimiter_0 = buf[0];
        let frame_delimiter_1 = buf[1];
        let version = buf[2];
        let priority = match buf[3] {
            0 => Priority::Low,
            1 => Priority::Medium,
            2 => Priority::High,
            _ => return Err(ProtocolError::UnsupportedPriority),
        };
        let check_type = match buf[4] {
            0x00 => CheckType::CheckSum,
            _ => return Err(ProtocolError::UnsupportedCheckType),
        };
        let frame_type = match buf[5] {
            0 => FrameType::Type0,
            1 => FrameType::Type1,
            _ => return Err(ProtocolError::UnsupportedFrameType),
        };
        let frame_seq_number = u16::from_le_bytes(buf[6..8].try_into().unwrap());
        let frame_length = u16::from_le_bytes(buf[8..10].try_into().unwrap());
        // 验证帧长度是否合法
        let expected_total_length = frame_length + 10; // 帧头10字节 + Frame Length定义的长度（Payload + 校验和）
        if buf.len() as u16 != expected_total_length {
            return Err(ProtocolError::InvalidLength);
        }

        // 解析Payload
        let payload_start_index = 10;
        let payload_end_index = payload_start_index + (frame_length - 2) as usize;
        if payload_end_index > buf.len() {
            return Err(ProtocolError::InvalidFrameLength);
        }
        let payload = buf[payload_start_index..payload_end_index].to_vec();

        let received_checksum = u16::from_le_bytes(buf[payload_end_index..payload_end_index + 2].try_into().unwrap());

        if !verify_checksum(&buf[0..payload_end_index], received_checksum) {
            return Err(ProtocolError::ChecksumMismatch);
        }

        Ok(Layer1Protocol {
            frame_delimiter_0,
            frame_delimiter_1,
            version,
            priority,
            check_type,
            frame_type,
            frame_seq_number,
            frame_length,
            payload,
            checksum: received_checksum,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer1_protocol_serialize_and_deserialize() {
        let payload_data = vec![0x01, 0x02, 0x03];
        let layer1 = Layer1Protocol {
            frame_delimiter_0: 0x55,
            frame_delimiter_1: 0xBB,
            version: 1,
            priority: Priority::Medium,
            check_type: CheckType::CheckSum,
            frame_type: FrameType::Type1,
            frame_seq_number: 1,
            frame_length: 0,  // 这里初始化值后续会在serialize中计算填充
            payload: payload_data,
            checksum: 0,  // 这里初始化值后续会在serialize中计算填充
        };

        let serialized = layer1.serialize();
        let deserialized = Layer1Protocol::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.frame_delimiter_0, layer1.frame_delimiter_0);
        assert_eq!(deserialized.frame_delimiter_1, layer1.frame_delimiter_1);
        assert_eq!(deserialized.version, layer1.version);
        assert_eq!(deserialized.priority, layer1.priority);
        assert_eq!(deserialized.check_type, layer1.check_type);
        assert_eq!(deserialized.frame_type, layer1.frame_type);
        assert_eq!(deserialized.frame_seq_number, layer1.frame_seq_number);
        assert_eq!(deserialized.payload, layer1.payload);
    }
}