# ThingsBoard 数据推送工具 - 使用指南

## 📦 获取可执行文件

### 方法 1：使用构建脚本（推荐）

双击运行 `build.bat` 文件，会自动构建 release 版本的 exe 文件。

### 方法 2：手动构建

在项目目录下打开命令行，执行：

```bash
cargo build --release
```

构建完成后，exe 文件位于：`target\release\push-message-thingsboard.exe`

## 🚀 使用 exe 文件

### 第一步：准备文件

确保以下文件在同一个目录下：

- `push-message-thingsboard.exe` （可执行文件）
- `.env` （配置文件）
- `data.json` （数据文件，或你指定的其他文件）

### 第二步：配置文件

确保 `.env` 文件包含正确的配置：

```env
# thingsboard配置
server=http://xxxx
device_token=xxxxx
```

### 第三步：运行 exe 文件

#### 基本使用

```bash
# 发送一次数据（使用默认设置）
push-message-thingsboard.exe

# 查看帮助信息
push-message-thingsboard.exe --help

# 查看版本信息
push-message-thingsboard.exe --version
```

#### 高级使用

```bash
# 设置间隔时间为3秒，发送2轮数据
push-message-thingsboard.exe --interval 3 --count 2

# 使用自定义数据文件
push-message-thingsboard.exe --file my_data.json

# 无限循环发送，间隔10秒
push-message-thingsboard.exe --interval 10 --count 0

# 组合使用多个参数
push-message-thingsboard.exe --file data_example.json --interval 5 --count 3
```

## 📋 命令行参数说明

| 参数         | 简写 | 说明                           | 默认值    |
| ------------ | ---- | ------------------------------ | --------- |
| `--interval` | `-i` | 发送数据的间隔时间（秒）       | 5         |
| `--count`    | `-c` | 发送数据的轮数，0 表示无限循环 | 1         |
| `--file`     | `-f` | 指定数据文件路径               | data.json |
| `--help`     | `-h` | 显示帮助信息                   | -         |
| `--version`  | `-V` | 显示版本信息                   | -         |

## 💡 使用示例

### 示例 1：快速测试

```bash
push-message-thingsboard.exe
```

使用默认设置发送一轮数据。

### 示例 2：持续监控模拟

```bash
push-message-thingsboard.exe --interval 30 --count 0
```

每 30 秒发送一轮数据，无限循环。

### 示例 3：批量测试

```bash
push-message-thingsboard.exe --interval 1 --count 10
```

每 1 秒发送一轮数据，总共发送 10 轮。

### 示例 4：使用自定义数据

```bash
push-message-thingsboard.exe --file sensor_data.json --interval 5 --count 5
```

使用自定义数据文件，每 5 秒发送一轮，总共 5 轮。

## 📊 输出说明

程序运行时会显示以下信息：

- ✅ 配置加载状态
- ✅ 数据文件加载状态
- 📤 每次数据发送的详细内容
- ✅ 发送成功确认
- ⏳ 等待间隔提示
- 🎉 完成统计信息

## ❌ 常见问题

### 问题 1：找不到配置文件

**错误信息**: `未找到环境变量 'server'`
**解决方案**: 确保 `.env` 文件在 exe 文件同一目录下，且包含正确的配置。

### 问题 2：找不到数据文件

**错误信息**: `无法读取数据文件: data.json`
**解决方案**: 确保数据文件存在，或使用 `--file` 参数指定正确的文件路径。

### 问题 3：网络连接失败

**错误信息**: `发送HTTP请求失败`
**解决方案**: 检查网络连接和 ThingsBoard 服务器地址是否正确。

### 问题 4：JSON 格式错误

**错误信息**: `无法解析JSON数据文件`
**解决方案**: 检查数据文件的 JSON 格式是否正确。

## 🔧 部署建议

### 单机部署

1. 将 exe 文件、.env 文件和数据文件放在同一目录
2. 创建批处理文件方便运行
3. 可以设置 Windows 计划任务定时运行

### 批处理文件示例

创建 `run.bat` 文件：

```batch
@echo off
echo 启动ThingsBoard数据推送工具...
push-message-thingsboard.exe --interval 60 --count 0
pause
```

### 服务化部署

可以使用 Windows 服务管理工具将程序注册为系统服务，实现开机自启动。

## 📝 注意事项

1. **文件路径**: 确保所有必需文件在正确位置
2. **网络连接**: 确保能访问 ThingsBoard 服务器
3. **数据格式**: 确保 JSON 数据格式正确
4. **权限问题**: 某些情况下可能需要管理员权限运行
5. **防火墙**: 确保防火墙允许程序访问网络

## 🆘 获取帮助

如果遇到问题，可以：

1. 运行 `push-message-thingsboard.exe --help` 查看帮助
2. 检查错误信息并参考常见问题部分
3. 查看 README.md 文件获取更多技术细节
