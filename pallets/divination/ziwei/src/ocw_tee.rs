//! # 紫微斗数模块 OCW + TEE 集成
//!
//! 实现 `DivinationModule` trait，接入通用 OCW + TEE 架构。

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use pallet_divination_ocw_tee::{
    DivinationModule, DivinationType, ModuleError, PrivacyMode,
};
use scale_info::TypeInfo;

#[cfg(not(feature = "std"))]
use sp_std::prelude::*;
#[cfg(feature = "std")]
use std::prelude::v1::*;

// ==================== 紫微输入类型 ====================

/// 紫微起盘方式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ZiweiMethod {
    Time = 0,
    Manual = 1,
    Random = 2,
}

impl Default for ZiweiMethod {
    fn default() -> Self {
        Self::Time
    }
}

/// 紫微明文输入
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ZiweiInputPlain {
    /// 起盘方式
    pub method: ZiweiMethod,
    /// 出生年（农历）
    pub year: Option<u16>,
    /// 出生月（农历）
    pub month: Option<u8>,
    /// 出生日（农历）
    pub day: Option<u8>,
    /// 出生时辰（地支索引 0-11）
    pub hour: Option<u8>,
    /// 性别（0: 男, 1: 女）
    pub gender: Option<u8>,
    /// 是否闰月
    pub is_leap_month: Option<bool>,
    /// 占问事宜
    pub question: Option<BoundedVec<u8, ConstU32<128>>>,
}

impl ZiweiInputPlain {
    pub fn is_valid(&self) -> bool {
        match self.method {
            ZiweiMethod::Time | ZiweiMethod::Manual => {
                self.year.is_some() && self.month.is_some() && 
                self.day.is_some() && self.hour.is_some() && self.gender.is_some()
            }
            ZiweiMethod::Random => true,
        }
    }
}

// ==================== 紫微索引类型 ====================

/// 紫微索引
#[derive(Clone, Copy, Debug, Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ZiweiIndex {
    /// 命宫位置 (0-11)
    pub ming_gong: u8,
    /// 身宫位置 (0-11)
    pub shen_gong: u8,
    /// 紫微星位置 (0-11)
    pub ziwei_pos: u8,
    /// 天府星位置 (0-11)
    pub tianfu_pos: u8,
    /// 五行局 (1-6)
    pub wu_xing_ju: u8,
}

// ==================== 紫微计算结果 ====================

/// 紫微计算结果
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct ZiweiComputeResult {
    pub index: ZiweiIndex,
    /// 十二宫位置
    pub palaces: [u8; 12],
    /// 主星分布
    pub main_stars: Vec<(u8, u8)>, // (星曜ID, 宫位)
}

// ==================== DivinationModule 实现 ====================

pub struct ZiweiModuleHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: crate::pallet::Config> DivinationModule<T> for ZiweiModuleHandler<T> {
    const MODULE_ID: DivinationType = DivinationType::ZiWei;
    const MODULE_NAME: &'static str = "ZiWei";
    const VERSION: u32 = 1;

    type PlainInput = ZiweiInputPlain;
    type Index = ZiweiIndex;
    type Result = ZiweiComputeResult;

    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError> {
        <Self as DivinationModule<T>>::validate_input(input)?;

        // 简化实现：内联基本算法
        let _year = input.year.unwrap_or(2000);
        let month = input.month.unwrap_or(1);
        let day = input.day.unwrap_or(1);
        let hour = input.hour.unwrap_or(0);
        let _gender = input.gender.unwrap_or(0);

        // 简化的五行局计算（实际应更复杂）
        let wu_xing_ju = ((month + day) % 5 + 2) as u8; // 2-6
        
        // 命宫位置：寅宫起正月，逆数至生月，再顺数至生时
        let ming_gong = ((14 - month + hour) % 12) as u8;
        
        // 身宫位置：寅宫起正月，顺数至生月，再顺数至生时
        let shen_gong = ((month + hour + 1) % 12) as u8;
        
        // 紫微星位置（简化）
        let ziwei_pos = ((day / wu_xing_ju as u8) % 12) as u8;
        
        // 天府星位置（与紫微对称）
        let tianfu_pos = (12 - ziwei_pos) % 12;

        Ok(ZiweiComputeResult {
            index: ZiweiIndex {
                ming_gong,
                shen_gong,
                ziwei_pos,
                tianfu_pos,
                wu_xing_ju,
            },
            palaces: [0; 12],
            main_stars: Vec::new(),
        })
    }

    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Public | PrivacyMode::Encrypted => Some(result.index),
            PrivacyMode::Private => None,
        }
    }

    fn generate_manifest(
        _input: &Self::PlainInput,
        result: &Self::Result,
        privacy_mode: PrivacyMode,
    ) -> Result<Vec<u8>, ModuleError> {
        let manifest = (
            Self::VERSION,
            Self::MODULE_ID as u8,
            privacy_mode as u8,
            &result.index,
        );
        Ok(manifest.encode())
    }

    fn validate_input(input: &Self::PlainInput) -> Result<(), ModuleError> {
        if !input.is_valid() {
            return Err(ModuleError::invalid_input(b"Invalid ZiWei input"));
        }
        Ok(())
    }
}
