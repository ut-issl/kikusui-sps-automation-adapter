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

    /// クエリを送信してf64を返す共通ヘルパー
    fn query_f64(&mut self, cmd: &[u8]) -> Result<f64, PowerSupplyError> {
        self.stream.write_all(cmd)?;
        let mut buffer = [0u8; 100];
        let n = self.stream.read(&mut buffer)?;
        let s = std::str::from_utf8(&buffer[0..n])
            .map_err(|e| PowerSupplyError::ParseError(e.to_string()))?
            .trim();
        s.parse::<f64>()
            .map_err(|e| PowerSupplyError::ParseError(format!("Failed to parse '{}': {}", s, e)))
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
        self.query_f64(b"MEAS:VOLT?\r\n")
    }

    /// 電流を測定
    pub fn measure_current(&mut self) -> Result<f64, PowerSupplyError> {
        self.query_f64(b"MEAS:CURR?\r\n")
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

    /// 電圧設定値をセット
    pub fn set_voltage(&mut self, v: f64) -> Result<(), PowerSupplyError> {
        let cmd = format!("VOLT {:.4}\r\n", v);
        self.stream.write_all(cmd.as_bytes())?;
        Ok(())
    }

    /// 電流制限値をセット
    pub fn set_current(&mut self, a: f64) -> Result<(), PowerSupplyError> {
        let cmd = format!("CURR {:.4}\r\n", a);
        self.stream.write_all(cmd.as_bytes())?;
        Ok(())
    }

    /// OVP（過電圧保護）設定値をセット
    pub fn set_ov(&mut self, v: f64) -> Result<(), PowerSupplyError> {
        let cmd = format!("VOLT:PROT {:.4}\r\n", v);
        self.stream.write_all(cmd.as_bytes())?;
        Ok(())
    }

    /// OCP（過電流保護）設定値をセット
    pub fn set_oc(&mut self, a: f64) -> Result<(), PowerSupplyError> {
        let cmd = format!("CURR:PROT {:.4}\r\n", a);
        self.stream.write_all(cmd.as_bytes())?;
        Ok(())
    }

    /// 電圧設定値を読み取る
    pub fn get_set_voltage(&mut self) -> Result<f64, PowerSupplyError> {
        self.query_f64(b"VOLT?\r\n")
    }

    /// 電流制限値を読み取る
    pub fn get_set_current(&mut self) -> Result<f64, PowerSupplyError> {
        self.query_f64(b"CURR?\r\n")
    }

    /// OVP（過電圧保護）設定値を読み取る
    pub fn get_ov(&mut self) -> Result<f64, PowerSupplyError> {
        self.query_f64(b"VOLT:PROT?\r\n")
    }

    /// OCP（過電流保護）設定値を読み取る
    pub fn get_oc(&mut self) -> Result<f64, PowerSupplyError> {
        self.query_f64(b"CURR:PROT?\r\n")
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

    /// テスト対象の電源設定（環境に合わせて変更）
    const TEST_HOST: &str = "169.254.9.175";
    const TEST_PORT: u16 = 5025;
    const TEST_ID: &str = "PSU1";

    fn connect() -> PowerSupply {
        PowerSupply::new(TEST_ID, TEST_HOST, TEST_PORT)
            .expect("電源への接続に失敗しました。ホスト・ポートを確認してください。")
    }

    // --- 実機テスト ---
    #[test]
    fn test_hw_connection_and_device_id() {
        let psu = connect();
        assert_eq!(psu.get_id(), TEST_ID);
        // デバイスIDが空でないことを確認
        assert!(!psu.get_device_id().is_empty());
        println!("device_id: {}", psu.get_device_id());
    }

    #[test]
    fn test_hw_output_on_off() {
        let mut psu = connect();
        psu.output_on().expect("output_on failed");
        std::thread::sleep(std::time::Duration::from_millis(5000));
        psu.output_off().expect("output_off failed");
    }

    #[test]
    fn test_hw_measure_voltage() {
        let mut psu = connect();
        let v = psu.measure_voltage().expect("measure_voltage failed");
        println!("電圧: {} V", v);
    }

    #[test]
    fn test_hw_measure_current() {
        let mut psu = connect();
        let a = psu.measure_current().expect("measure_current failed");
        println!("電流: {} A", a);
    }

    #[test]
    fn test_hw_measure_voltage_and_current() {
        let mut psu = connect();
        let m = psu.measure().expect("measure failed");
        println!("測定値: {}V / {}A @ {}", m.voltage, m.current, m.timestamp);
        // ISO 8601 形式か確認
        assert!(m.timestamp.contains('T'));
    }

    #[test]
    fn test_hw_set_voltage() {
        let mut psu = connect();
        let set_voltage = 2.0;
        psu.set_voltage(set_voltage).expect("set_voltage failed");
        let v = psu.get_set_voltage().expect("get_set_voltage failed");
        println!("電圧設定値: {} V", v);
        assert!((v - set_voltage).abs() < 0.01, "電圧設定値が一致しない: {}", v);
    }

    #[test]
    fn test_hw_set_current() {
        let mut psu = connect();
        let set_current = 0.5;
        psu.set_current(set_current).expect("set_current failed");
        let a = psu.get_set_current().expect("get_set_current failed");
        println!("電流設定値: {} A", a);
        assert!((a - set_current).abs() < 0.001, "電流設定値が一致しない: {}", a);
    }

    #[test]
    fn test_hw_set_ov() {
        let set_ov = 10.0;
        let mut psu = connect();
        psu.set_ov(set_ov).expect("set_ov failed");
        let v = psu.get_ov().expect("get_ov failed");
        println!("OVP設定値: {} V", v);
        assert!((v - set_ov).abs() < 0.01, "OVP設定値が一致しない: {}", v);
    }

    #[test]
    fn test_hw_set_oc() {
        let set_oc = 3.0;
        let mut psu = connect();
        psu.set_oc(set_oc).expect("set_oc failed");
        let a = psu.get_oc().expect("get_oc failed");
        println!("OCP設定値: {} A", a);
        assert!((a - set_oc).abs() < 0.001, "OCP設定値が一致しない: {}", a);
    }

}
