use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{thread, time::{Duration, Instant, SystemTime}, fs};
use std::path::{Path, PathBuf};
use std::io::BufWriter;
use std::fs::OpenOptions;
use chrono::{DateTime, TimeZone, Utc, SecondsFormat};
use serde_json::{json, Value};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Args {
    /// Name of the host
    #[arg(long)]
    host: String,

    /// Number of the port
    #[arg(long)]
    port: u16,

    /// Frequency [Hz]
    #[arg(long, default_value_t = 1.0)]
    freq: f64, 

    /// Name of log file
    #[arg(long)] // 使用目的が多岐にわたるのでoptionで名前を指定できるようにする(デフォルトではTimestamp)
    log_file: Option<String>,
}

use std::io::prelude::*;
use std::net::TcpStream;

// 毎回ファイル名を変えるのは大変，変え忘れる可能性あり
// 実行の度にjsonlファイルの名前を変更する(Timestamp)
fn main() -> std::io::Result<()> {
    let args = Args::parse();
    assert!(args.freq > 0.0, "freq must be > 0");

    let mut stream = TcpStream::connect((args.host.as_str(), args.port))?;

    stream.write(b"*IDN?\r\n")?;

    let mut buffer = [0; 100];

    // read up to 100 bytes
    let n = stream.read(&mut buffer[..])?;

    println!("{:?}", &buffer[0..n]);

    let s = std::str::from_utf8(&buffer[0..n]).unwrap();
    println!("{}", s.trim_end());

    stream.write(b"OUTP 1\r\n")?;

    let freq_hz = args.freq;
    let interval_ms = (1.0 / freq_hz * 1000.0) as u64;

    let log_dir = "./logs";
    if !Path::new(log_dir).exists() {
        fs::create_dir_all(log_dir).expect("Failed to create logs directory");
    }
    
    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let log_file_name = match &args.log_file {
        Some(name) if !name.is_empty() => format!("{}_{}.jsonl", name, ts),
        _ => format!("{}.jsonl", ts),
    };
    
    loop {
            // let time = SystemTime::now();
            let time = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            
            stream.write(b"MEAS:VOLT?\r\n")?;
            let mut buffer = [0; 100];
            // read up to 100 bytes
            let n = stream.read(&mut buffer[..])?;

            let voltage = std::str::from_utf8(&buffer[0..n]).unwrap();

            stream.write(b"MEAS:CURR?\r\n")?;
            let mut buffer = [0; 100];
            // read up to 100 bytes
            let n = stream.read(&mut buffer[..])?;

            let current = std::str::from_utf8(&buffer[0..n]).unwrap();

            let v = voltage.trim_end().parse::<f64>().unwrap(); // 一旦unwrap

            let c = current.trim_end().parse::<f64>().unwrap(); // 一旦unwrap

            println!("Time: {}, Voltage: {} V, Current: {} A", time, v, c);

            let log_file_path = format!("{}/{}", log_dir, log_file_name);

            write_jsonl(log_file_path, &to_json(&time, &v, &c));
            
            // Wait for interval_ms [ms]
            thread::sleep(Duration::from_millis(interval_ms));
        }

    Ok(())
} // the stream is closed here

fn to_json(time: &str, voltage: &f64, current: &f64) -> Value {
    json!({
        "time": time,
        "voltage": voltage,
        "current": current,
    })
}

fn write_jsonl<P: AsRef<Path>>(path: P, v:&Value) -> std::io::Result<()> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut w = BufWriter::new(file);
    serde_json::to_writer(&mut w, v)?;
    w.write_all(b"\n");
    w.flush()?;
    Ok(())
}
