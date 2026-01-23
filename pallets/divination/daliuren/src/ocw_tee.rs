//! # 大六壬模块 OCW + TEE 集成
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

// ==================== 大六壬输入类型 ====================

/// 大六壬起课方式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum DaliurenMethod {
    Time = 0,
    SolarTime = 1,
    Random = 2,
    Manual = 3,
}

impl Default for DaliurenMethod {
    fn default() -> Self {
        Self::Time
    }
}

/// 大六壬明文输入
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DaliurenInputPlain {
    /// 起课方式
    pub method: DaliurenMethod,
    /// 年（农历）
    pub year: Option<u16>,
    /// 月（农历）
    pub month: Option<u8>,
    /// 日（农历）
    pub day: Option<u8>,
    /// 时辰（地支索引 0-11）
    pub hour: Option<u8>,
    /// 日干索引 (0-9)
    pub day_gan: Option<u8>,
    /// 日支索引 (0-11)
    pub day_zhi: Option<u8>,
    /// 月将索引 (0-11)
    pub yue_jiang: Option<u8>,
    /// 占问事宜
    pub question: Option<BoundedVec<u8, ConstU32<128>>>,
}

impl DaliurenInputPlain {
    pub fn is_valid(&self) -> bool {
        match self.method {
            DaliurenMethod::Time | DaliurenMethod::SolarTime => {
                self.year.is_some() && self.month.is_some() && 
                self.day.is_some() && self.hour.is_some()
            }
            DaliurenMethod::Manual => {
                self.day_gan.is_some() && self.day_zhi.is_some() && 
                self.hour.is_some() && self.yue_jiang.is_some()
            }
            DaliurenMethod::Random => true,
        }
    }
}

// ==================== 大六壬索引类型 ====================

/// 大六壬索引
#[derive(Clone, Copy, Debug, Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DaliurenIndex {
    /// 日干索引 (0-9)
    pub day_gan: u8,
    /// 日支索引 (0-11)
    pub day_zhi: u8,
    /// 月将索引 (0-11)
    pub yue_jiang: u8,
    /// 时辰索引 (0-11)
    pub hour_zhi: u8,
}

// ==================== 大六壬计算结果 ====================

/// 大六壬计算结果
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct DaliurenComputeResult {
    pub index: DaliurenIndex,
    /// 天地盘（12宫位置）
    pub tian_pan: [u8; 12],
    pub di_pan: [u8; 12],
    /// 四课
    pub si_ke: [(u8, u8); 4],
    /// 三传
    pub san_chuan: [u8; 3],
}

// ==================== DivinationModule 实现 ====================

pub struct DaliurenModuleHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: crate::pallet::Config> DivinationModule<T> for DaliurenModuleHandler<T> {
    const MODULE_ID: DivinationType = DivinationType::DaLiuRen;
    const MODULE_NAME: &'static str = "DaLiuRen";
    const VERSION: u32 = 1;

    type PlainInput = DaliurenInputPlain;
    type Index = DaliurenIndex;
    type Result = DaliurenComputeResult;

    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError> {
        <Self as DivinationModule<T>>::validate_input(input)?;

        let day_gan = input.day_gan.unwrap_or(0);
        let day_zhi = input.day_zhi.unwrap_or(0);
        let hour_zhi = input.hour.unwrap_or(0);
        let yue_jiang = input.yue_jiang.unwrap_or(0);

        // 简化的天地盘计算
        // 地盘固定：子丑寅卯辰巳午未申酉戌亥
        let di_pan: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        
        // 天盘：月将加占时，顺时针旋转
        let offset = (yue_jiang + 12 - hour_zhi) % 12;
        let mut tian_pan = [0u8; 12];
        for i in 0..12 {
            tian_pan[i] = (i as u8 + offset) % 12;
        }
        
        // 简化的四课计算
        let si_ke: [(u8, u8); 4] = [
            (day_gan, tian_pan[day_zhi as usize % 12]),
            (tian_pan[day_zhi as usize % 12], tian_pan[tian_pan[day_zhi as usize % 12] as usize % 12]),
            (day_zhi, tian_pan[day_zhi as usize % 12]),
            (tian_pan[day_zhi as usize % 12], tian_pan[tian_pan[day_zhi as usize % 12] as usize % 12]),
        ];
        
        // 简化的三传计算
        let san_chuan: [u8; 3] = [si_ke[0].1, si_ke[1].1, si_ke[2].1];

        Ok(DaliurenComputeResult {
            index: DaliurenIndex {
                day_gan,
                day_zhi,
                yue_jiang,
                hour_zhi,
            },
            tian_pan,
            di_pan,
            si_ke,
            san_chuan,
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
            return Err(ModuleError::invalid_input(b"Invalid DaLiuRen input"));
        }
        Ok(())
    }
}
