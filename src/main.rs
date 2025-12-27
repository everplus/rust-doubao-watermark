use arboard::ImageData;
use image::{GenericImage, GenericImageView};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

fn get_desktop_path() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let mut desktop = PathBuf::from(home);
    desktop.push("Desktop");
    desktop
}

fn clear_clipboard() -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("无法访问剪贴板: {}", e))?;
    clipboard
        .set_text("")
        .map_err(|e| format!("无法清空剪贴板: {}", e))?;
    Ok(())
}

fn wait_for_image() -> Result<ImageData<'static>, String> {
    loop {
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| format!("无法访问剪贴板: {}", e))?;

        match clipboard.get_image() {
            Ok(img) => {
                // 检查图片是否有效（有宽高）
                if img.width > 0 && img.height > 0 {
                    return Ok(img);
                }
            }
            Err(_) => {
                // 剪贴板没有图片，继续等待
            }
        }

        thread::sleep(Duration::from_millis(200));
    }
}

fn convert_to_png(image_data: &ImageData) -> Result<image::DynamicImage, String> {
    // 剪贴板返回的是原始像素数据（RGBA 格式），需要直接构建图片
    let width = image_data.width;
    let height = image_data.height;

    // 验证数据长度是否正确
    let expected_len = width * height * 4; // RGBA = 4 bytes per pixel
    if image_data.bytes.len() != expected_len {
        return Err(format!(
            "图片数据长度不匹配，期望 {} 字节，实际 {} 字节",
            expected_len,
            image_data.bytes.len()
        ));
    }

    // 从原始 RGBA 像素数据创建图片（需要转换为 Vec<u8>）
    let rgba_image =
        image::RgbaImage::from_raw(width as u32, height as u32, image_data.bytes.to_vec())
            .ok_or_else(|| "无法从原始像素数据创建图片".to_string())?;

    Ok(image::DynamicImage::ImageRgba8(rgba_image))
}

fn display_image(img: &image::DynamicImage, title: &str) {
    println!("\n[图片预览] {}", title);
    println!("尺寸: {}x{}", img.width(), img.height());
    println!("格式: RGBA");
    println!("{}", "─".repeat(60));

    // 创建超时保护机制
    let (tx, rx) = mpsc::channel();
    let img_clone = img.clone();

    // 在单独的线程中尝试显示图片
    thread::spawn(move || {
        let config = viuer::Config {
            // 只设置宽度，高度会按比例自动计算以保持原始宽高比
            width: Some(80),
            // 不设置 height，让它自动按比例计算
            x: 0,
            y: 0,
            restore_cursor: false,
            absolute_offset: false,
            ..Default::default()
        };
        let result = viuer::print(&img_clone, &config);
        let _ = tx.send(result);
    });

    // 等待最多 2 秒
    let timeout = Duration::from_secs(2);
    let start = SystemTime::now();
    let mut completed = false;

    while start.elapsed().unwrap_or(Duration::ZERO) < timeout {
        match rx.try_recv() {
            Ok(_) => {
                completed = true;
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(_) => break,
        }
    }

    if !completed {
        println!("（图片显示超时，已跳过预览）");
    }

    println!("{}", "─".repeat(60));
}

fn stitch_images(
    img1: &image::DynamicImage,
    img2: &image::DynamicImage,
) -> Result<image::DynamicImage, String> {
    let (width1, height1) = img1.dimensions();
    let (width2, height2) = img2.dimensions();

    if width1 != width2 || height1 != height2 {
        return Err(format!(
            "图片尺寸不一致！图片一: {}x{}, 图片二: {}x{}",
            width1, height1, width2, height2
        ));
    }

    let mid_y = height1 / 2;

    // 裁剪上半部分（使用图片二的上半部分）
    let cropped_top = img2.crop_imm(0, 0, width2, mid_y);
    // 裁剪下半部分（使用图片一的下半部分）
    let cropped_bottom = img1.crop_imm(0, mid_y, width1, height1 - mid_y);

    // 创建最终拼接图片
    let mut result = image::DynamicImage::new_rgb8(width1, height1);
    result
        .copy_from(&cropped_top, 0, 0)
        .map_err(|e| format!("拼接上半部分失败: {}", e))?;
    result
        .copy_from(&cropped_bottom, 0, mid_y)
        .map_err(|e| format!("拼接下半部分失败: {}", e))?;

    Ok(result)
}

fn save_image(img: &image::DynamicImage) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    let filename = format!("doubao_image_{}.png", timestamp);
    let desktop = get_desktop_path();
    let filepath = desktop.join(&filename);

    img.save(&filepath)
        .map_err(|e| format!("保存图片失败: {}", e))?;

    Ok(filepath)
}

