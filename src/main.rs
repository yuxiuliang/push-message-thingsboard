use anyhow::{Context, Result};
use clap::{Arg, Command};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct Config {
    server: String,
    device_token: String,
}

#[derive(Debug, Serialize)]
struct TelemetryData {
    ts: u64,
    values: HashMap<String, Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 加载环境变量
    dotenv().ok();

    let matches = Command::new("ThingsBoard数据推送工具")
        .version("1.0")
        .author("Yu Xinyang")
        .about("向ThingsBoard发送模拟数据")
        .arg(
            Arg::new("interval")
                .short('i')
                .long("interval")
                .value_name("SECONDS")
                .help("发送数据的间隔时间（秒）")
                .default_value("5"),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .value_name("NUMBER")
                .help("发送数据的次数，0表示无限循环")
                .default_value("1"),
        )
        .arg(
            Arg::new("data-file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("数据文件路径")
                .default_value("data.json"),
        )
        .get_matches();

    let interval: u64 = matches
        .get_one::<String>("interval")
        .unwrap()
        .parse()
        .context("间隔时间必须是有效的数字")?;

    let count: u64 = matches
        .get_one::<String>("count")
        .unwrap()
        .parse()
        .context("发送次数必须是有效的数字")?;

    let data_file = matches.get_one::<String>("data-file").unwrap();

    // 读取配置
    let config = load_config()?;
    println!("✅ 配置加载成功:");
    println!("   服务器: {}", config.server);
    println!("   设备Token: {}...", &config.device_token[..8]);

    // 读取数据文件
    let data = load_data_file(data_file)?;
    println!("✅ 数据文件加载成功，包含 {} 条记录", data.len());

    // 创建HTTP客户端
    let client = Client::new();

    // 发送数据
    let mut sent_count = 0;
    loop {
        for (index, item) in data.iter().enumerate() {
            match send_telemetry(&client, &config, item).await {
                Ok(_) => {
                    sent_count += 1;
                    println!("✅ 第{}次发送成功 - 数据项 {}/{}", sent_count, index + 1, data.len());
                }
                Err(e) => {
                    eprintln!("❌ 发送失败: {}", e);
                }
            }

            if interval > 0 && index < data.len() - 1 {
                sleep(Duration::from_secs(interval)).await;
            }
        }

        if count > 0 {
            if sent_count >= count * data.len() as u64 {
                break;
            }
        }

        if count == 0 || sent_count < count * data.len() as u64 {
            println!("⏳ 等待 {} 秒后继续下一轮发送...", interval);
            sleep(Duration::from_secs(interval)).await;
        }
    }

    println!("🎉 数据发送完成！总共发送了 {} 条数据", sent_count);
    Ok(())
}

fn load_config() -> Result<Config> {
    let server = env::var("server").context("未找到环境变量 'server'")?;
    let device_token = env::var("device_token").context("未找到环境变量 'device_token'")?;

    Ok(Config {
        server,
        device_token,
    })
}

fn load_data_file(file_path: &str) -> Result<Vec<Value>> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("无法读取数据文件: {}", file_path))?;

    let data: Vec<Value> = serde_json::from_str(&content)
        .with_context(|| format!("无法解析JSON数据文件: {}", file_path))?;

    Ok(data)
}

async fn send_telemetry(client: &Client, config: &Config, data: &Value) -> Result<()> {
    // 获取当前时间戳（毫秒）
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("无法获取系统时间")?
        .as_millis() as u64;

    // 构建ThingsBoard遥测数据格式
    let telemetry = TelemetryData {
        ts: timestamp,
        values: extract_telemetry_values(data)?,
    };

    // 构建请求URL
    let url = format!("{}/api/v1/{}/telemetry", config.server, config.device_token);

    // 发送HTTP POST请求
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&telemetry)
        .send()
        .await
        .context("发送HTTP请求失败")?;

    if response.status().is_success() {
        println!("📤 数据发送成功: {}", serde_json::to_string(&telemetry.values)?);
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("HTTP请求失败: {} - {}", status, error_text);
    }
}

fn extract_telemetry_values(data: &Value) -> Result<HashMap<String, Value>> {
    let mut values = HashMap::new();

    match data {
        Value::Object(obj) => {
            for (key, value) in obj {
                match value {
                    Value::Object(inner_obj) => {
                        // 添加顶层键名作为一个字段
                        values.insert("data_category".to_string(), Value::String(key.clone()));

                        // 同时添加键名本身作为布尔值，方便搜索
                        values.insert(key.clone(), Value::Bool(true));

                        // 直接使用内部对象的所有字段，不重命名
                        for (inner_key, inner_value) in inner_obj {
                            values.insert(inner_key.clone(), inner_value.clone());
                        }
                    }
                    _ => {
                        values.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        _ => {
            anyhow::bail!("数据格式不正确，期望JSON对象");
        }
    }

    if values.is_empty() {
        anyhow::bail!("未能提取到有效的遥测数据");
    }

    Ok(values)
}


