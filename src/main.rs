/*!
 * ThingsBoard数据推送工具
 *
 * 这是一个用Rust编写的命令行工具，用于向ThingsBoard平台发送模拟数据。
 * 支持从JSON文件读取数据，可配置发送间隔和次数，支持多种数据类型。
 *
 * 主要功能：
 * - 从.env文件读取ThingsBoard服务器配置
 * - 从JSON文件读取模拟数据
 * - 支持定时发送和循环发送
 * - 完善的错误处理和日志输出
 *
 * 作者: Yu Xinyang
 * 版本: 1.0
 */

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

/// ThingsBoard服务器配置结构体
///
/// 包含连接ThingsBoard所需的基本配置信息
#[derive(Debug, Deserialize)]
struct Config {
    /// ThingsBoard服务器地址 (例如: http://localhost:8080)
    server: String,
    /// 设备访问令牌，用于身份验证
    device_token: String,
}

/// ThingsBoard遥测数据结构体
///
/// 符合ThingsBoard API要求的遥测数据格式
#[derive(Debug, Serialize)]
struct TelemetryData {
    /// 时间戳（毫秒）
    ts: u64,
    /// 遥测数据键值对
    values: HashMap<String, Value>,
}

/// 程序主入口函数
///
/// 负责解析命令行参数、加载配置、读取数据文件并执行数据发送任务
///
/// # 返回值
///
/// * `Result<()>` - 成功时返回Ok(())，失败时返回错误信息
#[tokio::main]
async fn main() -> Result<()> {
    // 加载.env文件中的环境变量
    dotenv().ok();

    // 构建命令行参数解析器
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

    // 解析命令行参数
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

    // 开始数据发送循环
    let mut sent_count = 0;
    loop {
        // 遍历数据文件中的每一项数据
        for (index, item) in data.iter().enumerate() {
            // 尝试发送遥测数据到ThingsBoard
            match send_telemetry(&client, &config, item).await {
                Ok(_) => {
                    sent_count += 1;
                    println!("✅ 第{}次发送成功 - 数据项 {}/{}", sent_count, index + 1, data.len());
                }
                Err(e) => {
                    eprintln!("❌ 发送失败: {}", e);
                }
            }

            // 在发送数据项之间等待指定间隔时间
            if interval > 0 && index < data.len() - 1 {
                sleep(Duration::from_secs(interval)).await;
            }
        }

        // 检查是否达到指定的发送次数
        if count > 0 {
            if sent_count >= count * data.len() as u64 {
                break;
            }
        }

        // 如果需要继续发送，等待下一轮
        if count == 0 || sent_count < count * data.len() as u64 {
            println!("⏳ 等待 {} 秒后继续下一轮发送...", interval);
            sleep(Duration::from_secs(interval)).await;
        }
    }

    println!("🎉 数据发送完成！总共发送了 {} 条数据", sent_count);
    Ok(())
}

/// 从环境变量加载ThingsBoard配置
///
/// 从.env文件或系统环境变量中读取服务器地址和设备令牌
///
/// # 返回值
///
/// * `Result<Config>` - 成功时返回配置对象，失败时返回错误信息
///
/// # 错误
///
/// 当环境变量'server'或'device_token'不存在时返回错误
fn load_config() -> Result<Config> {
    let server = env::var("server").context("未找到环境变量 'server'")?;
    let device_token = env::var("device_token").context("未找到环境变量 'device_token'")?;

    Ok(Config {
        server,
        device_token,
    })
}

/// 从文件加载JSON数据
///
/// 读取指定路径的JSON文件并解析为Value数组
///
/// # 参数
///
/// * `file_path` - JSON数据文件的路径
///
/// # 返回值
///
/// * `Result<Vec<Value>>` - 成功时返回JSON数据数组，失败时返回错误信息
///
/// # 错误
///
/// 当文件不存在、无法读取或JSON格式错误时返回错误
fn load_data_file(file_path: &str) -> Result<Vec<Value>> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("无法读取数据文件: {}", file_path))?;

    let data: Vec<Value> = serde_json::from_str(&content)
        .with_context(|| format!("无法解析JSON数据文件: {}", file_path))?;

    Ok(data)
}

/// 向ThingsBoard发送遥测数据
///
/// 将JSON数据转换为ThingsBoard遥测格式并通过HTTP API发送
///
/// # 参数
///
/// * `client` - HTTP客户端实例
/// * `config` - ThingsBoard配置信息
/// * `data` - 要发送的JSON数据
///
/// # 返回值
///
/// * `Result<()>` - 成功时返回Ok(())，失败时返回错误信息
///
/// # 错误
///
/// 当网络请求失败、服务器返回错误状态码或数据格式错误时返回错误
async fn send_telemetry(client: &Client, config: &Config, data: &Value) -> Result<()> {
    // 获取当前时间戳（毫秒），用于ThingsBoard时间序列数据
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("无法获取系统时间")?
        .as_millis() as u64;

    // 构建符合ThingsBoard API要求的遥测数据格式
    let telemetry = TelemetryData {
        ts: timestamp,
        values: extract_telemetry_values(data)?,
    };

    // 构建ThingsBoard遥测数据API的请求URL
    let url = format!("{}/api/v1/{}/telemetry", config.server, config.device_token);

    // 发送HTTP POST请求到ThingsBoard
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&telemetry)
        .send()
        .await
        .context("发送HTTP请求失败")?;

    // 检查响应状态并处理结果
    if response.status().is_success() {
        println!("📤 数据发送成功: {}", serde_json::to_string(&telemetry.values)?);
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("HTTP请求失败: {} - {}", status, error_text);
    }
}

/// 从JSON数据中提取遥测值
///
/// 将嵌套的JSON结构转换为扁平的键值对，适合ThingsBoard遥测数据格式
///
/// # 参数
///
/// * `data` - 输入的JSON数据
///
/// # 返回值
///
/// * `Result<HashMap<String, Value>>` - 成功时返回遥测数据键值对，失败时返回错误信息
///
/// # 数据转换规则
///
/// 1. 对于嵌套对象（如{"rain": {...}}），会：
///    - 添加"data_category"字段，值为顶层键名
///    - 添加顶层键名作为布尔字段，值为true（便于搜索）
///    - 展开内部对象的所有字段
/// 2. 对于非对象值，直接使用原键值对
///
/// # 错误
///
/// 当输入数据不是JSON对象或无法提取有效数据时返回错误
fn extract_telemetry_values(data: &Value) -> Result<HashMap<String, Value>> {
    let mut values = HashMap::new();

    match data {
        Value::Object(obj) => {
            for (key, value) in obj {
                match value {
                    Value::Object(inner_obj) => {
                        // 添加数据分类字段，用于标识数据类型
                        values.insert("data_category".to_string(), Value::String(key.clone()));

                        // 添加顶层键名作为布尔字段，方便在ThingsBoard中搜索
                        values.insert(key.clone(), Value::Bool(true));

                        // 展开内部对象的所有字段，保持原始字段名
                        for (inner_key, inner_value) in inner_obj {
                            values.insert(inner_key.clone(), inner_value.clone());
                        }
                    }
                    _ => {
                        // 对于非对象值，直接使用原键值对
                        values.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        _ => {
            anyhow::bail!("数据格式不正确，期望JSON对象");
        }
    }

    // 验证是否提取到有效数据
    if values.is_empty() {
        anyhow::bail!("未能提取到有效的遥测数据");
    }

    Ok(values)
}


