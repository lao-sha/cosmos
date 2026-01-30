//! # 小六壬模块 OCW + TEE 集成
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

// ==================== 小六壬输入类型 ====================

/// 小六壬起课方式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum XiaoliurenMethod {
    Time = 0,
    Number = 1,
    Random = 2,
    Manual = 3,
}

impl Default for XiaoliurenMethod {
    fn default() -> Self {
        Self::Time
    }
}

/// 小六壬明文输入
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct XiaoliurenInputPlain {
    /// 起课方式
    pub method: XiaoliurenMethod,
    /// 农历月
    pub month: Option<u8>,
    /// 农历日
    pub day: Option<u8>,
    /// 时辰（地支索引 0-11）
    pub hour: Option<u8>,
    /// 数字起课用的三个数字
    pub numbers: Option<[u8; 3]>,
    /// 手动指定的三宫
    pub gongs: Option<[u8; 3]>,
    /// 占问事宜
    pub question: Option<BoundedVec<u8, ConstU32<128>>>,
}

impl XiaoliurenInputPlain {
    pub fn is_valid(&self) -> bool {
        match self.method {
            XiaoliurenMethod::Time => {
                self.month.is_some() && self.day.is_some() && self.hour.is_some()
            }
            XiaoliurenMethod::Number => self.numbers.is_some(),
            XiaoliurenMethod::Manual => self.gongs.is_some(),
            XiaoliurenMethod::Random => true,
        }
    }
}

// ==================== 小六壬索引类型 ====================

/// 小六壬索引
#[derive(Clone, Copy, Debug, Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct XiaoliurenIndex {
    /// 月宫 (0-5)
    pub month_gong: u8,
    /// 日宫 (0-5)
    pub day_gong: u8,
    /// 时宫 (0-5)
    pub hour_gong: u8,
}

// ==================== 小六壬计算结果 ====================

/// 小六壬计算结果
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct XiaoliurenComputeResult {
    pub index: XiaoliurenIndex,
    /// 月宫名称索引
    pub month_gong: u8,
    /// 日宫名称索引
    pub day_gong: u8,
    /// 时宫名称索引（最终结果）
    pub hour_gong: u8,
}

// ==================== DivinationModule 实现 ====================

pub struct XiaoliurenModuleHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: crate::pallet::Config> DivinationModule<T> for XiaoliurenModuleHandler<T> {
    const MODULE_ID: DivinationType = DivinationType::XiaoLiuRen;
    const MODULE_NAME: &'static str = "XiaoLiuRen";
    const VERSION: u32 = 1;

    type PlainInput = XiaoliurenInputPlain;
    type Index = XiaoliurenIndex;
    type Result = XiaoliurenComputeResult;

    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError> {
        <Self as DivinationModule<T>>::validate_input(input)?;

        let (month_gong, day_gong, hour_gong) = match input.method {
            XiaoliurenMethod::Time => {
                let m = input.month.unwrap_or(1);
                let d = input.day.unwrap_or(1);
                let h = input.hour.unwrap_or(0);
                // 小六壬算法：月宫从大安起，日宫从月宫起，时宫从日宫起
                let month_g = (m - 1) % 6;
                let day_g = (month_g + d - 1) % 6;
                let hour_g = (day_g + h) % 6;
                (month_g, day_g, hour_g)
            }
            XiaoliurenMethod::Number => {
                let nums = input.numbers.unwrap_or([1, 1, 1]);
                // 数字起课法
                let month_g = (nums[0].saturating_sub(1)) % 6;
                let day_g = (nums[0] + nums[1]).saturating_sub(2) % 6;
                let hour_g = (nums[0] + nums[1] + nums[2]).saturating_sub(3) % 6;
                (month_g, day_g, hour_g)
            }
            XiaoliurenMethod::Manual => {
                let gongs = input.gongs.unwrap_or([0, 0, 0]);
                (gongs[0] % 6, gongs[1] % 6, gongs[2] % 6)
            }
            XiaoliurenMethod::Random => {
                // 随机生成（实际应由随机源提供）
                (0, 2, 4)
            }
        };

        Ok(XiaoliurenComputeResult {
            index: XiaoliurenIndex {
                month_gong,
                day_gong,
                hour_gong,
            },
            month_gong,
            day_gong,
            hour_gong,
        })
    }

    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Public | PrivacyMode::PublicEncrypted | PrivacyMode::Encrypted => Some(result.index),
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
            return Err(ModuleError::invalid_input(b"Invalid XiaoLiuRen input"));
        }
        Ok(())
    }
}
