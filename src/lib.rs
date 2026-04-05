//! Kikusui SPS 安定化電源制御ライブラリ
//!
//! このライブラリは菊水電子工業のSPSシリーズ安定化電源をTCP/IP経由で制御します。

pub mod power_supply;

pub use power_supply::PowerSupply;
pub use power_supply::PowerSupplyError;
