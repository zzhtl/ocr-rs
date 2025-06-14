use eframe::egui;
use image::{DynamicImage, GenericImageView};
use crate::ocr::OcrResult;

pub struct ImageDisplay {
    texture: Option<egui::TextureHandle>,
    image_size: Option<(u32, u32)>,
    image_data: Option<DynamicImage>,
}

impl ImageDisplay {
    pub fn new() -> Self {
        Self {
            texture: None,
            image_size: None,
            image_data: None,
        }
    }
    
    pub fn set_image(&mut self, image: DynamicImage) {
        let (width, height) = image.dimensions();
        self.image_size = Some((width, height));
        self.image_data = Some(image);
        self.texture = None; // 重置纹理，将在show中重新创建
    }
    
    pub fn has_image(&self) -> bool {
        self.image_size.is_some()
    }
    
    pub fn get_texture(&self) -> Option<&egui::TextureHandle> {
        self.texture.as_ref()
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        let mut clicked = false;
        
        if let Some((width, height)) = self.image_size {
            // 计算显示尺寸，保持宽高比
            let available_size = ui.available_size();
            let max_width = (available_size.x - 20.0).max(300.0);
            let max_height = (available_size.y - 100.0).max(200.0);
            
            let aspect_ratio = width as f32 / height as f32;
            let (display_width, display_height) = if aspect_ratio > max_width / max_height {
                (max_width, max_width / aspect_ratio)
            } else {
                (max_height * aspect_ratio, max_height)
            };
            
            // 如果还没有纹理，从图像数据创建
            if self.texture.is_none() {
                if let Some(image) = &self.image_data {
                    self.texture = Some(create_texture_from_image(ui.ctx(), image, "main_image"));
                }
            }
            
            if let Some(texture) = &self.texture {
                ui.vertical_centered(|ui| {
                    // 添加可点击的图片
                    let image_response = ui.add(
                        egui::Image::from_texture(texture)
                            .fit_to_exact_size(egui::vec2(display_width, display_height))
                            .sense(egui::Sense::click())
                    );
                    
                    if image_response.clicked() {
                        clicked = true;
                    }
                    
                    // 鼠标悬停提示
                    if image_response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        image_response.on_hover_text("点击查看原图");
                    }
                    
                    ui.add_space(8.0);
                    ui.weak(format!("原始尺寸: {}×{}", width, height));
                    ui.weak("点击图片查看原图");
                });
            }
        }
        
        clicked
    }
}

// 状态显示组件
pub struct StatusDisplay {
    message: String,
    status_type: StatusType,
}

#[derive(Debug, Clone, PartialEq)]
enum StatusType {
    None,
    Info,
    Success,
    Error,
}

impl StatusDisplay {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            status_type: StatusType::None,
        }
    }
    
    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
        self.status_type = StatusType::Info;
    }
    
    pub fn set_success(&mut self, message: &str) {
        self.message = message.to_string();
        self.status_type = StatusType::Success;
    }
    
    pub fn set_error(&mut self, message: &str) {
        self.message = message.to_string();
        self.status_type = StatusType::Error;
    }
    
    pub fn clear(&mut self) {
        self.message.clear();
        self.status_type = StatusType::None;
    }
    
    pub fn show(&self, ui: &mut egui::Ui) {
        if !self.message.is_empty() {
            let (icon, color) = match self.status_type {
                StatusType::Info => ("ℹ️", egui::Color32::from_rgb(100, 149, 237)),
                StatusType::Success => ("✅", egui::Color32::from_rgb(34, 139, 34)),
                StatusType::Error => ("❌", egui::Color32::from_rgb(220, 20, 60)),
                StatusType::None => return,
            };
            
            ui.horizontal(|ui| {
                ui.label(icon);
                ui.colored_label(color, &self.message);
            });
        } else {
            ui.weak("就绪");
        }
    }
}

// 结果面板组件
pub struct ResultPanel {
    result: Option<OcrResult>,
    text_content: String,
    show_details: bool,
    preserve_whitespace: bool,
    font_size: f32,
    line_spacing: f32,
}

impl ResultPanel {
    pub fn new() -> Self {
        Self {
            result: None,
            text_content: String::new(),
            show_details: false,
            preserve_whitespace: true,
            font_size: 14.0,
            line_spacing: 1.2,
        }
    }
    
