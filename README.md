# 豆包 AI 图片去水印工具

<div align="center">

一款用 Rust 实现的命令行工具，通过智能拼接去除豆包 AI 生成图片的水印。

[![GitHub](https://img.shields.io/badge/GitHub-ever%2B-blue.svg)](https://github.com/everplus/rust-doubao-watermark)
[![Rust](https://img.shields.io/badge/Rust-2024.0-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-1.0.1-green.svg)](Cargo.toml)

</div>

---

## 功能特性

- **智能拼接** - 自动识别并拼接图片的上下部分，精准去除水印区域
- **剪贴板监听** - 实时监听剪贴板变化，自动捕获复制的图片
- **终端预览** - 支持在终端中直接预览图片，无需打开外部查看器
- **跨平台** - 支持 Windows、macOS 和 Linux 系统
- **零依赖运行** - 编译后的可执行文件独立运行，无需额外依赖


## 安装使用

### 前置要求

- [Rust](https://www.rust-lang.org/) 1.85+ (2024 edition)

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/everplus/rust-doubao-watermark.git
cd rust-doubao-watermark

# 开发构建
cargo build

# 发布构建（推荐，优化性能）
cargo build --release
```

### 运行程序

```bash
# 开发版本
cargo run

# 发布版本（Windows）
.\target\release\doubao-watermark.exe

# 发布版本（macOS/Linux）
./target/release/doubao-watermark
```

## 使用流程

1. **启动程序** - 运行后程序会自动清空剪贴板

2. **获取上半部分图片**
   - 在浏览器中将生成的大图拖动到新标签页
   - 右键点击图片，选择「复制图片」
   - 程序自动捕获并显示预览

3. **获取下半部分图片**
   - 直接右键点击生成的大图
   - 选择「复制」菜单项
   - 程序自动捕获并显示预览

4. **自动拼接** - 程序自动完成图片拼接并显示结果

5. **保存结果** - 图片自动保存至桌面，文件名格式为 `doubao_image_时间戳.png`

## 技术栈

| 库 | 用途 | 说明 |
|---|---|---|
| [arboard](https://github.com/1Password/arboard) | 剪贴板操作 | 跨平台剪贴板访问 |
| [image](https://github.com/image-rs/image) | 图片处理 | 多格式图片读取、转换和拼接 |
| [viuer](https://github.com/atanunq/viuer) | 终端显示 | 支持 Sixel、Kitty、iTerm2 等协议 |

## 终端图片支持

`viuer` 会自动检测并使用最佳终端图像协议：

| 协议 | 终端示例 |
|---|---|
| Sixel | xterm, mintty, ConEmu |
| Kitty Graphics Protocol | Kitty 终端 |
| iTerm2 Inline Images | macOS iTerm2 |
| ASCII Art fallback | 不支持图像时降级 |

## 项目结构

```
rust-doubao-watermark/
├── Cargo.toml          # 项目配置和依赖
├── Cargo.lock          # 依赖版本锁定
├── src/
│   └── main.rs         # 主程序（单文件架构）
├── LICENSE             # MIT 许可证
├── README.md           # 项目说明
└── CLAUDE.md           # Claude Code 工作指引
```

## 核心模块说明

### 剪贴板操作
- `clear_clipboard()` - 清空剪贴板，确保开始时没有旧数据
- `wait_for_image()` - 轮询监听剪贴板变化，等待用户复制图片

### 图片处理
- `convert_to_png()` - 将剪贴板获取的图片数据解析为 `DynamicImage`
- `stitch_images()` - 核心拼接逻辑，验证尺寸并垂直分割拼接

### 图片显示
- `display_image()` - 使用 `viuer` 在终端显示图片预览，带超时保护

### 文件操作
- `get_desktop_path()` - 跨平台获取桌面路径
- `save_image()` - 保存图片为 PNG 格式到桌面

## 许可证

本项目采用 [MIT](LICENSE) 许可证开源。

## 贡献

欢迎提交 Issue 和 Pull Request！

---

<div align="center">

Made with ❤️ and Rust

</div>
