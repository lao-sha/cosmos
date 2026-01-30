//! # 梅花易数模块 OCW + TEE 集成
//!
//! 实现 `DivinationModule` trait，接入通用 OCW + TEE 架构。
//!
//! ## 功能
//!
//! - 实现 `DivinationModule` trait
//! - 定义梅花专用输入/输出类型
//! - 提供 JSON 清单生成
//! - 支持三种隐私模式
//!
//! ## 梅花易数特有的隐私考虑
//!
//! 梅花易数的敏感数据主要是"占问事宜"（question），
//! 卦象本身（上卦、下卦、动爻）通常不涉及隐私。
//!
//! - **Public**: 占问事宜明文存储
//! - **Encrypted**: 占问事宜加密，卦象索引明文
//! - **Private**: 所有数据加密

use crate::types::{
    Bagua, DivinationMethod, Fortune, SingleGua, TiYongRelation, WuXing, WangShuai,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use pallet_divination_ocw_tee::{
    DivinationModule, DivinationType, ModuleError, PrivacyMode, ProcessResult,
};
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ==================== 梅花输入类型（明文）====================

/// 梅花明文输入
///
/// 用于 OCW + TEE 架构的标准化输入格式。
/// 支持多种起卦方式。
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct MeihuaInputPlain {
    /// 起卦方式
    pub method: DivinationMethod,
    /// 起卦时间 - 年（时间起卦时使用）
    pub year: Option<u16>,
    /// 起卦时间 - 月
    pub month: Option<u8>,
    /// 起卦时间 - 日
    pub day: Option<u8>,
    /// 起卦时间 - 时
    pub hour: Option<u8>,
    /// 起卦时间 - 分
    pub minute: Option<u8>,
    /// 双数起卦 - 第一个数
    pub number1: Option<u32>,
    /// 双数起卦 - 第二个数
    pub number2: Option<u32>,
    /// 手动指定 - 上卦 (1-8)
    pub shang_gua: Option<u8>,
    /// 手动指定 - 下卦 (1-8)
    pub xia_gua: Option<u8>,
    /// 手动指定 - 动爻 (1-6)
    pub dong_yao: Option<u8>,
    /// 性别（0: 未指定, 1: 男, 2: 女）
    pub gender: u8,
    /// 出生年份（可选）
    pub birth_year: Option<u16>,
    /// 占问事宜（可选，最大128字节）
    pub question: Option<BoundedVec<u8, ConstU32<128>>>,
}

impl MeihuaInputPlain {
    /// 验证输入有效性
    pub fn is_valid(&self) -> bool {
        match self.method {
            DivinationMethod::LunarDateTime | DivinationMethod::GregorianDateTime => {
                // 时间起卦需要完整时间
                if let (Some(year), Some(month), Some(day), Some(hour)) =
                    (self.year, self.month, self.day, self.hour)
                {
                    year >= 1900 && year <= 2100 &&
                    month >= 1 && month <= 12 &&
                    day >= 1 && day <= 31 &&
                    hour <= 23
                } else {
                    false
                }
            }
            DivinationMethod::TwoNumbers => {
                // 双数起卦需要两个数字
                self.number1.is_some() && self.number2.is_some()
            }
            DivinationMethod::SingleNumber => {
                // 单数起卦需要一个数字
                self.number1.is_some()
            }
            DivinationMethod::Manual => {
                // 手动指定需要上卦、下卦、动爻
                if let (Some(shang), Some(xia), Some(dong)) =
                    (self.shang_gua, self.xia_gua, self.dong_yao)
                {
                    shang >= 1 && shang <= 8 &&
                    xia >= 1 && xia <= 8 &&
                    dong >= 1 && dong <= 6
                } else {
                    false
                }
            }
            DivinationMethod::Random | DivinationMethod::ChainShake => {
                // 随机起卦和链摇起卦不需要额外参数
                true
            }
        }
    }
}

// ==================== 梅花索引类型 ====================

/// 梅花卦象索引（3 bytes）
///
/// 仅保存卦象的核心索引，用于链上存储。
/// 不包含任何敏感信息（如占问事宜）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct MeihuaIndex {
    /// 上卦索引 (1-8，先天八卦数)
    pub shang_gua: u8,
    /// 下卦索引 (1-8，先天八卦数)
    pub xia_gua: u8,
    /// 动爻位置 (1-6)
    pub dong_yao: u8,
}

