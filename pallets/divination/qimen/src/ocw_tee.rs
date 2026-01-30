//! # 奇门遁甲模块 OCW + TEE 集成
//!
//! 实现 `DivinationModule` trait，接入通用 OCW + TEE 架构。
//!
//! ## 功能
//!
//! - 实现 `DivinationModule` trait
//! - 定义奇门专用输入/输出类型
//! - 提供 JSON 清单生成
//! - 支持三种隐私模式
//!
//! ## 奇门特有的隐私考虑
//!
//! 奇门遁甲与八字不同，其敏感数据主要是"占问事宜"（question），
//! 而非出生时间。因此隐私模式的处理有所不同：
//!
//! - **Public**: 占问事宜明文存储
//! - **Encrypted**: 占问事宜加密，但四柱索引明文
//! - **Private**: 所有数据加密，链上不存储任何索引

use crate::types::{
    DunType, GanZhi, Gender, JieQi, JiuGong, JiuXing, BaMen, Palace, SanYuan,
    QimenChartResult, GeJuType, Fortune, DeLiStatus,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use pallet_divination_ocw_tee::{
    DivinationModule, DivinationType, ModuleError, PrivacyMode, ProcessResult,
};
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ==================== 奇门输入类型（明文）====================

/// 奇门明文输入
///
/// 用于 OCW + TEE 架构的标准化输入格式。
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct QimenInputPlain {
    /// 起卦时间 - 年
    pub year: u16,
    /// 起卦时间 - 月
    pub month: u8,
    /// 起卦时间 - 日
    pub day: u8,
    /// 起卦时间 - 时
    pub hour: u8,
    /// 起卦时间 - 分
    pub minute: u8,
    /// 命主性别（可选）
    pub gender: Option<Gender>,
    /// 命主姓名（可选，最大32字节）
    pub name: Option<BoundedVec<u8, ConstU32<32>>>,
    /// 占问事宜（可选，最大128字节）
    pub question: Option<BoundedVec<u8, ConstU32<128>>>,
}

impl QimenInputPlain {
    /// 验证输入有效性
    pub fn is_valid(&self) -> bool {
        // 验证年份范围
        if self.year < 1900 || self.year > 2100 {
            return false;
        }
        // 验证月份
        if self.month < 1 || self.month > 12 {
            return false;
        }
        // 验证日期
        if self.day < 1 || self.day > 31 {
            return false;
        }
        // 验证小时
        if self.hour > 23 {
            return false;
        }
        // 验证分钟
        if self.minute > 59 {
            return false;
        }
        true
    }
}

// ==================== 奇门索引类型 ====================

/// 奇门四柱索引（8 bytes）
///
/// 仅保存四柱的干支索引，用于链上存储。
/// 与八字不同，奇门的四柱是起卦时间而非出生时间。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct QimenSiZhuIndex {
    /// 年柱天干索引 (0-9)
    pub year_gan: u8,
    /// 年柱地支索引 (0-11)
    pub year_zhi: u8,
    /// 月柱天干索引 (0-9)
    pub month_gan: u8,
    /// 月柱地支索引 (0-11)
    pub month_zhi: u8,
    /// 日柱天干索引 (0-9)
    pub day_gan: u8,
    /// 日柱地支索引 (0-11)
    pub day_zhi: u8,
    /// 时柱天干索引 (0-9)
    pub hour_gan: u8,
    /// 时柱地支索引 (0-11)
    pub hour_zhi: u8,
}

impl QimenSiZhuIndex {
    /// 从 GanZhi 创建索引
    pub fn from_ganzhi(
        year: &GanZhi,
        month: &GanZhi,
        day: &GanZhi,
        hour: &GanZhi,
    ) -> Self {
        Self {
            year_gan: year.gan.index(),
            year_zhi: year.zhi.index(),
            month_gan: month.gan.index(),
            month_zhi: month.zhi.index(),
            day_gan: day.gan.index(),
            day_zhi: day.zhi.index(),
            hour_gan: hour.gan.index(),
            hour_zhi: hour.zhi.index(),
        }
    }

