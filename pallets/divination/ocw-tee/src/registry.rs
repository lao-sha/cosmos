//! # 模块注册表实现
//!
//! 本模块提供 ModuleRegistry trait 的默认实现，
//! 用于在 Public 模式下调用各占卜模块的计算逻辑。
//!
//! ## 使用方式
//!
//! 在 runtime 中配置 ModuleRegistry：
//!
//! ```ignore
//! impl pallet_divination_ocw_tee::Config for Runtime {
//!     // ...
//!     type ModuleRegistry = DivinationModuleRegistry;
//! }
//! ```

use crate::traits::ModuleRegistry;
use crate::types::{DivinationType, ModuleError, ProcessResult};
use codec::{Decode, Encode};
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;

#[cfg(not(feature = "std"))]
use alloc::format;

/// 默认模块注册表实现
///
/// 支持所有占卜类型的 Public 模式处理。
/// 每个占卜类型需要实现对应的处理函数。
pub struct DefaultModuleRegistry;

impl ModuleRegistry for DefaultModuleRegistry {
    fn process_public(
        divination_type: DivinationType,
        input_data: &[u8],
    ) -> Result<ProcessResult, ModuleError> {
        match divination_type {
            DivinationType::Meihua => Self::process_meihua(input_data),
            DivinationType::BaZi => Self::process_bazi(input_data),
            DivinationType::LiuYao => Self::process_liuyao(input_data),
            DivinationType::QiMen => Self::process_qimen(input_data),
            DivinationType::ZiWei => Self::process_ziwei(input_data),
            DivinationType::XiaoLiuRen => Self::process_xiaoliuren(input_data),
            DivinationType::DaLiuRen => Self::process_daliuren(input_data),
            DivinationType::TaiYi => Self::process_taiyi(input_data),
            DivinationType::Tarot => Self::process_tarot(input_data),
        }
    }

    fn is_registered(divination_type: DivinationType) -> bool {
        // 目前支持的模块
        matches!(
            divination_type,
            DivinationType::Meihua
                | DivinationType::BaZi
                | DivinationType::XiaoLiuRen
                | DivinationType::Tarot
        )
    }

    fn get_version(divination_type: DivinationType) -> Option<u32> {
        match divination_type {
            DivinationType::Meihua => Some(1),
            DivinationType::BaZi => Some(1),
            DivinationType::XiaoLiuRen => Some(1),
            DivinationType::Tarot => Some(1),
            _ => None,
        }
    }
}

impl DefaultModuleRegistry {
    /// 处理梅花易数请求
    ///
    /// 输入数据格式：MeihuaPublicInput (SCALE 编码)
    fn process_meihua(input_data: &[u8]) -> Result<ProcessResult, ModuleError> {
        // 解码输入
        let input = MeihuaPublicInput::decode(&mut &input_data[..])
            .map_err(|_| ModuleError::invalid_input(b"Failed to decode Meihua input"))?;

        log::info!(
            "🔮 Meihua: Processing request, method: {:?}",
            input.method
        );

        // 执行计算
        let result = Self::compute_meihua(&input)?;

        // 生成 JSON 清单
        let manifest = Self::generate_meihua_manifest(&input, &result)?;
        let manifest_hash = blake2_256(&manifest);

        // 生成索引（用于链上存储）
        let type_index = result.encode();

        Ok(ProcessResult {
            manifest_cid: Vec::new(), // 由 OCW 上传后填充
            manifest_hash,
            type_index: Some(type_index),
            proof: None,
            manifest_data: Some(manifest),
        })
    }

    /// 梅花易数计算
    fn compute_meihua(input: &MeihuaPublicInput) -> Result<MeihuaResult, ModuleError> {
        // 根据起卦方法计算上卦、下卦、动爻
        let (shang_gua, xia_gua, dong_yao) = match input.method {
            MeihuaMethod::Number { upper, lower } => {
                // 双数起卦
                let shang = calc_gua_num(upper);
                let xia = calc_gua_num(lower);
                let dong = calc_dong_yao(upper + lower);
                (shang, xia, dong)
            }
            MeihuaMethod::Time { year, month, day, hour } => {
                // 时间起卦
                let year_num = year as u32;
                let month_num = month as u32;
                let day_num = day as u32;
                let hour_num = hour as u32;
                
                let shang = calc_gua_num(year_num + month_num + day_num);
                let xia = calc_gua_num(year_num + month_num + day_num + hour_num);
                let dong = calc_dong_yao(year_num + month_num + day_num + hour_num);
                (shang, xia, dong)
            }
            MeihuaMethod::Random { seed } => {
                // 随机起卦
                let hash = blake2_256(&seed.to_le_bytes());
                let shang = calc_gua_num(hash[0] as u32);
                let xia = calc_gua_num(hash[1] as u32);
                let dong = calc_dong_yao(hash[2] as u32);
                (shang, xia, dong)
            }
        };

        // 计算本卦
        let ben_gua = combine_gua(shang_gua, xia_gua);

        // 计算变卦（动爻变化）
        let bian_gua = calc_bian_gua(shang_gua, xia_gua, dong_yao);

        // 计算互卦
        let hu_gua = calc_hu_gua(shang_gua, xia_gua);

        // 判断体用
        let ti_yong = calc_ti_yong(dong_yao);

        Ok(MeihuaResult {
            shang_gua,
            xia_gua,
            dong_yao,
            ben_gua,
            bian_gua,
            hu_gua,
            ti_yong,
        })
    }