impl MeihuaIndex {
    /// 从卦象创建索引
    pub fn from_gua(shang: &SingleGua, xia: &SingleGua, dong_yao: u8) -> Self {
        Self {
            shang_gua: shang.number(),
            xia_gua: xia.number(),
            dong_yao,
        }
    }

    /// 验证索引有效性
    pub fn is_valid(&self) -> bool {
        self.shang_gua >= 1 && self.shang_gua <= 8 &&
        self.xia_gua >= 1 && self.xia_gua <= 8 &&
        self.dong_yao >= 1 && self.dong_yao <= 6
    }

    /// 获取六十四卦索引 (0-63)
    pub fn hexagram_index(&self) -> u8 {
        let shang_idx = if self.shang_gua == 8 { 0 } else { self.shang_gua };
        let xia_idx = if self.xia_gua == 8 { 0 } else { self.xia_gua };
        shang_idx * 8 + xia_idx
    }

    /// 判断体卦位置
    /// 规则：动爻在哪卦，哪卦为用，另一卦为体
    pub fn ti_is_shang(&self) -> bool {
        self.dong_yao <= 3 // 动爻1-3在下卦，上卦为体
    }
}

// ==================== 梅花计算结果 ====================

/// 梅花计算结果（简化版，用于 JSON 清单）
#[derive(Clone, Debug, Encode, Decode)]
pub struct MeihuaComputeResult {
    /// 卦象索引
    pub index: MeihuaIndex,
    /// 本卦六十四卦索引
    pub ben_gua_index: u8,
    /// 变卦上卦
    pub bian_shang: u8,
    /// 变卦下卦
    pub bian_xia: u8,
    /// 互卦上卦
    pub hu_shang: u8,
    /// 互卦下卦
    pub hu_xia: u8,
    /// 本卦体用关系
    pub ben_relation: TiYongRelation,
    /// 变卦体用关系
    pub bian_relation: TiYongRelation,
    /// 综合吉凶
    pub fortune: Fortune,
    /// 体卦五行
    pub ti_wuxing: WuXing,
    /// 用卦五行
    pub yong_wuxing: WuXing,
    /// 体卦旺衰（如果有季节信息）
    pub ti_wangshuai: Option<WangShuai>,
}

// ==================== JSON 清单结构 ====================

/// 梅花 JSON 清单（用于 IPFS 存储）
#[derive(Clone, Debug, Encode, Decode)]
pub struct MeihuaManifest {
    /// 版本号
    pub version: u32,
    /// 模块 ID
    pub module_id: u8,
    /// 隐私模式
    pub privacy_mode: u8,
    /// 创建时间戳（Unix 秒）
    pub created_at: u64,

    // ===== 公开数据（Public/Encrypted 模式）=====
    /// 卦象索引
    pub index: Option<MeihuaIndex>,
    /// 起卦方式
    pub method: Option<u8>,
    /// 本卦六十四卦索引
    pub ben_gua_index: Option<u8>,
    /// 变卦
    pub bian_gua: Option<(u8, u8)>,
    /// 互卦
    pub hu_gua: Option<(u8, u8)>,
    /// 本卦体用关系
    pub ben_relation: Option<u8>,
    /// 变卦体用关系
    pub bian_relation: Option<u8>,
    /// 综合吉凶
    pub fortune: Option<u8>,
    /// 体卦五行
    pub ti_wuxing: Option<u8>,
    /// 用卦五行
    pub yong_wuxing: Option<u8>,

