use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use image::{DynamicImage, GenericImageView};

use crate::ocr::{OcrEngine, OcrResult};
use crate::ui::{ImageDisplay, StatusDisplay, ResultPanel};

#[derive(Debug)]
pub enum AppMessage {
    ImageSelected(PathBuf),
    OcrCompleted(OcrResult),
    OcrError(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Idle,
    Loading,
    Processing,
    Completed,
    Error(String),
}

pub struct OcrApp {
    // 应用状态
    state: AppState,
    
    // 图像相关
    selected_image_path: Option<PathBuf>,
    current_image: Option<DynamicImage>,
    image_display: ImageDisplay,
    
    // OCR相关
    ocr_result: Option<OcrResult>,
    ocr_engine: Arc<OcrEngine>,
    
    // UI组件
    status_display: StatusDisplay,
    result_panel: ResultPanel,
    
    // 异步通信
    tx: mpsc::UnboundedSender<AppMessage>,
    rx: mpsc::UnboundedReceiver<AppMessage>,
    rt: tokio::runtime::Runtime,
    
    // UI状态
    show_settings: bool,
    dark_mode: bool,
    show_image_viewer: bool,
    image_scale: f32,
}

impl OcrApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let ocr_engine = Arc::new(OcrEngine::new());
        
        Self {
            state: AppState::Idle,
            selected_image_path: None,
            current_image: None,
            image_display: ImageDisplay::new(),
            ocr_result: None,
            ocr_engine,
            status_display: StatusDisplay::new(),
            result_panel: ResultPanel::new(),
            tx,
            rx,
            rt,
            show_settings: false,
            dark_mode: true,
            show_image_viewer: false,
            image_scale: 1.0,
        }
    }
    
    fn reset_state(&mut self) {
        self.state = AppState::Idle;
        self.ocr_result = None;
        self.status_display.clear();
    }
    
    fn handle_file_selection(&mut self) {
        let tx = self.tx.clone();
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("图片文件", &["png", "jpg", "jpeg", "bmp", "tiff", "webp", "gif"])
            .set_title("选择要识别的图片")
            .pick_file()
        {
            let _ = tx.send(AppMessage::ImageSelected(path));
        }
    }
    
    fn handle_image_selected(&mut self, path: PathBuf) {
        self.state = AppState::Loading;
        self.selected_image_path = Some(path.clone());
        self.status_display.set_message("正在加载图片...");
        
        match image::open(&path) {
            Ok(img) => {
                self.current_image = Some(img.clone());
                self.image_display.set_image(img.clone());
                self.start_ocr_processing(img, path);
            }
            Err(e) => {
                self.state = AppState::Error(format!("无法加载图片: {}", e));
                self.status_display.set_error(&format!("图片加载失败: {}", e));
            }
        }
    }
    
    fn start_ocr_processing(&mut self, image: DynamicImage, path: PathBuf) {
        self.state = AppState::Processing;
        self.status_display.set_message("正在识别文字...");
        
        let tx = self.tx.clone();
        let ocr_engine = self.ocr_engine.clone();
        
        self.rt.spawn(async move {
            match ocr_engine.process_image(image, &path).await {
                Ok(result) => {
                    let _ = tx.send(AppMessage::OcrCompleted(result));
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::OcrError(e.to_string()));
                }
            }
        });
    }
    
    fn handle_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::ImageSelected(path) => {
                    self.reset_state();
                    self.handle_image_selected(path);
                }
                AppMessage::OcrCompleted(result) => {
                    self.state = AppState::Completed;
                    self.status_display.set_success(&format!(
                        "识别完成！置信度: {:.1}%, 用时: {:.0}ms", 
                        result.confidence * 100.0, 
                        result.processing_time
                    ));
                    self.result_panel.set_result(result.clone());
                    self.ocr_result = Some(result);
                }
                AppMessage::OcrError(error) => {
                    self.state = AppState::Error(error.clone());
                    self.status_display.set_error(&format!("识别失败: {}", error));
                }
            }
        }
    }
    
    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("🔍 OCR 文字识别工具");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 设置按钮
                if ui.button("⚙️").on_hover_text("设置").clicked() {
                    self.show_settings = !self.show_settings;
                }
                
                // 主题切换
                let theme_text = if self.dark_mode { "🌙" } else { "☀️" };
                if ui.button(theme_text).on_hover_text("切换主题").clicked() {
                    self.dark_mode = !self.dark_mode;
                }
                
                // 新建/重置按钮
                if ui.button("🆕 新建").clicked() {
                    self.reset_state();
                    self.selected_image_path = None;
                    self.current_image = None;
                    self.image_display = ImageDisplay::new();
                }
            });
        });
    }
    
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📁 选择图片").clicked() {
                self.handle_file_selection();
            }
            
            ui.separator();
            
            // 显示当前文件
            if let Some(path) = &self.selected_image_path {
                ui.label("📄");
                ui.label(path.file_name().unwrap_or_default().to_string_lossy());
            } else {
                ui.weak("未选择文件");
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 导出按钮
                if let Some(_result) = &self.ocr_result {
                    if ui.button("💾 导出结果").clicked() {
                        self.export_result();
                    }
                }
            });
        });
    }
    
    fn render_main_content(&mut self, ui: &mut egui::Ui) {
        // 使用可调整大小的面板布局
        egui::SidePanel::left("image_panel")
            .resizable(true)
            .default_width(500.0)
            .min_width(300.0)
            .max_width(800.0)
            .show_inside(ui, |ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.strong("📸 图片预览");
                        ui.separator();
                        
                        if self.image_display.has_image() {
                            let clicked = self.image_display.show(ui);
                            if clicked {
                                self.show_image_viewer = true;
                            }
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(50.0);
                                ui.label(egui::RichText::new("📎 拖拽图片到此处").size(18.0));
                                ui.weak("或点击选择图片按钮");
                                ui.add_space(20.0);
                                ui.weak("支持格式: PNG, JPG, BMP, TIFF, WebP, GIF");
                                ui.add_space(50.0);
                            });
                        }
                    });
                });
            });
        
        // 右侧结果区域
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.strong("📝 识别结果");
                    ui.separator();
                    
                    match &self.state {
                        AppState::Idle => {
                            ui.vertical_centered(|ui| {
                                ui.add_space(30.0);
                                ui.weak("请选择图片开始识别");
                                ui.add_space(30.0);
                            });
                        }
                        AppState::Loading => {
                            ui.vertical_centered(|ui| {
                                ui.add_space(30.0);
                                ui.spinner();
                                ui.label("正在加载图片...");
                                ui.add_space(30.0);
                            });
                        }
                        AppState::Processing => {
                            ui.vertical_centered(|ui| {
                                ui.add_space(30.0);
                                ui.spinner();
                                ui.label("正在识别文字...");
                                ui.weak("请稍候");
                                ui.add_space(30.0);
                            });
                        }
                        AppState::Completed => {
                            self.result_panel.show(ui);
                        }
                        AppState::Error(error) => {
                            ui.vertical_centered(|ui| {
                                ui.add_space(30.0);
                                ui.weak("识别失败，请重试");
                                ui.add_space(10.0);
                                ui.collapsing("查看详情", |ui| {
                                    ui.weak(error);
                                });
                                ui.add_space(30.0);
                            });
                        }
                    }
                });
            });
        });
    }
    
    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.status_display.show(ui);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(image) = &self.current_image {
                    ui.weak(format!("{}×{}", image.width(), image.height()));
                }
            });
        });
    }
    
    fn export_result(&self) {
        if let Some(result) = &self.ocr_result {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("ocr_result.txt")
                .add_filter("文本文件", &["txt"])
                .save_file()
            {
                let _ = std::fs::write(path, &result.text);
            }
        }
    }
    
    fn handle_drag_and_drop(&mut self, ctx: &egui::Context) {
        // 处理拖拽文件
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            
            for file in dropped_files {
                if let Some(path) = &file.path {
                    // 检查是否为图片文件
                    if let Some(extension) = path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "tiff" | "webp" | "gif") {
                            let _ = self.tx.send(AppMessage::ImageSelected(path.clone()));
                            break; // 只处理第一个图片文件
                        }
                    }
                }
            }
        }
    }
    
    fn render_image_viewer(&mut self, ctx: &egui::Context) {
        if let Some(image) = &self.current_image {
            let (img_width, img_height) = image.dimensions();
            
            egui::Window::new("🖼️ 图片查看器")
                .default_size(egui::vec2(
                    (img_width as f32 * 0.8).min(1200.0).max(600.0),
                    (img_height as f32 * 0.8).min(800.0).max(400.0)
                ))
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| {
                    // 顶部控制栏
                    ui.horizontal(|ui| {
                        ui.label(format!("尺寸: {}×{}", img_width, img_height));
                        ui.separator();
                        
                        // 缩放控制
                        ui.label("缩放:");
                        if ui.button("🔍−").on_hover_text("缩小").clicked() {
                            self.image_scale = (self.image_scale * 0.8).max(0.1);
                        }
                        ui.label(format!("{:.0}%", self.image_scale * 100.0));
                        if ui.button("🔍+").on_hover_text("放大").clicked() {
                            self.image_scale = (self.image_scale * 1.25).min(10.0);
                        }
                        if ui.button("1:1").on_hover_text("原始大小").clicked() {
                            self.image_scale = 1.0;
                        }
                        if ui.button("适应").on_hover_text("适应窗口").clicked() {
                            let available_size = ui.available_size();
                            let scale_x = (available_size.x - 40.0) / img_width as f32;
                            let scale_y = (available_size.y - 100.0) / img_height as f32;
                            self.image_scale = scale_x.min(scale_y).min(1.0);
                        }
                        
                        ui.separator();
                        if let Some(path) = &self.selected_image_path {
                            ui.label(format!("文件: {}", path.file_name().unwrap_or_default().to_string_lossy()));
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌ 关闭").clicked() {
                                self.show_image_viewer = false;
                                self.image_scale = 1.0; // 重置缩放
                            }
                        });
                    });
                    
                    ui.separator();
                    
                    // 显示可缩放的图片
                    egui::ScrollArea::both()
                        .id_salt("image_viewer_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // 处理鼠标滚轮缩放
                            if ui.rect_contains_pointer(ui.max_rect()) {
                                let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
                                if scroll_delta != 0.0 && ui.ctx().input(|i| i.modifiers.ctrl) {
                                    let zoom_factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                                    self.image_scale = (self.image_scale * zoom_factor).clamp(0.1, 10.0);
                                }
                            }
                            
                            if let Some(texture) = self.image_display.get_texture() {
                                let scaled_width = img_width as f32 * self.image_scale;
                                let scaled_height = img_height as f32 * self.image_scale;
                                
                                ui.add(
                                    egui::Image::from_texture(texture)
                                        .fit_to_exact_size(egui::vec2(scaled_width, scaled_height))
                                );
                            }
                        });
                        
                    // 底部提示
                    ui.horizontal(|ui| {
                        ui.weak("提示: 按住 Ctrl + 滚轮可以缩放图片");
                    });
                });
        }
    }
}

impl eframe::App for OcrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 禁用egui的调试信息和警告显示
        ctx.options_mut(|opt| {
            opt.warn_on_id_clash = false;
        });
        
        // 处理异步消息
        self.handle_messages();
        
        // 处理拖拽文件
        self.handle_drag_and_drop(ctx);
        
        // 设置主题
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        // 顶部面板
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            self.render_header(ui);
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            self.render_toolbar(ui);
            ui.add_space(8.0);
        });
        
        // 底部状态栏
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            self.render_status_bar(ui);
            ui.add_space(8.0);
        });
        
        // 主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            egui::ScrollArea::both()
                .id_salt("main_content_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.render_main_content(ui);
                });
        });
        
        // 设置窗口（如果显示）
        if self.show_settings {
            egui::Window::new("⚙️ 设置")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.dark_mode, "深色主题");
                    ui.separator();
                    if ui.button("关闭").clicked() {
                        self.show_settings = false;
                    }
                });
        }
        
        // 图片查看器窗口
        if self.show_image_viewer {
            self.render_image_viewer(ctx);
        }
        
        // 请求重绘
        ctx.request_repaint();
    }
} 