    pub fn set_result(&mut self, result: OcrResult) {
        self.text_content = result.text.clone();
        self.result = Some(result);
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let result = match &self.result {
            Some(r) => r.clone(),
            None => return,
        };
        
        // 简化的格式控制选项，默认收起
        ui.collapsing("🔧 显示选项", |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.preserve_whitespace, "保持空格格式");
                ui.separator();
                ui.label("字体大小:");
                ui.add(egui::Slider::new(&mut self.font_size, 10.0..=20.0));
            });
        });
        
        ui.add_space(4.0);
        
        // 文本内容显示区域 - 保持原有格式
        ui.group(|ui| {
            ui.strong("识别内容:");
            ui.separator();
            
            // 计算可用高度，为其他UI元素留出空间
            let available_height = ui.available_height() - 120.0; // 为按钮和其他元素留出空间
            let scroll_height = available_height.max(200.0).min(600.0); // 最小200px，最大600px
            
            egui::ScrollArea::vertical()
                .id_salt("ocr_result_display")
                .max_height(scroll_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // 设置等宽字体
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(self.font_size));
                    
                    if self.preserve_whitespace {
                        // 保持原有格式模式 - 逐行显示
                        for line in self.text_content.lines() {
                            if line.trim().is_empty() {
                                // 空行显示为空白行
                                ui.add_space(ui.text_style_height(&egui::TextStyle::Body));
                            } else {
                                // 保持行内的空格和制表符
                                let formatted_line = line.replace('\t', "    ");
                                ui.label(&formatted_line);
                            }
                        }
                    } else {
                        // 标准格式模式 - 使用可选择的标签
                        ui.add(
                            egui::TextEdit::multiline(&mut self.text_content.clone())
                                .desired_width(f32::INFINITY)
                                .interactive(false)
                        );
                    }
                });
        });
        
        ui.add_space(8.0);
        
        // 操作按钮 - 简化版
        ui.horizontal(|ui| {
            let copy_clicked = ui.button("📋 复制").clicked();
            let save_file_clicked = ui.button("💾 保存").clicked();
            let show_details_clicked = ui.button("🔍 详情").clicked();
            
            // 处理按钮点击事件
            if copy_clicked {
                let text_to_copy = if self.preserve_whitespace {
                    self.text_content.clone()
                } else {
                    self.text_content
                        .lines()
                        .map(|line| line.trim())
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                ui.ctx().copy_text(text_to_copy);
            }
            
            if save_file_clicked {
                self.save_to_file();
            }
            
            if show_details_clicked {
                self.show_details = !self.show_details;
            }
        });

        
        ui.add_space(8.0);
        
        // 详细信息
        if self.show_details {
            ui.group(|ui| {
                ui.strong("详细信息:");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("置信度:");
                    ui.strong(format!("{:.1}%", result.confidence * 100.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("处理时间:");
                    ui.strong(format!("{:.0}ms", result.processing_time));
                });
                
                ui.horizontal(|ui| {
                    ui.label("文字长度:");
                    ui.strong(format!("{} 字符", result.text.len()));
                });
                
                ui.horizontal(|ui| {
                    ui.label("行数:");
                    ui.strong(format!("{} 行", result.text.lines().count()));
                });
                
                ui.horizontal(|ui| {
                    ui.label("非空行数:");
                    let non_empty_lines = result.text.lines().filter(|line| !line.trim().is_empty()).count();
                    ui.strong(format!("{} 行", non_empty_lines));
                });
                
                if !result.bounding_boxes.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("检测区域:");
                        ui.strong(format!("{} 个", result.bounding_boxes.len()));
                    });
                }
            });
        }
    }
    
    fn save_to_file(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("ocr_result.txt")
            .add_filter("文本文件", &["txt"])
            .save_file()
        {
            let content = if self.preserve_whitespace {
                self.text_content.clone()
            } else {
                self.text_content
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let _ = std::fs::write(path, content);
        }
    }
}

// 辅助函数：创建简单的图像纹理
pub fn create_texture_from_image(
    ctx: &egui::Context,
    image: &DynamicImage,
    name: &str,
) -> egui::TextureHandle {
    let rgba_image = image.to_rgba8();
    let (width, height) = image.dimensions();
    let pixels = rgba_image.as_flat_samples();
    
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        pixels.as_slice(),
    );
    
    ctx.load_texture(name, color_image, egui::TextureOptions::default())
}

// UI样式辅助函数
pub fn setup_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // 设置现代化样式
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.indent = 20.0;
    
    ctx.set_style(style);
}

// 错误显示组件
pub struct ErrorDisplay {
    message: String,
    show_details: bool,
}

impl ErrorDisplay {
    pub fn new(message: String) -> Self {
        Self {
            message,
            show_details: false,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::RED, "❌ 错误:");
            ui.label(&self.message);
            
            if ui.button("详情").clicked() {
                self.show_details = !self.show_details;
            }
        });
        
        if self.show_details {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("错误详情:");
                    ui.code(&self.message);
                });
            });
        }
    }
}

// 进度指示器组件
pub struct ProgressIndicator {
    current: usize,
    total: usize,
    message: String,
}

impl ProgressIndicator {
    pub fn new(total: usize, message: String) -> Self {
        Self {
            current: 0,
            total,
            message,
        }
    }
    
    pub fn set_progress(&mut self, current: usize) {
        self.current = current;
    }
    
    pub fn show(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(&self.message);
            
            let progress = if self.total > 0 {
                self.current as f32 / self.total as f32
            } else {
                0.0
            };
            
            ui.add(egui::ProgressBar::new(progress).show_percentage());
            ui.label(format!("{}/{}", self.current, self.total));
        });
    }
} 