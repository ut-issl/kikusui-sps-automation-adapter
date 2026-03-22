# kikusui-sps-adapter

菊水電子工業のSPSシリーズ安定化電源をTCP/IP経由で制御するRustライブラリ

## 機能

- TCP/IP経由での電源接続
- 出力ON/OFF制御
- 電圧・電流の測定
- エラーハンドリング
- オプションのCLIツール

## ライブラリとして使用

```rust
use kikusui_sps_adapter::PowerSupply;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 電源に接続
    let mut psu = PowerSupply::new("PSU1", "192.168.1.100", 8462)?;

    // デバイス情報を表示
    println!("接続先: {}", psu.get_device_id());

    // 出力をON
    psu.output_on()?;

    // 電圧と電流を測定
    let measurement = psu.measure()?;
    println!("電圧: {} V", measurement.voltage);
    println!("電流: {} A", measurement.current);
    println!("時刻: {}", measurement.timestamp);

    // 出力をOFF
    psu.output_off()?;

    Ok(())
}
```

## CLIツールとして使用

CLIツールをビルド:
```bash
cargo build --features cli
```

実行:
```bash
./target/debug/kikusui-sps-cli --host 192.168.1.100 --port 8462 --freq 1.0 --log-file test
```

オプション:
- `--host`: ホスト名またはIPアドレス
- `--port`: ポート番号
- `--freq`: 測定周波数 [Hz] (デフォルト: 1.0)
- `--log-file`: ログファイル名（オプション）

## API

### PowerSupply

#### `new(id: &str, host: &str, port: u16) -> Result<Self, PowerSupplyError>`
新しい電源接続を作成します。

#### `output_on() -> Result<(), PowerSupplyError>`
出力をONにします。

#### `output_off() -> Result<(), PowerSupplyError>`
出力をOFFにします。

#### `measure_voltage() -> Result<f64, PowerSupplyError>`
電圧を測定します。

#### `measure_current() -> Result<f64, PowerSupplyError>`
電流を測定します。

#### `measure() -> Result<Measurement, PowerSupplyError>`
電圧と電流を同時に測定します。

### Measurement

測定値を表す構造体:
```rust
pub struct Measurement {
    pub voltage: f64,     // 電圧 [V]
    pub current: f64,     // 電流 [A]
    pub timestamp: String, // ISO 8601形式のタイムスタンプ
}
```

### PowerSupplyError

エラー型:
- `ConnectionError`: 接続エラー
- `CommunicationError`: 通信エラー
- `ParseError`: パースエラー
- `Other`: その他のエラー
