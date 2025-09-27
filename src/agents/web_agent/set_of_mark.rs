use std::collections::HashMap;
use image::{DynamicImage, Rgba, RgbaImage, ImageBuffer};
use imageproc::drawing::{draw_hollow_rect_mut, draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use rusttype::{Font, Scale, point};
use std::error::Error;

use crate::tools::chrome::types::{DOMRectangle, InteractiveRegion };

const TOP_NO_LABEL_ZONE: i32 = 20;

// Note: You need to provide a path to a TTF font file. For example, download DejaVuSans.ttf and use include_bytes!.
// For this code to compile, replace the path below with a valid font file path.
const FONT_DATA: &[u8] = include_bytes!("../../../dejavu-sans.book.ttf");



pub fn _add_set_of_mark(
    screenshot: &[u8],
    rois: &HashMap<String, InteractiveRegion>,
    use_sequential_ids: bool,
) -> Result<(DynamicImage, Vec<String>, Vec<String>, Vec<String>, HashMap<String, String>), Box<dyn Error>> {
    let base_img = image::load_from_memory(screenshot)?.to_rgba8();
    let width = base_img.width() as f32;
    let height = base_img.height() as f32;

    let mut visible_rects: Vec<String> = Vec::new();
    let mut rects_above: Vec<String> = Vec::new();
    let mut rects_below: Vec<String> = Vec::new();
    let mut id_mapping: HashMap<String, String> = HashMap::new();

    // 进行分类
    for (original_id, roi) in rois {
        let tag = &roi.tag_name;
        if tag == "option" || tag == "input, type=file" {
            // 对于option和file input，只添加到可见列表但不绘制
            if !visible_rects.contains(original_id) {
                visible_rects.push(original_id.clone());
            }
            continue;
        }

        for rect in &roi.rects {
            if rect.width * rect.height == 0.0 || rect.width == 0.0 || rect.height == 0.0 {
                continue;
            }

            let mid_x = (rect.right + rect.left) / 2.0;
            let mid_y = (rect.bottom + rect.top) / 2.0;

            if 0.0 <= mid_x && mid_x < width.into() {
                if mid_y < 0.0 && !rects_above.contains(original_id) {
                    rects_above.push(original_id.clone());
                } else if mid_y >= height.into() && !rects_below.contains(original_id) {
                    rects_below.push(original_id.clone());
                } else if 0.0 <= mid_y && mid_y < height.into() && !visible_rects.contains(original_id) {
                    visible_rects.push(original_id.clone());
                }
            }
        }
    }

    // Create new sequential IDs
    let mut next_id: u32 = 1;
    let mut original_to_new: HashMap<String, String> = HashMap::new();

    let map_ids = |original_ids: &[String], next_id: &mut u32, id_mapping: &mut HashMap<String, String>, original_to_new: &mut HashMap<String, String>| -> Vec<String> {
        let mut new_ids = Vec::new();
        for original in original_ids {
            let new_id = next_id.to_string();
            id_mapping.insert(new_id.clone(), original.clone());
            original_to_new.insert(original.clone(), new_id.clone());
            new_ids.push(new_id);
            *next_id += 1;
        }
        new_ids
    };

    let (new_visible_rects, new_rects_above, new_rects_below) = if use_sequential_ids {
        let new_visible = map_ids(&visible_rects, &mut next_id, &mut id_mapping, &mut original_to_new);
        let new_above = map_ids(&rects_above, &mut next_id, &mut id_mapping, &mut original_to_new);
        let new_below = map_ids(&rects_below, &mut next_id, &mut id_mapping, &mut original_to_new);
        (new_visible, new_above, new_below)
    } else {
        let new_visible = visible_rects.clone();
        let new_above = rects_above.clone();
        let new_below = rects_below.clone();
        for list in vec![&visible_rects, &rects_above, &rects_below] {
            for id in list {
                id_mapping.insert(id.clone(), id.clone());
                original_to_new.insert(id.clone(), id.clone());
            }
        }
        (new_visible, new_above, new_below)
    };

    // Load font
    let font = Font::try_from_bytes(FONT_DATA).ok_or("Failed to load font")?;
    let scale = Scale { x: 14.0, y: 14.0 };

    // Create overlay
    let mut overlay: RgbaImage = ImageBuffer::from_fn(base_img.width(), base_img.height(), |_,_| Rgba([0, 0, 0, 0]));

    // Drawing
    for (original_id, roi) in rois {
        let tag = &roi.tag_name;
        if tag == "option" {
            continue;
        }

        if let Some(new_id) = original_to_new.get(original_id) {
            for rect in &roi.rects {
                if rect.width * rect.height == 0.0 {
                    continue;
                }

                let mid_x = (rect.right + rect.left) / 2.0;
                let mid_y = (rect.bottom + rect.top) / 2.0;

                if 0.0 <= mid_x && mid_x < width.into() && 0.0 <= mid_y && mid_y < height.into() {
                    _draw_roi(&mut overlay, new_id, &font, scale, rect)?;
                }
            }
        }
    }

    // Composite overlay onto base
    let mut comp = base_img.clone();
    image::imageops::overlay(&mut comp, &overlay, 0, 0);

    let final_img = DynamicImage::ImageRgba8(comp);

    Ok((final_img, new_visible_rects, new_rects_above, new_rects_below, id_mapping))
}

fn _draw_roi(
    draw: &mut RgbaImage,
    idx: &str,
    font: &Font<'_>,
    scale: Scale,
    rect: &DOMRectangle,
) -> Result<(), Box<dyn Error>> {
    let color = Rgba([255, 0, 0, 255]);
    let text_color = Rgba([255, 255, 255, 255]);

    let left = rect.left.round() as i32;
    let top = rect.top.round() as i32;
    let right = rect.right.round() as i32;
    let bottom = rect.bottom.round() as i32;

    // Draw rectangle outline
    let roi_rect = Rect::at(left, top).of_size((right - left) as u32, (bottom - top) as u32);
    draw_hollow_rect_mut(draw, roi_rect, color);

    // Adjust label position
    let label_x = right;
    let mut label_y = top;
    let mut anchor_rb = true;  // true for "rb" (right-bottom), false for "rt" (right-top)

    if label_y <= TOP_NO_LABEL_ZONE {
        label_y = bottom;
        anchor_rb = false;
    }

    // Calculate text metrics
    let v_metrics = font.v_metrics(scale);
    let text = idx.to_string();
    let glyphs: Vec<_> = font.layout(&text, scale, point(0.0, v_metrics.ascent)).collect();
    let text_width = glyphs.last().map(|g| g.position().x + g.unpositioned().h_metrics().advance_width).unwrap_or(0.0).round() as i32;
    let text_height = (v_metrics.ascent - v_metrics.descent).ceil() as i32;

    // Adjust for anchor
    let text_x = if anchor_rb {
        label_x - text_width - 3  // Align right
    } else {
        label_x - text_width - 3
    };
    let text_y = if anchor_rb {
        label_y - text_height - 3  // Baseline above bottom
    } else {
        label_y + 3  // Baseline below top
    };

    // Draw background rectangle for label
    let bbox_left = text_x - 3;
    let bbox_top = text_y - 3;
    let bbox_width = text_width + 6;
    let bbox_height = text_height + 6;
    let bbox_rect = Rect::at(bbox_left, bbox_top).of_size(bbox_width as u32, bbox_height as u32);
    draw_filled_rect_mut(draw, bbox_rect, color);

    // Draw text (note: draw_text_mut expects u32 for positions)
    draw_text_mut(draw, text_color, (text_x as u32).try_into().unwrap(), (text_y as u32).try_into().unwrap(), scale, font, &text);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_set_of_mark() {
        let screenshot = include_bytes!("screenshot.png");
        use std::fs;

        // 读取 rois.json 文件内容并反序列化为 HashMap<String, InteractiveRegion>
        let rois_data = fs::read("src/agents/web_agent/rois.json").expect("无法读取 rois.json 文件");
        let rois: std::collections::HashMap<String, InteractiveRegion> = serde_json::from_slice(&rois_data).expect("rois.json 解析失败");

        let (img, visible_rects, rects_above, rects_below, id_mapping) = _add_set_of_mark(screenshot, &rois, true).unwrap();
        img.save("screenshot_with_mark.png").unwrap();
        println!("visible_rects: {:?}", visible_rects);
        println!("rects_above: {:?}", rects_above);
        println!("rects_below: {:?}", rects_below);
        println!("id_mapping: {:?}", id_mapping);
    }
}