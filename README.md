# Kikusui SPS Automation Adapter

菊水電源装置からデータを取得し、[fluentbit](https://fluentbit.io/)を通して[influxDB](https://www.influxdata.com/)に格納するRustアプリケーションです。TCP接続を通じて電源装置と通信し、電圧と電流などの測定値をリアルタイムで取得・記録します。

## 要件

- 菊水電源装置（TCP/IP接続対応）
    - 動作確認済み機種：**PMX35-3A**
- Rust 1.70以上
- [fluentbit](https://fluentbit.io/)
- [influxDB](https://www.influxdata.com/)

## インストール

```bash
git clone https://github.com/ut-issl/kikusui-sps-automation-adapter.git
cd kikusui-sps-automation-adapter
cargo build
```

## 使用方法

### 1. 記録スクリプトを実行

```bash
cargo run -- --host <HOST_IP> --port <PORT>
```

#### オプション

- `--host <HOST_IP>`: 電源装置のIPアドレス（必須）
- `--port <PORT>`: 電源装置のポート番号（必須）
- `--freq <FREQUENCY>`: 測定頻度（Hz）（デフォルト: 1.0）
- `--log-file <NAME>`: ログファイル名（オプション、デフォルトはタイムスタンプ）

#### 実行例

```bash
# 基本的な実行（1Hzで測定）
cargo run -- --host 192.168.1.100 --port 10001

# 10Hzで測定
cargo run -- --host 192.168.1.100 --port 10001 --freq 10.0

# カスタムログファイル名を指定
cargo run -- --host 192.168.1.100 --port 10001 --log-file experiment1
```

#### ログファイル

測定データは`./logs/`ディレクトリに以下の形式で保存されます：

- ファイル名: `<log_file>_<timestamp>.jsonl` または `<timestamp>.jsonl`
- 形式: JSONL（JSON Lines）

#### ログデータ形式

```json
{"time":"2025-09-09T12:00:00.000Z","voltage":5.0,"current":1.5}
```

- `time`: 測定時刻（ISO 8601形式）
- `voltage`: 電圧値（V）
- `current`: 電流値（A）

## 2. Fluentbitの実行

`fluentbit.yaml`内outputs配下の、host、port、http_tokenを記録先のinfluxDBが立ち上げられているPCの情報に書き換えてください。そのうえで`fluentbit.yaml`を設定ファイルとして、fluentbitサーバーを起動させます。

```bash
fluent-bit --config fluentbit.yaml
```
1で実行したスクリプトによりjsonlファイルが更新されるたびに、指定したinfluxDBにデータが格納されます。

## ライセンス

本プロジェクトのライセンス情報については、[LICENSE](LICENSE)ファイルを参照してください。