    /// 生成梅花易数 JSON 清单
    fn generate_meihua_manifest(
        input: &MeihuaPublicInput,
        result: &MeihuaResult,
    ) -> Result<Vec<u8>, ModuleError> {
        // 构建 JSON 格式的清单
        let json = format!(
            r#"{{"type":"meihua","version":1,"input":{{"method":"{:?}","question":"{}"}},"result":{{"shang_gua":{},"xia_gua":{},"dong_yao":{},"ben_gua":{},"bian_gua":{},"hu_gua":{},"ti_yong":"{:?}"}}}}"#,
            input.method,
            core::str::from_utf8(&input.question).unwrap_or(""),
            result.shang_gua,
            result.xia_gua,
            result.dong_yao,
            result.ben_gua,
            result.bian_gua,
            result.hu_gua,
            result.ti_yong,
        );

        Ok(json.into_bytes())
    }

    /// 处理八字请求
    fn process_bazi(input_data: &[u8]) -> Result<ProcessResult, ModuleError> {
        // 解码输入
        let input = BaziPublicInput::decode(&mut &input_data[..])
            .map_err(|_| ModuleError::invalid_input(b"Failed to decode Bazi input"))?;

        log::info!(
            "🔮 Bazi: Processing request, birth: {}-{}-{} {}:{}",
            input.year, input.month, input.day, input.hour, input.minute
        );

        // 执行计算（简化版，实际应调用完整的八字排盘算法）
        let result = Self::compute_bazi(&input)?;

        // 生成 JSON 清单
        let manifest = Self::generate_bazi_manifest(&input, &result)?;
        let manifest_hash = blake2_256(&manifest);

        // 生成索引
        let type_index = result.encode();

        Ok(ProcessResult {
            manifest_cid: Vec::new(),
            manifest_hash,
            type_index: Some(type_index),
            proof: None,
            manifest_data: Some(manifest),
        })
    }

    /// 八字计算（简化版）
    fn compute_bazi(input: &BaziPublicInput) -> Result<BaziResult, ModuleError> {
        // 这里应该调用完整的八字排盘算法
        // 目前使用简化版本
        Ok(BaziResult {
            year_gan: ((input.year - 4) % 10) as u8,
            year_zhi: ((input.year - 4) % 12) as u8,
            month_gan: 0, // 需要完整算法
            month_zhi: (input.month % 12) as u8,
            day_gan: 0,   // 需要完整算法
            day_zhi: 0,   // 需要完整算法
            hour_gan: 0,  // 需要完整算法
            hour_zhi: (input.hour / 2) as u8,
        })
    }

    /// 生成八字 JSON 清单
    fn generate_bazi_manifest(
        input: &BaziPublicInput,
        result: &BaziResult,
    ) -> Result<Vec<u8>, ModuleError> {
        let json = format!(
            r#"{{"type":"bazi","version":1,"input":{{"year":{},"month":{},"day":{},"hour":{},"minute":{},"gender":"{:?}"}},"result":{{"year_gan":{},"year_zhi":{},"month_gan":{},"month_zhi":{},"day_gan":{},"day_zhi":{},"hour_gan":{},"hour_zhi":{}}}}}"#,
            input.year, input.month, input.day, input.hour, input.minute,
            input.gender,
            result.year_gan, result.year_zhi,
            result.month_gan, result.month_zhi,
            result.day_gan, result.day_zhi,
            result.hour_gan, result.hour_zhi,
        );

        Ok(json.into_bytes())
    }

    /// 处理六爻请求（待实现）
    fn process_liuyao(_input_data: &[u8]) -> Result<ProcessResult, ModuleError> {
        log::warn!("🔮 LiuYao: Not yet implemented");
        Err(ModuleError::ModuleNotRegistered)
    }

