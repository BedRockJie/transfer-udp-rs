// utils.rs

/// 计算简单的校验和（所有字节相加的反码加一）
pub fn calc_checksum(data: &[u8]) -> u16 {
    // 创建迭代器针对迭代器中所有的属性求和
    let sum: u16 = data.iter().map(|&b| b as u16).sum();
    (!sum as u16).wrapping_add(1)
}

/// 验证数据和校验和是否一致
pub fn verify_checksum(data: &[u8], checksum: u16) -> bool {
    calc_checksum(data) == checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let data = [0x01, 0x02, 0x03];
        let checksum = calc_checksum(&data);
        assert!(verify_checksum(&data, checksum));
    }

    #[test]
    fn test_invalid_checksum() {
        let data = [0x10, 0x20, 0x30];
        let checksum = 0x00;
        assert!(!verify_checksum(&data, checksum));
    }
}
