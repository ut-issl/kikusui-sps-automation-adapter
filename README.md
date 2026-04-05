# kikusui-sps-adapter

菊水電子工業のSPSシリーズ安定化電源をTCP/IP経由で制御するRustライブラリ

## 機能

- TCP/IP経由での電源接続
- 出力ON/OFF制御
- 電圧・電流の設定・読み取り
- OVP（過電圧保護）・OCP（過電流保護）の設定・読み取り
- 電圧・電流の測定
- エラーハンドリング

## 使い方

本ライブラリを`Cargo.toml` に追加:

```toml
[dependencies]
kikusui-sps-adapter = { git = "https://github.com/ut-issl/kikusui-sps-automation-adapter", rev = "コミットハッシュ" }
```

```rust
use kikusui_sps_adapter::PowerSupply;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 電源に接続（接続時に *IDN? を送信してデバイスIDを取得）
    let mut psu = PowerSupply::new("PSU1", "192.168.1.100", 8462)?;
    println!("接続先: {}", psu.get_device_id());

    // 電圧・電流・保護値を設定
    psu.set_voltage(5.0)?;
    psu.set_current(1.0)?;
    psu.set_ov(6.0)?;
    psu.set_oc(1.5)?;

    // 出力をON
    psu.output_on()?;

    // 電圧・電流を測定
    let v = psu.measure_voltage()?;
    let a = psu.measure_current()?;
    println!("電圧: {} V", v);
    println!("電流: {} A", a);

    // 出力をOFF
    psu.output_off()?;

    Ok(())
}
```

## API

### `PowerSupply`

| メソッド | 説明 |
|---|---|
| `new(id, host, port)` | 接続を確立し、デバイスIDを取得 |
| `output_on()` | 出力ON |
| `output_off()` | 出力OFF |
| `get_output_state()` | 出力状態を取得（`true`: ON, `false`: OFF） |
| `set_voltage(v)` | 電圧設定値をセット [V] |
| `set_current(a)` | 電流制限値をセット [A] |
| `set_ov(v)` | OVP設定値をセット [V] |
| `set_oc(a)` | OCP設定値をセット [A] |
| `get_set_voltage()` | 電圧設定値を読み取り [V] |
| `get_set_current()` | 電流制限値を読み取り [A] |
| `get_ov()` | OVP設定値を読み取り [V] |
| `get_oc()` | OCP設定値を読み取り [A] |
| `measure_voltage()` | 電圧を測定 [V] |
| `measure_current()` | 電流を測定 [A] |
| `get_id()` | 識別IDを取得 |
| `get_device_id()` | デバイスID文字列を取得 |

### `PowerSupplyError`

| バリアント | 説明 |
|---|---|
| `ConnectionError(io::Error)` | TCP接続失敗 |
| `CommunicationError(io::Error)` | 送受信エラー |
| `ParseError(String)` | レスポンスのパース失敗 |
| `Other(String)` | その他のエラー |