fn print_separator() {
    println!("{}", "=".repeat(60));
}

fn print_step(step: u32, title: &str) {
    print_separator();
    println!("[步骤 {}] {}", step, title);
    print_separator();
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           豆包AI图片去水印工具 v1.0                       ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    // 步骤0：清空剪贴板
    print_step(0, "初始化");
    println!("正在清空剪贴板...");
    if let Err(e) = clear_clipboard() {
        eprintln!("警告: {}", e);
    } else {
        println!("✓ 剪贴板已清空");
    }

    // 步骤1：获取图片一（复制图片）
    print_step(1, "获取上半部分图片");
    println!("请按以下步骤操作：");
    println!("  1. 在浏览器中将生成的大图拖动到新Tab");
    println!("  2. 右键点击图片，选择「复制图片」");
    println!("  3. 关闭Tab");
    println!("\n正在监听剪贴板变化...");

    let image1_data = match wait_for_image() {
        Ok(img) => {
            println!("✓ 已获取图片一（尺寸: {}x{}）", img.width, img.height);
            img
        }
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    };

    // 转换并显示图片一
    let image1 = match convert_to_png(&image1_data) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("✗ 图片一转换失败: {}", e);
            std::process::exit(1);
        }
    };
    display_image(&image1, "上半部分图片（复制图片）");

    // 清空剪贴板，为获取第二张图片做准备
    println!("正在清空剪贴板...");
    if let Err(e) = clear_clipboard() {
        eprintln!("警告: {}", e);
    } else {
        println!("✓ 剪贴板已清空");
    }

    // 等待用户完成下一步操作
    thread::sleep(Duration::from_millis(500));

    // 步骤2：获取图片二（直接复制）
    print_step(2, "获取下半部分图片");
    println!("请按以下步骤操作：");
    println!("  1. 直接右键点击生成的大图");
    println!("  2. 选择「复制」菜单项");
    println!("\n正在监听剪贴板变化...");

    let image2_data = match wait_for_image() {
        Ok(img) => {
            println!("✓ 已获取图片二（尺寸: {}x{}）", img.width, img.height);
            img
        }
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    };

    // 转换并显示图片二
    let image2 = match convert_to_png(&image2_data) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("✗ 图片二转换失败: {}", e);
            std::process::exit(1);
        }
    };
    display_image(&image2, "下半部分图片（直接复制）");

    // 拼接图片
    print_step(3, "拼接图片");
    println!("正在进行图片拼接...");

    let result = match stitch_images(&image1, &image2) {
        Ok(img) => {
            println!("✓ 图片拼接完成");
            img
        }
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    };

    // 显示拼接后的结果
    display_image(&result, "拼接后的完整图片");

    // 保存图片
    print_step(4, "保存结果");
    let filepath = match save_image(&result) {
        Ok(path) => {
            println!("✓ 图片已保存至:");
            println!("  {}", path.display());
            path
        }
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    };

    print_separator();
    println!("🎉 处理完成！");
    println!("最终图片尺寸: {}x{}", result.width(), result.height());
    println!("保存位置: {}", filepath.display());
    print_separator();
}
