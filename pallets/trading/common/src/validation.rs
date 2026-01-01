//! # 验证函数模块
//!
//! 提供 TRON 地址、EPAY 配置的验证

use sp_std::prelude::*;

/// 函数级详细中文注释：验证 TRON 地址格式
///
/// # 规则
/// - 长度：34 字符
/// - 开头：'T'
/// - 编码：Base58（字符集：1-9, A-H, J-N, P-Z, a-k, m-z）
///
/// # 参数
/// - address: TRON 地址字节数组
///
/// # 返回
/// - bool: 有效返回 true，无效返回 false
pub fn is_valid_tron_address(address: &[u8]) -> bool {
    // 长度检查
    if address.len() != 34 {
        return false;
    }

    // 开头检查
    if address[0] != b'T' {
        return false;
    }

    // Base58 字符集检查
    const BASE58_CHARS: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    for &byte in address {
        if !BASE58_CHARS.contains(&byte) {
            return false;
        }
    }

    true
}

/// 函数级详细中文注释：验证 EPAY 配置
///
/// # 规则
/// - epay_no: 10-32 字符
/// - epay_key: 16-64 字符
/// - 两者要么都有，要么都没有
///
/// # 参数
/// - epay_no: EPAY 商户号（可选）
/// - epay_key: EPAY 密钥（可选）
///
/// # 返回
/// - bool: 有效返回 true，无效返回 false
pub fn is_valid_epay_config(epay_no: &Option<Vec<u8>>, epay_key: &Option<Vec<u8>>) -> bool {
    match (epay_no, epay_key) {
        (Some(no), Some(key)) => {
            // 都有：检查长度
            no.len() >= 10 && no.len() <= 32 && key.len() >= 16 && key.len() <= 64
        },
        (None, None) => {
            // 都没有：有效
            true
        },
        _ => {
            // 只有一个：无效
            false
        }
    }
}

// ===== 单元测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_tron_address() {
        assert!(is_valid_tron_address(b"TYASr5UV6HEcXatwdFQfmLVUqQQQMUxHLS"));
        assert!(!is_valid_tron_address(b"TYASR5UV6HEcXatwdFQfmLVUqQQQMUxHLS")); // 长度不对
        assert!(!is_valid_tron_address(b"AYASr5UV6HEcXatwdFQfmLVUqQQQMUxHLS")); // 不是T开头
        assert!(!is_valid_tron_address(b"TYASr5UV6HEcXatwdFQfmLVUqQQQMUxHL0")); // 包含0（非Base58）
    }

    #[test]
    fn test_is_valid_epay_config() {
        assert!(is_valid_epay_config(&None, &None));
        assert!(is_valid_epay_config(
            &Some(b"1234567890".to_vec()),
            &Some(b"1234567890123456".to_vec())
        ));
        assert!(!is_valid_epay_config(&Some(b"123".to_vec()), &None));
        assert!(!is_valid_epay_config(&None, &Some(b"123".to_vec())));
        assert!(!is_valid_epay_config(&Some(b"123".to_vec()), &Some(b"123".to_vec())));
    }
}