    // ===== 敏感数据（仅 Public 模式明文）=====
    /// 起卦时间（仅 Public 模式）
    pub divination_time: Option<DivinationTimeJson>,
    /// 占问事宜（仅 Public 模式）
    pub question: Option<Vec<u8>>,
    /// 性别
    pub gender: Option<u8>,
    /// 出生年份
    pub birth_year: Option<u16>,

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

/// 梅花模块 OCW + TEE 处理器
pub struct MeihuaModuleHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: crate::pallet::Config> DivinationModule<T> for MeihuaModuleHandler<T> {
    const MODULE_ID: DivinationType = DivinationType::Meihua;
    const MODULE_NAME: &'static str = "MeiHua";
    const VERSION: u32 = 1;

    type PlainInput = MeihuaInputPlain;
    type Index = MeihuaIndex;
    type Result = MeihuaComputeResult;

    /// 执行梅花计算
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError> {
        // 验证输入
        <Self as DivinationModule<T>>::validate_input(input)?;

        // 根据起卦方式计算上卦、下卦、动爻
        let (shang_gua_num, xia_gua_num, dong_yao) = match input.method {
            DivinationMethod::LunarDateTime | DivinationMethod::GregorianDateTime => {
                // 时间起卦
                let year = input.year.unwrap_or(2024);
                let month = input.month.unwrap_or(1);
                let day = input.day.unwrap_or(1);
                let hour = input.hour.unwrap_or(0);
                
                // 简化的时间起卦算法
                let shang = ((year % 8) + (month as u16)) % 8;
                let xia = ((year % 8) + (month as u16) + (day as u16)) % 8;
                let dong = ((year % 6) + (month as u16) + (day as u16) + (hour as u16)) % 6 + 1;
                
                (if shang == 0 { 8 } else { shang as u8 }, 
                 if xia == 0 { 8 } else { xia as u8 }, 
                 dong as u8)
            }
            DivinationMethod::TwoNumbers => {
                let num1 = input.number1.unwrap_or(1);
                let num2 = input.number2.unwrap_or(1);
                let hour = input.hour.unwrap_or(0);
                
                let shang = (num1 % 8) as u8;
                let xia = (num2 % 8) as u8;
                let dong = ((num1 + num2 + hour as u32) % 6 + 1) as u8;
                
                (if shang == 0 { 8 } else { shang }, 
                 if xia == 0 { 8 } else { xia }, 
                 dong)
            }
            DivinationMethod::SingleNumber => {
                let num = input.number1.unwrap_or(1);
                let shang = ((num / 10) % 8) as u8;
                let xia = (num % 8) as u8;
                let dong = (num % 6 + 1) as u8;
                
                (if shang == 0 { 8 } else { shang }, 
                 if xia == 0 { 8 } else { xia }, 
                 dong)
            }
            DivinationMethod::Manual => {
                (input.shang_gua.unwrap_or(1),
                 input.xia_gua.unwrap_or(1),
                 input.dong_yao.unwrap_or(1))
            }
            DivinationMethod::Random | DivinationMethod::ChainShake => {
                // 随机起卦（简化）
                (1, 2, 3)
            }
        };

        // 计算变卦
        let (bian_shang, bian_xia) = if dong_yao <= 3 {
            // 动爻在下卦
            let new_xia = xia_gua_num ^ (1 << (dong_yao - 1));
            (shang_gua_num, if new_xia == 0 { 8 } else { new_xia })
        } else {
            // 动爻在上卦
            let new_shang = shang_gua_num ^ (1 << (dong_yao - 4));
            (if new_shang == 0 { 8 } else { new_shang }, xia_gua_num)
        };

        // 计算互卦
        let hu_shang = ((shang_gua_num & 0b110) >> 1) | ((xia_gua_num & 0b100) >> 2);
        let hu_xia = ((shang_gua_num & 0b001) << 2) | ((xia_gua_num & 0b110) >> 1);

        // 计算六十四卦索引
        let ben_gua_index = (shang_gua_num - 1) * 8 + (xia_gua_num - 1);

        // 简化的体用关系和吉凶判断
        let ti_wuxing = WuXing::from_gua_num(if dong_yao <= 3 { shang_gua_num } else { xia_gua_num });
        let yong_wuxing = WuXing::from_gua_num(if dong_yao <= 3 { xia_gua_num } else { shang_gua_num });
        let ben_relation = TiYongRelation::calculate(&ti_wuxing, &yong_wuxing);
        let bian_relation = TiYongRelation::BiHe;
        let fortune = Fortune::from_relations(&ben_relation, Some(&bian_relation));
        let ti_wangshuai = Some(WangShuai::default());

        Ok(MeihuaComputeResult {
            index: MeihuaIndex {
                shang_gua: shang_gua_num,
                xia_gua: xia_gua_num,
                dong_yao,
            },
            ben_gua_index,
            bian_shang,
            bian_xia,
            hu_shang,
            hu_xia,
            ben_relation,
            bian_relation,
            fortune,
            ti_wuxing,
            yong_wuxing,
            ti_wangshuai,
        })
    }

