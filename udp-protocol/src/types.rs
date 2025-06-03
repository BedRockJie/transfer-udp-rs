// types.rs
use std::fmt;

pub type ProtocolResult<T> = Result<T, ProtocolError>;

#[derive(Debug)]
pub enum ProtocolError {
    InvalidChecksum,
    InvalidHeader,
    InvalidLength,
    UnknownCommandType,
    InvalidPayload,
    ChecksumMismatch,
    UnsupportedPriority,
    UnsupportedCheckType,
    UnsupportedFrameType,
    InvalidFrameLength,
    UnsupportedRequestBodyType,
    UnsupportedDeviceType,
    Other(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::InvalidChecksum => write!(f, "Invalid checksum"),
            ProtocolError::InvalidHeader => write!(f, "Invalid header"),
            ProtocolError::InvalidLength => write!(f, "Invalid length"),
            ProtocolError::UnknownCommandType => write!(f, "Unknown command type"),
            ProtocolError::InvalidPayload => write!(f, "Invalid payload"),
            ProtocolError::ChecksumMismatch => write!(f, "Checksum mismatch"),
            ProtocolError::UnsupportedPriority => write!(f, "Unsupported priority"),
            ProtocolError::UnsupportedCheckType => write!(f, "Unsupported check type"),
            ProtocolError::UnsupportedFrameType => write!(f, "Unsupported frame type"),
            ProtocolError::InvalidFrameLength => write!(f, "Invalid frame length"),
            ProtocolError::UnsupportedRequestBodyType => write!(f, "Unsupported request body type"),
            ProtocolError::UnsupportedDeviceType => write!(f, "Unsupported device type"),
            ProtocolError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for ProtocolError {}

// 定义设备类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    FPGA = 0x00,
    MCU = 0x01,
    NetworkPort = 0x02,
    OpticalPort = 0x03,
}

// 定义请求/响应枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReqRsp {
    Request = 0,
    Response = 1,
}

// 定义请求体类型枚举，用于区分是TLV还是寄存器等协议
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestBodyType {
    RegisterProtocol = 0,
    TlvProtocol = 1,
    // 可以根据实际情况扩展其他协议类型
}

// 定义第一层协议中的校验类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckType {
    CheckSum = 0x00,
    // 可以根据实际情况扩展其他校验类型
}

// 定义第一层协议中的优先级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    // 可以根据实际情况扩展其他优先级
}

// 定义第一层协议中的帧类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Type0 = 0,
    Type1 = 1,
    // 可以根据实际情况扩展其他帧类型
}