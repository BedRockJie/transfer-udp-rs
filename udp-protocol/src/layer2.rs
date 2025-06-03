// layer2.rs
use crate::types::{ProtocolError};
pub use crate::types::{DeviceType, ReqRsp, RequestBodyType, ProtocolResult};
// 定义第二层协议结构体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer2Protocol {
    pub req_rsp: ReqRsp,
    pub is_need_reply: bool,
    pub code: bool,
    pub flag: bool,
    pub request_body_type: RequestBodyType,
    pub device_type: DeviceType,
    pub device_index: u16,
    pub group: [u8; 8],
    pub payload: Vec<u8>,
}

impl Layer2Protocol {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // 封装Request Head
        let mut request_head: u8 = 0;
        request_head |= (self.req_rsp as u8 & 0x01) << 7;
        request_head |= (self.is_need_reply as u8 & 0x01) << 6;
        request_head |= (self.code as u8 & 0x01) << 5;
        request_head |= (self.flag as u8 & 0x01) << 4;
        request_head |= self.request_body_type as u8 & 0x0f;
        buf.push(request_head);

        // 封装Device Type
        buf.push(self.device_type as u8);

        // 封装Device Index
        buf.extend_from_slice(&self.device_index.to_le_bytes());

        // 封装Group
        buf.extend_from_slice(&self.group);

        // 封装Payload
        buf.extend_from_slice(&self.payload);

        buf
    }

    pub fn deserialize(buf: &[u8]) -> ProtocolResult<Self> {
        if buf.len() < 12 {
            return Err(ProtocolError::InvalidLength);
        }

        // 解析Request Head
        let request_head = buf[0];
        let req_rsp = if (request_head & 0x80) >> 7 == 0 {
            ReqRsp::Request
        } else {
            ReqRsp::Response
        };
        let is_need_reply = (request_head & 0x40) >> 6 == 1;
        let code = (request_head & 0x20) >> 5 == 1;
        let flag = (request_head & 0x10) >> 4 == 1;
        let request_body_type = match request_head & 0x0f {
            0 => RequestBodyType::RegisterProtocol,
            1 => RequestBodyType::TlvProtocol,
            _ => return Err(ProtocolError::UnsupportedRequestBodyType),
        };

        // 解析Device Type
        let device_type = match buf[1] {
            0x00 => DeviceType::FPGA,
            0x01 => DeviceType::MCU,
            0x02 => DeviceType::NetworkPort,
            0x03 => DeviceType::OpticalPort,
            _ => return Err(ProtocolError::UnsupportedDeviceType),
        };

        // 解析Device Index
        let device_index = u16::from_le_bytes(buf[2..4].try_into().unwrap());

        // 解析Group
        let mut group = [0u8; 8];
        group.copy_from_slice(&buf[4..12]);

        // 解析Payload
        let payload = buf[12..].to_vec();

        Ok(Layer2Protocol {
            req_rsp,
            is_need_reply,
            code,
            flag,
            request_body_type,
            device_type,
            device_index,
            group,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer2_protocol_serialize_and_deserialize() {
        let layer2 = Layer2Protocol {
            req_rsp: ReqRsp::Request,
            is_need_reply: true,
            code: false,
            flag: true,
            request_body_type: RequestBodyType::TlvProtocol,
            device_type: DeviceType::MCU,
            device_index: 10,
            group: [0x01; 8],  // 初始化示例值
            payload: vec![0x01, 0x02, 0x03],
        };

        let serialized = layer2.serialize();
        // print!("serialized: {:#02X?}", serialized);
        let deserialized = Layer2Protocol::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.req_rsp, layer2.req_rsp);
        assert_eq!(deserialized.is_need_reply, layer2.is_need_reply);
        assert_eq!(deserialized.code, layer2.code);
        assert_eq!(deserialized.flag, layer2.flag);
        assert_eq!(deserialized.request_body_type, layer2.request_body_type);
        assert_eq!(deserialized.device_type, layer2.device_type);
        assert_eq!(deserialized.device_index, layer2.device_index);
        assert_eq!(deserialized.group, layer2.group);
        assert_eq!(deserialized.payload, layer2.payload);
    }
}