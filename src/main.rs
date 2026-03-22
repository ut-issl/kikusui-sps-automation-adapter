use clap::Parser;
use std::{thread, time::Duration, fs};
use std::path::Path;
use std::io::BufWriter;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::{Utc, SecondsFormat};
use serde_json::{json, Value};
use kikusui_sps_adapter::PowerSupply;

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
    #[arg(long)]
    log_file: Option<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    assert!(args.freq > 0.0, "freq must be > 0");

    let mut psu = PowerSupply::new("PSU1", &args.host, args.port)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    println!("Connected: {}", psu.get_device_id());

    psu.output_on()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let interval_ms = (1.0 / args.freq * 1000.0) as u64;

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
        let measurement = psu.measure()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        println!(
            "Time: {}, Voltage: {} V, Current: {} A",
            measurement.timestamp, measurement.voltage, measurement.current
        );

        let log_file_path = format!("{}/{}", log_dir, log_file_name);
        write_jsonl(log_file_path, &to_json(&measurement.timestamp, &measurement.voltage, &measurement.current))?;

        thread::sleep(Duration::from_millis(interval_ms));
    }
}

fn to_json(time: &str, voltage: &f64, current: &f64) -> Value {
    json!({
        "time": time,
        "voltage": voltage,
        "current": current,
    })
}

fn write_jsonl<P: AsRef<Path>>(path: P, v: &Value) -> std::io::Result<()> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut w = BufWriter::new(file);
    serde_json::to_writer(&mut w, v)?;
    w.write_all(b"\n")?;
    w.flush()?;
    Ok(())
}
