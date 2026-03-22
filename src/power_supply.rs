use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// 電源の測定値
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// 電圧 [V]
    pub voltage: f64,
    /// 電流 [A]
    pub current: f64,
    /// タイムスタンプ (ISO 8601形式)
    pub timestamp: String,
}

/// 電源制御エラー
#[derive(Debug)]
pub enum PowerSupplyError {
    /// 接続エラー
    ConnectionError(std::io::Error),
    /// 通信エラー
    CommunicationError(std::io::Error),
    /// パースエラー
    ParseError(String),
    /// その他のエラー
    Other(String),
}

impl std::fmt::Display for PowerSupplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerSupplyError::ConnectionError(e) => write!(f, "Connection error: {}", e),
            PowerSupplyError::CommunicationError(e) => write!(f, "Communication error: {}", e),
            PowerSupplyError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            PowerSupplyError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for PowerSupplyError {}

impl From<std::io::Error> for PowerSupplyError {
    fn from(err: std::io::Error) -> Self {
        PowerSupplyError::CommunicationError(err)
    }
}

/// Kikusui SPS 安定化電源
pub struct PowerSupply {
    /// 識別ID（管理用）
    pub id: String,
    /// TCPストリーム
    stream: TcpStream,
    /// デバイス識別情報
    pub device_id: String,
}

impl PowerSupply {
    /// 新しい電源接続を作成
    ///
    /// # Arguments
    /// * `id` - この電源の識別ID（例: "PSU1", "PSU2"）
    /// * `host` - ホスト名またはIPアドレス
    /// * `port` - ポート番号
    ///
    pub fn new(id: &str, host: &str, port: u16) -> Result<Self, PowerSupplyError> {
        let mut stream = TcpStream::connect((host, port))
            .map_err(PowerSupplyError::ConnectionError)?;

        // タイムアウト設定
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        // デバイスIDを取得
        stream.write_all(b"*IDN?\r\n")?;
        let mut buffer = [0u8; 256];
        let n = stream.read(&mut buffer)?;
        let device_id = String::from_utf8_lossy(&buffer[0..n])
            .trim()
            .to_string();

        Ok(PowerSupply {
            id: id.to_string(),
            stream,
            device_id,
        })
    }

    /// 出力をONにする
    pub fn output_on(&mut self) -> Result<(), PowerSupplyError> {
        self.stream.write_all(b"OUTP 1\r\n")?;
        Ok(())
    }

    /// 出力をOFFにする
    pub fn output_off(&mut self) -> Result<(), PowerSupplyError> {
        self.stream.write_all(b"OUTP 0\r\n")?;
        Ok(())
    }

    /// 電圧を測定
    pub fn measure_voltage(&mut self) -> Result<f64, PowerSupplyError> {
        self.stream.write_all(b"MEAS:VOLT?\r\n")?;
        let mut buffer = [0u8; 100];
        let n = self.stream.read(&mut buffer)?;

        let voltage_str = std::str::from_utf8(&buffer[0..n])
            .map_err(|e| PowerSupplyError::ParseError(e.to_string()))?
            .trim();

        voltage_str.parse::<f64>()
            .map_err(|e| PowerSupplyError::ParseError(format!("Failed to parse voltage: {}", e)))
    }

    /// 電流を測定
    pub fn measure_current(&mut self) -> Result<f64, PowerSupplyError> {
        self.stream.write_all(b"MEAS:CURR?\r\n")?;
        let mut buffer = [0u8; 100];
        let n = self.stream.read(&mut buffer)?;

        let current_str = std::str::from_utf8(&buffer[0..n])
            .map_err(|e| PowerSupplyError::ParseError(e.to_string()))?
            .trim();

        current_str.parse::<f64>()
            .map_err(|e| PowerSupplyError::ParseError(format!("Failed to parse current: {}", e)))
    }

    /// 電圧と電流を測定
    pub fn measure(&mut self) -> Result<Measurement, PowerSupplyError> {
        use chrono::{Utc, SecondsFormat};

        let voltage = self.measure_voltage()?;
        let current = self.measure_current()?;
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

        Ok(Measurement {
            voltage,
            current,
            timestamp,
        })
    }

    /// デバイスIDを取得
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }

    /// 識別IDを取得
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 実際のハードウェアが必要なため、テストは手動で実行
    #[test]
    #[ignore]
    fn test_connection() {
        let result = PowerSupply::new("TEST", "192.168.1.100", 8462);
        assert!(result.is_ok());
    }
}