    /// 验证索引有效性
    pub fn is_valid(&self) -> bool {
        self.year_gan < 10 && self.year_zhi < 12 &&
        self.month_gan < 10 && self.month_zhi < 12 &&
        self.day_gan < 10 && self.day_zhi < 12 &&
        self.hour_gan < 10 && self.hour_zhi < 12
    }
}

// ==================== 奇门计算结果 ====================

/// 奇门计算结果（简化版，用于 JSON 清单）
#[derive(Clone, Debug, Encode, Decode)]
pub struct QimenComputeResult {
    /// 四柱索引
    pub sizhu_index: QimenSiZhuIndex,
    /// 节气
    pub jie_qi: JieQi,
    /// 阴阳遁
    pub dun_type: DunType,
    /// 三元
    pub san_yuan: SanYuan,
    /// 局数（1-9）
    pub ju_number: u8,
    /// 值符星
    pub zhi_fu_xing: JiuXing,
    /// 值使门
    pub zhi_shi_men: BaMen,
    /// 九宫数据（简化版，仅存储关键信息）
    pub palace_data: [PalaceData; 9],
    /// 格局类型
    pub ge_ju: Option<GeJuType>,
    /// 综合吉凶
    pub fortune: Option<Fortune>,
}

/// 宫位数据（简化版）
#[derive(Clone, Copy, Debug, Encode, Decode)]
pub struct PalaceData {
    /// 宫位
    pub gong: JiuGong,
    /// 天盘干
    pub tian_pan_gan: u8,
    /// 地盘干
    pub di_pan_gan: u8,
    /// 九星
    pub xing: u8,
    /// 八门
    pub men: u8,
    /// 八神
    pub shen: u8,
}

// ==================== JSON 清单结构 ====================

/// 奇门 JSON 清单（用于 IPFS 存储）
#[derive(Clone, Debug, Encode, Decode)]
pub struct QimenManifest {
    /// 版本号
    pub version: u32,
    /// 模块 ID
    pub module_id: u8,
    /// 隐私模式
    pub privacy_mode: u8,
    /// 创建时间戳（Unix 秒）
    pub created_at: u64,

    // ===== 公开数据（Public/Encrypted 模式）=====
    /// 四柱索引
    pub sizhu_index: Option<QimenSiZhuIndex>,
    /// 节气
    pub jie_qi: Option<u8>,
    /// 阴阳遁
    pub dun_type: Option<u8>,
    /// 三元
    pub san_yuan: Option<u8>,
    /// 局数
    pub ju_number: Option<u8>,
    /// 值符星
    pub zhi_fu_xing: Option<u8>,
    /// 值使门
    pub zhi_shi_men: Option<u8>,
    /// 九宫数据
    pub palace_data: Option<[PalaceData; 9]>,
    /// 格局类型
    pub ge_ju: Option<u8>,
    /// 综合吉凶
    pub fortune: Option<u8>,

    // ===== 敏感数据（仅 Public 模式明文）=====
    /// 起卦时间（仅 Public 模式）
    pub divination_time: Option<DivinationTimeJson>,
    /// 命主姓名（仅 Public 模式）
    pub name: Option<Vec<u8>>,
    /// 占问事宜（仅 Public 模式）
    pub question: Option<Vec<u8>>,

    // ===== 加密数据（Encrypted/Private 模式）=====
    /// 加密的敏感数据
    pub encrypted_sensitive: Option<Vec<u8>>,
    /// 加密数据的哈希
    pub sensitive_hash: Option<[u8; 32]>,
}

/// 起卦时间 JSON 格式
#[derive(Clone, Debug, Encode, Decode)]
pub struct DivinationTimeJson {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

// ==================== DivinationModule 实现 ====================

/// 奇门模块 OCW + TEE 处理器
pub struct QimenModuleHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: crate::pallet::Config> DivinationModule<T> for QimenModuleHandler<T> {
    const MODULE_ID: DivinationType = DivinationType::QiMen;
    const MODULE_NAME: &'static str = "QiMen";
    const VERSION: u32 = 1;

    type PlainInput = QimenInputPlain;
    type Index = QimenSiZhuIndex;
    type Result = QimenComputeResult;

