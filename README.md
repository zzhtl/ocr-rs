# OCR-RS - 跨平台OCR图片识别工具

一个基于Rust开发的跨平台OCR（光学字符识别）应用程序，支持Windows、Linux和macOS。

## 功能特点

- 🖼️ **图片上传识别**: 支持多种图片格式（PNG、JPG、JPEG、BMP、TIFF、WebP）
- 🎯 **高精度识别**: 支持多种OCR引擎（Tesseract、自定义Candle模型）
- 🌐 **跨平台支持**: 可在Windows、Linux、macOS上运行
- 📦 **无系统依赖**: 静态编译，无需额外安装系统依赖
- 🎨 **现代UI界面**: 基于egui的现代化用户界面
- 📊 **识别结果统计**: 显示置信度和处理时间
- 🔄 **实时处理**: 异步处理，界面不卡顿

## 技术架构

- **GUI框架**: egui - 轻量级跨平台GUI
- **图像处理**: image crate - Rust图像处理库
- **OCR引擎**: 
  - Tesseract OCR (传统OCR引擎)
  - Candle ML (自定义深度学习模型)
- **异步处理**: Tokio异步运行时
- **文件对话框**: rfd - 跨平台文件选择器

## 安装和运行

### 前置要求

- Rust 1.70+ 
- 如果使用Tesseract功能，需要系统安装Tesseract

### 编译运行

```bash
# 克隆项目
git clone <repository_url>
cd ocr-rs

# 运行项目
cargo run --release

# 编译发布版本
cargo build --release
```

### 禁用Tesseract（如果系统没有安装）

```bash
cargo run --release --no-default-features
```

## 使用方法

1. 启动应用程序
2. 点击"📁 选择图片"按钮选择要识别的图片
3. 应用程序会自动加载图片并开始OCR识别
4. 识别结果会显示在右侧面板中
5. 可以查看识别的文本、置信度和处理时间

## 项目结构

```
ocr-rs/
├── src/
│   ├── main.rs          # 程序入口
│   ├── app.rs           # 主应用程序逻辑
│   ├── ocr.rs           # OCR引擎实现
│   └── ui.rs            # UI组件
├── Cargo.toml           # 项目配置
└── README.md            # 项目说明
```

## OCR引擎说明

### Tesseract OCR
- 传统的OCR引擎，支持多种语言
- 需要系统安装Tesseract
- 适合处理清晰的文档图片

### Candle ML模型
- 基于深度学习的OCR模型
- 可以加载自定义训练的模型
- 支持更复杂的场景识别

## 开发计划

- [ ] 支持批量图片处理
- [ ] 添加图片预处理功能（去噪、二值化等）
- [ ] 支持PDF文件OCR
- [ ] 结果导出功能（文本文件、JSON等）
- [ ] 多语言界面支持
- [ ] 云端OCR API集成

## 自定义模型

要使用自定义的Candle OCR模型：

1. 将训练好的模型文件放在 `models/ocr_model.safetensors`
2. 重新编译项目
3. 应用程序会自动加载并使用自定义模型

详细的模型训练指南请参考 `ocr-ai` 项目。

## 贡献

欢迎提交Issue和Pull Request来改进这个项目。

## 许可证

MIT License 