    /// 从计算结果提取索引
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Public | PrivacyMode::PublicEncrypted | PrivacyMode::Encrypted => Some(result.index),
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
                let divination_time = if let (Some(year), Some(month), Some(day), Some(hour)) =
                    (input.year, input.month, input.day, input.hour)
                {
                    Some(DivinationTimeJson {
                        year,
                        month,
                        day,
                        hour,
                        minute: input.minute.unwrap_or(0),
                    })
                } else {
                    None
                };

                MeihuaManifest {
                    version: Self::VERSION,
                    module_id: Self::MODULE_ID as u8,
                    privacy_mode: privacy_mode as u8,
                    created_at: 0,
                    index: Some(result.index),
                    method: Some(input.method as u8),
                    ben_gua_index: Some(result.ben_gua_index),
                    bian_gua: Some((result.bian_shang, result.bian_xia)),
                    hu_gua: Some((result.hu_shang, result.hu_xia)),
                    ben_relation: Some(result.ben_relation as u8),
                    bian_relation: Some(result.bian_relation as u8),
                    fortune: Some(result.fortune as u8),
                    ti_wuxing: Some(result.ti_wuxing as u8),
                    yong_wuxing: Some(result.yong_wuxing as u8),
                    divination_time,
                    question: input.question.as_ref().map(|q| q.to_vec()),
                    gender: Some(input.gender),
                    birth_year: input.birth_year,
                    encrypted_sensitive: None,
                    sensitive_hash: None,
                }
            }
            PrivacyMode::Encrypted => {
                // 加密模式：索引和计算数据明文，敏感数据加密
                MeihuaManifest {
                    version: Self::VERSION,
                    module_id: Self::MODULE_ID as u8,
                    privacy_mode: privacy_mode as u8,
                    created_at: 0,
                    index: Some(result.index),
                    method: Some(input.method as u8),
                    ben_gua_index: Some(result.ben_gua_index),
                    bian_gua: Some((result.bian_shang, result.bian_xia)),
                    hu_gua: Some((result.hu_shang, result.hu_xia)),
                    ben_relation: Some(result.ben_relation as u8),
                    bian_relation: Some(result.bian_relation as u8),
                    fortune: Some(result.fortune as u8),
                    ti_wuxing: Some(result.ti_wuxing as u8),
                    yong_wuxing: Some(result.yong_wuxing as u8),
                    divination_time: None, // 加密
                    question: None,        // 加密（最敏感）
                    gender: None,          // 加密
                    birth_year: None,      // 加密
                    encrypted_sensitive: None, // TODO: 由 TEE 填充
                    sensitive_hash: None,
                }
            }
            PrivacyMode::Private => {
                // 私密模式：所有数据加密
                MeihuaManifest {
                    version: Self::VERSION,
                    module_id: Self::MODULE_ID as u8,
                    privacy_mode: privacy_mode as u8,
                    created_at: 0,
                    index: None,
                    method: None,
                    ben_gua_index: None,
                    bian_gua: None,
                    hu_gua: None,
                    ben_relation: None,
                    bian_relation: None,
                    fortune: None,
                    ti_wuxing: None,
                    yong_wuxing: None,
                    divination_time: None,
                    question: None,
                    gender: None,
                    birth_year: None,
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
            return Err(ModuleError::invalid_input(b"Invalid MeiHua input"));
        }
        Ok(())
    }