    /// 执行奇门计算
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError> {
        // 验证输入
        <Self as DivinationModule<T>>::validate_input(input)?;

        // 简化实现：内联基本算法
        let year = input.year.unwrap_or(2024);
        let month = input.month.unwrap_or(1);
        let day = input.day.unwrap_or(1);
        let hour = input.hour.unwrap_or(0);

        // 简化的四柱计算
        let year_gan = ((year - 4) % 10) as u8;
        let year_zhi = ((year - 4) % 12) as u8;
        let month_gan = ((year_gan as u16 * 2 + month as u16) % 10) as u8;
        let month_zhi = ((month + 1) % 12) as u8;
        let day_gan = ((year as u32 * 5 + (year as u32 / 4) + day as u32) % 10) as u8;
        let day_zhi = ((year as u32 * 5 + (year as u32 / 4) + day as u32) % 12) as u8;
        let hour_gan = ((day_gan as u16 * 2 + hour as u16 / 2) % 10) as u8;
        let hour_zhi = (hour / 2) % 12;

        // 简化的阴阳遁和局数
        let is_yang_dun = month <= 6;
        let ju_num = if is_yang_dun { (day % 9 + 1) as u8 } else { (9 - day % 9) as u8 };

        // 简化的九宫数据
        let jiu_gong = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];

        Ok(QimenComputeResult {
            sizhu_index: QimenSizhuIndex {
                year_gan,
                year_zhi,
                month_gan,
                month_zhi,
                day_gan,
                day_zhi,
                hour_gan,
                hour_zhi,
            },
            is_yang_dun,
            ju_num,
            jiu_gong,
        })
    }

    /// 从计算结果提取索引
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Public | PrivacyMode::PublicEncrypted | PrivacyMode::Encrypted => Some(result.sizhu_index),
            PrivacyMode::Private => None,
        }
    }

    /// 生成 JSON 清单
    fn generate_manifest(
        input: &Self::PlainInput,
        result: &Self::Result,
        privacy_mode: PrivacyMode,
    ) -> Result<Vec<u8>, ModuleError> {
        let manifest = match privacy_mode {
            PrivacyMode::Public | PrivacyMode::PublicEncrypted => {
                // 公开/公开加密模式：所有数据明文（PublicEncrypted 会在 OCW 层加密整个清单）
                QimenManifest {
                    version: Self::VERSION,
                    module_id: Self::MODULE_ID as u8,
                    privacy_mode: privacy_mode as u8,
                    created_at: 0,
                    sizhu_index: Some(result.sizhu_index),
                    jie_qi: Some(result.jie_qi as u8),
                    dun_type: Some(result.dun_type as u8),
                    san_yuan: Some(result.san_yuan as u8),
                    ju_number: Some(result.ju_number),
                    zhi_fu_xing: Some(result.zhi_fu_xing as u8),
                    zhi_shi_men: Some(result.zhi_shi_men as u8),
                    palace_data: Some(result.palace_data),
                    ge_ju: result.ge_ju.map(|g| g as u8),
                    fortune: result.fortune.map(|f| f as u8),
                    divination_time: Some(DivinationTimeJson {
                        year: input.year,
                        month: input.month,
                        day: input.day,
                        hour: input.hour,
                        minute: input.minute,
                    }),
                    name: input.name.as_ref().map(|n| n.to_vec()),
                    question: input.question.as_ref().map(|q| q.to_vec()),
                    encrypted_sensitive: None,
                    sensitive_hash: None,
                }
            }
            PrivacyMode::Encrypted => {
                // 加密模式：索引和计算数据明文，敏感数据加密
                QimenManifest {
                    version: Self::VERSION,
                    module_id: Self::MODULE_ID as u8,
                    privacy_mode: privacy_mode as u8,
                    created_at: 0,
                    sizhu_index: Some(result.sizhu_index),
                    jie_qi: Some(result.jie_qi as u8),
                    dun_type: Some(result.dun_type as u8),
                    san_yuan: Some(result.san_yuan as u8),
                    ju_number: Some(result.ju_number),
                    zhi_fu_xing: Some(result.zhi_fu_xing as u8),
                    zhi_shi_men: Some(result.zhi_shi_men as u8),
                    palace_data: Some(result.palace_data),
                    ge_ju: result.ge_ju.map(|g| g as u8),
                    fortune: result.fortune.map(|f| f as u8),
                    divination_time: None, // 加密
                    name: None,            // 加密
                    question: None,        // 加密（最敏感）
                    encrypted_sensitive: None, // TODO: 由 TEE 填充
                    sensitive_hash: None,
                }
            }
            PrivacyMode::Private => {
                // 私密模式：所有数据加密
                QimenManifest {
                    version: Self::VERSION,
                    module_id: Self::MODULE_ID as u8,
                    privacy_mode: privacy_mode as u8,
                    created_at: 0,
                    sizhu_index: None,
                    jie_qi: None,
                    dun_type: None,
                    san_yuan: None,
                    ju_number: None,
                    zhi_fu_xing: None,
                    zhi_shi_men: None,
                    palace_data: None,
                    ge_ju: None,
                    fortune: None,
                    divination_time: None,
                    name: None,
                    question: None,
                    encrypted_sensitive: None, // TODO: 由 TEE 填充
                    sensitive_hash: None,
                }
            }
        };

        Ok(manifest.encode())
    }

    /// 验证输入有效性
    fn validate_input(input: &Self::PlainInput) -> Result<(), ModuleError> {
        if !input.is_valid() {
            return Err(ModuleError::invalid_input(b"Invalid QiMen input"));
        }
        Ok(())
    }

    /// 获取推荐超时时间（奇门计算较复杂）
    fn recommended_timeout() -> u32 {
        150 // ~15 分钟
    }

    /// 获取最大输入大小（包含占问事宜）
    fn max_input_size() -> u32 {
        512
    }

    /// 是否支持批量处理
    fn supports_batch() -> bool {
        false
    }

    /// 获取 TEE 端点路径
    fn tee_endpoint() -> &'static str {
        "/compute/qimen"
    }
}