    /// 处理奇门遁甲请求（待实现）
    fn process_qimen(_input_data: &[u8]) -> Result<ProcessResult, ModuleError> {
        log::warn!("🔮 QiMen: Not yet implemented");
        Err(ModuleError::ModuleNotRegistered)
    }

    /// 处理紫微斗数请求（待实现）
    fn process_ziwei(_input_data: &[u8]) -> Result<ProcessResult, ModuleError> {
        log::warn!("🔮 ZiWei: Not yet implemented");
        Err(ModuleError::ModuleNotRegistered)
    }

    /// 处理小六壬请求
    fn process_xiaoliuren(input_data: &[u8]) -> Result<ProcessResult, ModuleError> {
        let input = XiaoLiuRenInput::decode(&mut &input_data[..])
            .map_err(|_| ModuleError::invalid_input(b"Failed to decode XiaoLiuRen input"))?;

        log::info!("🔮 XiaoLiuRen: Processing request");

        // 小六壬六神：大安、留连、速喜、赤口、小吉、空亡
        let result = (input.month + input.day + input.hour) % 6;
        let shen = match result {
            0 => "空亡",
            1 => "大安",
            2 => "留连",
            3 => "速喜",
            4 => "赤口",
            5 => "小吉",
            _ => "未知",
        };

        let json = format!(
            r#"{{"type":"xiaoliuren","version":1,"input":{{"month":{},"day":{},"hour":{}}},"result":{{"shen":"{}","index":{}}}}}"#,
            input.month, input.day, input.hour, shen, result
        );

        let manifest = json.into_bytes();
        let manifest_hash = blake2_256(&manifest);

        Ok(ProcessResult {
            manifest_cid: Vec::new(),
            manifest_hash,
            type_index: Some(vec![result as u8]),
            proof: None,
            manifest_data: Some(manifest),
        })
    }

    /// 处理大六壬请求（待实现）
    fn process_daliuren(_input_data: &[u8]) -> Result<ProcessResult, ModuleError> {
        log::warn!("🔮 DaLiuRen: Not yet implemented");
        Err(ModuleError::ModuleNotRegistered)
    }

    /// 处理太乙神数请求（待实现）
    fn process_taiyi(_input_data: &[u8]) -> Result<ProcessResult, ModuleError> {
        log::warn!("🔮 TaiYi: Not yet implemented");
        Err(ModuleError::ModuleNotRegistered)
    }

    /// 处理塔罗牌请求
    fn process_tarot(input_data: &[u8]) -> Result<ProcessResult, ModuleError> {
        let input = TarotInput::decode(&mut &input_data[..])
            .map_err(|_| ModuleError::invalid_input(b"Failed to decode Tarot input"))?;

        log::info!("🔮 Tarot: Processing request, spread: {:?}", input.spread);

        // 使用种子生成随机牌
        let hash = blake2_256(&input.seed.to_le_bytes());
        let cards: Vec<u8> = (0..input.card_count.min(10))
            .map(|i| hash[i as usize] % 78) // 78张塔罗牌
            .collect();

        let json = format!(
            r#"{{"type":"tarot","version":1,"input":{{"spread":"{:?}","question":"{}"}},"result":{{"cards":{:?}}}}}"#,
            input.spread,
            core::str::from_utf8(&input.question).unwrap_or(""),
            cards
        );

        let manifest = json.into_bytes();
        let manifest_hash = blake2_256(&manifest);

        Ok(ProcessResult {
            manifest_cid: Vec::new(),
            manifest_hash,
            type_index: Some(cards.clone()),
            proof: None,
            manifest_data: Some(manifest),
        })
    }
}

// ==================== 辅助函数 ====================

/// 计算卦数（1-8）
#[inline]
fn calc_gua_num(n: u32) -> u8 {
    let r = (n % 8) as u8;
    if r == 0 { 8 } else { r }
}

/// 计算动爻数（1-6）
#[inline]
fn calc_dong_yao(n: u32) -> u8 {
    let r = (n % 6) as u8;
    if r == 0 { 6 } else { r }
}

/// 组合上下卦为六十四卦
#[inline]
fn combine_gua(shang: u8, xia: u8) -> u8 {
    (shang - 1) * 8 + xia
}

