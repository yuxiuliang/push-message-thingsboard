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
use chrono::Local;
use tokio::time::{sleep, Duration};
use rand::Rng;

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
    /// 发送时间
    time: String,
}

/// 数据文件解析结果结构体
///
/// 包含从数据文件中解析出的随机键和数据数组
#[derive(Debug)]
struct DataFileResult {
    /// 随机键名称（如 "drp"）
    random_key: Option<String>,
    /// 数据数组
    data: Vec<Value>,
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
    let data_result = load_data_file(data_file)?;
    println!("✅ 数据文件加载成功，包含 {} 条记录", data_result.data.len());
    if let Some(ref key) = data_result.random_key {
        println!("🎲 检测到随机字段: {}", key);
    }

    // 创建HTTP客户端
    let client = Client::new();

    // 开始数据发送循环
    let mut sent_count = 0;
    loop {
        // 遍历数据文件中的每一项数据
        for (index, item) in data_result.data.iter().enumerate() {
            // 尝试发送遥测数据到ThingsBoard
            match send_telemetry(&client, &config, item, &data_result.random_key).await {
                Ok(_) => {
                    sent_count += 1;
                    println!("✅ 第{}次发送成功 - 数据项 {}/{}", sent_count, index + 1, data_result.data.len());
                }
                Err(e) => {
                    eprintln!("❌ 发送失败: {}", e);
                }
            }

            // 在发送数据项之间等待指定间隔时间
            if interval > 0 && index < data_result.data.len() - 1 {
                sleep(Duration::from_secs(interval)).await;
            }
        }

        // 检查是否达到指定的发送次数
        if count > 0 {
            if sent_count >= count * data_result.data.len() as u64 {
                break;
            }
        }

        // 如果需要继续发送，等待下一轮
        if count == 0 || sent_count < count * data_result.data.len() as u64 {
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
/// 读取指定路径的JSON文件并解析为DataFileResult结构体
/// 支持两种格式：
/// 1. 直接数组格式: [{"sensor1": {...}}, {"sensor2": {...}}]
/// 2. 包装对象格式: {"random_key": "...", "data": [{"sensor1": {...}}, {"sensor2": {...}}]}
///
/// # 参数
///
/// * `file_path` - JSON数据文件的路径
///
/// # 返回值
///
/// * `Result<DataFileResult>` - 成功时返回包含随机键和数据数组的结果，失败时返回错误信息
///
/// # 错误
///
/// 当文件不存在、无法读取或JSON格式错误时返回错误
fn load_data_file(file_path: &str) -> Result<DataFileResult> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("无法读取数据文件: {}", file_path))?;

    // 首先尝试解析为通用Value
    let json_value: Value = serde_json::from_str(&content)
        .with_context(|| format!("无法解析JSON数据文件: {}", file_path))?;

    // 根据JSON结构判断格式并提取数据数组和随机键
    let result = match json_value {
        // 格式1: 直接数组 [{"sensor1": {...}}, {"sensor2": {...}}]
        Value::Array(arr) => {
            println!("🔍 检测到直接数组格式的数据文件");
            DataFileResult {
                random_key: None,
                data: arr,
            }
        }
        // 格式2: 包装对象 {"random_key": "...", "data": [...]}
        Value::Object(obj) => {
            println!("🔍 检测到包装对象格式的数据文件");
            
            // 提取数据数组
            let data = if let Some(Value::Array(data_array)) = obj.get("data") {
                data_array.clone()
            } else {
                anyhow::bail!("包装对象格式中未找到 'data' 字段或 'data' 不是数组");
            };

            // 查找随机键的值（"random_key" 字段的值，这个值指示要随机修改哪个字段）
            let random_key = obj.get("random_key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            DataFileResult {
                random_key,
                data,
            }
        }
        _ => {
            anyhow::bail!("不支持的JSON格式，期望数组或包含'data'字段的对象");
        }
    };

    // 验证数据是否为空
    if result.data.is_empty() {
        anyhow::bail!("数据文件中没有找到有效数据");
    }

    Ok(result)
}

/// 向ThingsBoard发送遥测数据
///
/// 将JSON数据转换为ThingsBoard遥测格式并通过HTTP API发送
/// 如果提供了随机键，会随机修改对应字段的值
///
/// # 参数
///
/// * `client` - HTTP客户端实例
/// * `config` - ThingsBoard配置信息
/// * `data` - 要发送的JSON数据
/// * `random_key` - 可选的随机键名称，如果存在会随机修改对应字段的值
///
/// # 返回值
///
/// * `Result<()>` - 成功时返回Ok(())，失败时返回错误信息
///
/// # 错误
///
/// 当网络请求失败、服务器返回错误状态码或数据格式错误时返回错误
async fn send_telemetry(client: &Client, config: &Config, data: &Value, random_key: &Option<String>) -> Result<()> {
    // 获取当前时间戳（毫秒），用于ThingsBoard时间序列数据
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("无法获取系统时间")?
        .as_millis() as u64;
    // 获取当前时间的字符串格式 yyyy-MM-dd HH:mm:ss
    let send_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    // 构建符合ThingsBoard API要求的遥测数据格式
    let mut values = extract_telemetry_values(data, random_key)?;
    // 将发送时间添加到遥测数据中
    values.insert("send_time".to_string(), Value::String(send_time.clone()));

    let telemetry = TelemetryData {
        ts: timestamp,
        values,
        time: send_time,
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
        println!("📤 数据发送成功!");
        println!("🕒 发送时间: {}", telemetry.time);
        println!("📊 发送数据: {}", serde_json::to_string_pretty(&telemetry.values)?);
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("HTTP请求失败: {} - {}", status, error_text);
    }
}

/// 从JSON数据中提取遥测值
///
/// 保持原始的JSON对象结构，不展开嵌套对象
/// 如果提供了随机键，会在对应字段中随机修改相关值
///
/// # 参数
///
/// * `data` - 输入的JSON数据
/// * `random_key` - 可选的随机键名称，如果存在会随机修改对应字段的值
///
/// # 返回值
///
/// * `Result<HashMap<String, Value>>` - 成功时返回遥测数据键值对，失败时返回错误信息
///
/// # 数据转换规则
///
/// 1. 对于嵌套对象（如{"rain": {...}}），会：
///    - 保持完整的对象结构作为值
///    - 使用顶层键名作为字段名
///    - 如果指定了随机键，会在嵌套对象中查找并随机修改对应字段的值
/// 2. 对于非对象值，直接使用原键值对
///
/// # 错误
///
/// 当输入数据不是JSON对象或无法提取有效数据时返回错误
fn extract_telemetry_values(data: &Value, random_key: &Option<String>) -> Result<HashMap<String, Value>> {
    let mut values = HashMap::new();

    match data {
        Value::Object(obj) => {
            for (key, value) in obj {
                // 如果存在随机键且当前值是对象，则尝试随机修改对应字段
                if let (Some(random_field), Value::Object(nested_obj)) = (random_key, value) {
                    if let Some(random_value) = nested_obj.get(random_field) {
                        // 创建修改后的嵌套对象
                        let mut modified_nested = nested_obj.clone();
                        let new_random_value = generate_random_value(random_value)?;
                        modified_nested.insert(random_field.clone(), new_random_value.clone());
                        
                        println!("🎲 随机修改字段 '{}': {} -> {}", 
                            random_field, 
                            random_value, 
                            new_random_value
                        );
                        
                        values.insert(key.clone(), Value::Object(modified_nested));
                    } else {
                        // 如果没有找到随机字段，保持原始结构
                        values.insert(key.clone(), value.clone());
                    }
                } else {
                    // 直接使用原始的键值对，保持对象结构
                    values.insert(key.clone(), value.clone());
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

/// 根据原始值的类型生成随机值
///
/// 支持数字类型的随机生成，保持原始值的数据类型
///
/// # 参数
///
/// * `original_value` - 原始值，用于确定生成随机值的类型和范围
///
/// # 返回值
///
/// * `Result<Value>` - 成功时返回随机生成的值，失败时返回错误信息
///
/// # 随机值生成规则
///
/// 1. 整数：生成 [1, 原值*2] 范围内的随机整数
/// 2. 浮点数：生成 [1.0, 原值*2.0] 范围内的随机浮点数
/// 3. 其他类型：保持原值不变
fn generate_random_value(original_value: &Value) -> Result<Value> {
    let mut rng = rand::thread_rng();
    
    match original_value {
        Value::Number(num) => {
            if let Some(int_val) = num.as_i64() {
                // 整数类型：生成 [1, 原值*2] 范围内的随机整数
                let max_val = std::cmp::max(1, int_val * 2);
                let random_val = rng.gen_range(1..=max_val);
                Ok(Value::Number(serde_json::Number::from(random_val)))
            } else if let Some(float_val) = num.as_f64() {
                // 浮点数类型：生成 [1.0, 原值*2.0] 范围内的随机浮点数
                let max_val = if float_val > 0.0 { float_val * 2.0 } else { 100.0 };
                let random_val = rng.gen_range(1.0..=max_val);
                Ok(Value::Number(serde_json::Number::from_f64(random_val)
                    .context("无法创建随机浮点数")?))
            } else {
                // 无法识别的数字类型，保持原值
                Ok(original_value.clone())
            }
        }
        _ => {
            // 非数字类型，保持原值不变
            Ok(original_value.clone())
        }
    }
}