    /// 获取推荐超时时间（梅花计算较简单）
    fn recommended_timeout() -> u32 {
        60 // ~6 分钟
    }

    /// 获取最大输入大小
    fn max_input_size() -> u32 {
        256
    }

    /// 是否支持批量处理
    fn supports_batch() -> bool {
        true // 梅花计算简单，可以批量
    }

    /// 获取 TEE 端点路径
    fn tee_endpoint() -> &'static str {
        "/compute/meihua"
    }
}

// ==================== 辅助函数 ====================

/// 从 FullDivination 创建 ProcessResult
pub fn create_process_result<AccountId, BlockNumber>(
    divination: &crate::types::FullDivination<AccountId, BlockNumber>,
    manifest_cid: Vec<u8>,
    manifest_hash: [u8; 32],
) -> ProcessResult {
    let index = MeihuaIndex::from_gua(
        &divination.ben_gua.shang_gua,
        &divination.ben_gua.xia_gua,
        divination.ben_gua.dong_yao,
    );

    ProcessResult {
        manifest_cid,
        manifest_hash,
        type_index: Some(index.encode()),
        proof: None,
        manifest_data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meihua_input_validation() {
        // 有效的时间起卦输入
        let valid_time = MeihuaInputPlain {
            method: DivinationMethod::LunarDateTime,
            year: Some(2024),
            month: Some(1),
            day: Some(15),
            hour: Some(10),
            minute: Some(30),
            number1: None,
            number2: None,
            shang_gua: None,
            xia_gua: None,
            dong_yao: None,
            gender: 1,
            birth_year: None,
            question: None,
        };
        assert!(valid_time.is_valid());

        // 有效的双数起卦输入
        let valid_numbers = MeihuaInputPlain {
            method: DivinationMethod::TwoNumbers,
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            number1: Some(123),
            number2: Some(456),
            shang_gua: None,
            xia_gua: None,
            dong_yao: None,
            gender: 0,
            birth_year: None,
            question: None,
        };
        assert!(valid_numbers.is_valid());

        // 有效的手动指定输入
        let valid_manual = MeihuaInputPlain {
            method: DivinationMethod::Manual,
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            number1: None,
            number2: None,
            shang_gua: Some(1), // 乾
            xia_gua: Some(8),   // 坤
            dong_yao: Some(3),
            gender: 2,
            birth_year: Some(1990),
            question: None,
        };
        assert!(valid_manual.is_valid());

        // 无效的时间起卦（缺少时间）
        let invalid_time = MeihuaInputPlain {
            method: DivinationMethod::LunarDateTime,
            year: Some(2024),
            month: None, // 缺少月份
            day: Some(15),
            hour: Some(10),
            minute: None,
            number1: None,
            number2: None,
            shang_gua: None,
            xia_gua: None,
            dong_yao: None,
            gender: 0,
            birth_year: None,
            question: None,
        };
        assert!(!invalid_time.is_valid());
    }

    #[test]
    fn test_meihua_index() {
        let index = MeihuaIndex {
            shang_gua: 1, // 乾
            xia_gua: 8,   // 坤
            dong_yao: 3,
        };
        assert!(index.is_valid());
        assert!(index.ti_is_shang()); // 动爻3在下卦，上卦为体

        let index2 = MeihuaIndex {
            shang_gua: 1,
            xia_gua: 8,
            dong_yao: 5,
        };
        assert!(!index2.ti_is_shang()); // 动爻5在上卦，下卦为体
    }

    #[test]
    fn test_hexagram_index() {
        // 乾上坤下 = 天地否
        let index = MeihuaIndex {
            shang_gua: 1, // 乾
            xia_gua: 8,   // 坤 -> 索引0
            dong_yao: 1,
        };
        assert_eq!(index.hexagram_index(), 1 * 8 + 0); // = 8
    }
}