/// 计算变卦
fn calc_bian_gua(shang: u8, xia: u8, dong_yao: u8) -> u8 {
    // 动爻在上卦还是下卦
    if dong_yao <= 3 {
        // 动爻在下卦，下卦变
        let new_xia = (xia ^ (1 << (dong_yao - 1))) % 8;
        let new_xia = if new_xia == 0 { 8 } else { new_xia };
        combine_gua(shang, new_xia)
    } else {
        // 动爻在上卦，上卦变
        let new_shang = (shang ^ (1 << (dong_yao - 4))) % 8;
        let new_shang = if new_shang == 0 { 8 } else { new_shang };
        combine_gua(new_shang, xia)
    }
}

/// 计算互卦
fn calc_hu_gua(shang: u8, xia: u8) -> u8 {
    // 互卦：取本卦2、3、4爻为下卦，3、4、5爻为上卦
    // 简化实现
    combine_gua((shang + xia) % 8 + 1, (shang * xia) % 8 + 1)
}

/// 判断体用
fn calc_ti_yong(dong_yao: u8) -> TiYong {
    if dong_yao <= 3 {
        TiYong::ShangTi // 上卦为体
    } else {
        TiYong::XiaTi // 下卦为体
    }
}

// ==================== 输入输出类型 ====================

/// 梅花易数公开输入
#[derive(Clone, Debug, Encode, Decode)]
pub struct MeihuaPublicInput {
    /// 起卦方法
    pub method: MeihuaMethod,
    /// 占问事项
    pub question: Vec<u8>,
}

/// 梅花易数起卦方法
#[derive(Clone, Debug, Encode, Decode)]
pub enum MeihuaMethod {
    /// 双数起卦
    Number { upper: u32, lower: u32 },
    /// 时间起卦
    Time { year: u16, month: u8, day: u8, hour: u8 },
    /// 随机起卦
    Random { seed: u64 },
}

/// 梅花易数结果
#[derive(Clone, Debug, Encode, Decode)]
pub struct MeihuaResult {
    pub shang_gua: u8,
    pub xia_gua: u8,
    pub dong_yao: u8,
    pub ben_gua: u8,
    pub bian_gua: u8,
    pub hu_gua: u8,
    pub ti_yong: TiYong,
}

/// 体用关系
#[derive(Clone, Debug, Encode, Decode)]
pub enum TiYong {
    ShangTi, // 上卦为体
    XiaTi,   // 下卦为体
}

/// 八字公开输入
#[derive(Clone, Debug, Encode, Decode)]
pub struct BaziPublicInput {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub gender: Gender,
}

/// 性别
#[derive(Clone, Debug, Encode, Decode)]
pub enum Gender {
    Male,
    Female,
}

/// 八字结果
#[derive(Clone, Debug, Encode, Decode)]
pub struct BaziResult {
    pub year_gan: u8,
    pub year_zhi: u8,
    pub month_gan: u8,
    pub month_zhi: u8,
    pub day_gan: u8,
    pub day_zhi: u8,
    pub hour_gan: u8,
    pub hour_zhi: u8,
}

/// 小六壬输入
#[derive(Clone, Debug, Encode, Decode)]
pub struct XiaoLiuRenInput {
    pub month: u8,
    pub day: u8,
    pub hour: u8,
}

/// 塔罗牌输入
#[derive(Clone, Debug, Encode, Decode)]
pub struct TarotInput {
    pub spread: TarotSpread,
    pub question: Vec<u8>,
    pub seed: u64,
    pub card_count: u8,
}

/// 塔罗牌牌阵
#[derive(Clone, Debug, Encode, Decode)]
pub enum TarotSpread {
    Single,      // 单牌
    ThreeCard,   // 三牌
    Celtic,      // 凯尔特十字
    Custom,      // 自定义
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_gua_num() {
        assert_eq!(calc_gua_num(1), 1);
        assert_eq!(calc_gua_num(8), 8);
        assert_eq!(calc_gua_num(9), 1);
        assert_eq!(calc_gua_num(16), 8);
    }

    #[test]
    fn test_calc_dong_yao() {
        assert_eq!(calc_dong_yao(1), 1);
        assert_eq!(calc_dong_yao(6), 6);
        assert_eq!(calc_dong_yao(7), 1);
        assert_eq!(calc_dong_yao(12), 6);
    }

    #[test]
    fn test_meihua_number() {
        let input = MeihuaPublicInput {
            method: MeihuaMethod::Number { upper: 5, lower: 3 },
            question: b"test".to_vec(),
        };
        let result = DefaultModuleRegistry::compute_meihua(&input).unwrap();
        assert!(result.shang_gua >= 1 && result.shang_gua <= 8);
        assert!(result.xia_gua >= 1 && result.xia_gua <= 8);
        assert!(result.dong_yao >= 1 && result.dong_yao <= 6);
    }
}