// ==================== 辅助函数 ====================

/// 从 QimenChartResult 创建 ProcessResult
pub fn create_process_result(
    result: &QimenChartResult,
    manifest_cid: Vec<u8>,
    manifest_hash: [u8; 32],
) -> ProcessResult {
    let sizhu_index = QimenSiZhuIndex::from_ganzhi(
        &result.year_ganzhi,
        &result.month_ganzhi,
        &result.day_ganzhi,
        &result.hour_ganzhi,
    );

    ProcessResult {
        manifest_cid,
        manifest_hash,
        type_index: Some(sizhu_index.encode()),
        proof: None,
        manifest_data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qimen_input_validation() {
        // 有效输入
        let valid_input = QimenInputPlain {
            year: 2024,
            month: 1,
            day: 15,
            hour: 10,
            minute: 30,
            gender: Some(Gender::Male),
            name: None,
            question: None,
        };
        assert!(valid_input.is_valid());

        // 无效年份
        let invalid_year = QimenInputPlain {
            year: 1800,
            month: 1,
            day: 15,
            hour: 10,
            minute: 30,
            gender: None,
            name: None,
            question: None,
        };
        assert!(!invalid_year.is_valid());

        // 无效月份
        let invalid_month = QimenInputPlain {
            year: 2024,
            month: 13,
            day: 15,
            hour: 10,
            minute: 30,
            gender: None,
            name: None,
            question: None,
        };
        assert!(!invalid_month.is_valid());
    }

    #[test]
    fn test_sizhu_index_validation() {
        let valid_index = QimenSiZhuIndex {
            year_gan: 0,
            year_zhi: 0,
            month_gan: 2,
            month_zhi: 2,
            day_gan: 4,
            day_zhi: 4,
            hour_gan: 6,
            hour_zhi: 6,
        };
        assert!(valid_index.is_valid());

        let invalid_index = QimenSiZhuIndex {
            year_gan: 10, // 超出范围
            year_zhi: 0,
            month_gan: 0,
            month_zhi: 0,
            day_gan: 0,
            day_zhi: 0,
            hour_gan: 0,
            hour_zhi: 0,
        };
        assert!(!invalid_index.is_valid());
    }
}
