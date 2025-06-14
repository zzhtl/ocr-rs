use eframe::egui;
use std::sync::Arc;

mod app;
mod ocr;
mod ui;

use app::OcrApp;
use ui::setup_custom_style;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("OCR 文字识别工具")
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "OCR文字识别工具",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            setup_custom_style(&cc.egui_ctx);
            Ok(Box::new(OcrApp::new(cc)))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 尝试加载中文字体
    match std::fs::read("assets/font/hei.ttf") {
        Ok(font_data) => {
            log::info!("成功加载中文字体文件: assets/font/hei.ttf");
            
            // 添加中文字体
            fonts.font_data.insert(
                "hei".to_owned(),
                Arc::new(egui::FontData::from_owned(font_data)),
            );
            
            // 设置字体族，优先使用中文字体
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "hei".to_owned());
            
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "hei".to_owned());
        }
        Err(e) => {
            log::warn!("无法加载中文字体文件: {}", e);
            log::info!("将使用系统默认字体，中文可能显示为方块");
        }
    }
    
    ctx.set_fonts(fonts);
}
