//! # 八字排盘计算模块
//!
//! 本模块包含所有八字排盘相关的计算逻辑，包括：
//! - 干支计算：干支索引转换、序列计算
//! - 四柱计算：年月日时柱的完整计算逻辑
//! - 大运计算：起运年龄、大运序列
//! - 五行计算：五行强度、喜用神分析
//! - 节气计算：精确节气时间（寿星天文历算法）
//! - 神煞计算：天乙贵人、桃花、羊刃等神煞
//! - 空亡计算：六十甲子旬空查询
//! - 星运计算：十二长生状态
//! - 刑冲计算：地支刑冲合会关系
//! - 真太阳时：经度时差 + 时差方程修正

pub mod ganzhi;
pub mod sizhu;
pub mod dayun;
pub mod wuxing;
pub mod jieqi;
pub mod shensha;
pub mod kongwang;
pub mod xingyun;
pub mod xingchong;
pub mod true_solar_time;

// 重新导出核心函数
pub use ganzhi::*;
pub use sizhu::{calculate_day_ganzhi, calculate_year_ganzhi, calculate_month_ganzhi, calculate_month_ganzhi_with_hour, calculate_hour_ganzhi};
pub use dayun::{calculate_qiyun_age, calculate_dayun_list, calculate_dayun_shishen};
pub use wuxing::{calculate_wuxing_strength, determine_xiyong_shen};
pub use jieqi::{get_month_zhi_by_jieqi, get_jieqi_time, calculate_year_jieqi, JieQiTime};
// 神煞、空亡、星运、刑冲模块导出
pub use shensha::{ShenSha, calculate_shensha_list, calculate_shensha_list_temp};
pub use kongwang::{calculate_kongwang, calculate_all_kongwang, calculate_all_kongwang_temp};
pub use xingyun::{get_changsheng, calculate_xingyun, calculate_xingyun_temp};
pub use xingchong::{DiZhiGuanXi, TianGanGuanXi, analyze_sizhu_guanxi, SiZhuGuanXi};
// 真太阳时模块导出
pub use true_solar_time::{apply_true_solar_time, should_apply_correction, adjust_date, TrueSolarTimeResult};
