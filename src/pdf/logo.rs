use printpdf::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::PdfContext;

pub fn draw_logo(
    ctx: &mut PdfContext,
    logo_path: Option<&Path>,
    left_margin: f32,
    doc: &mut PdfDocument,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = logo_path {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        let image =
            RawImage::decode_from_bytes(&buf, &mut Vec::new()).map_err(|e| e.to_string())?;

        let box_width_mm = 70.0;
        let box_height_mm = 40.0;
        let top_margin_mm = 10.0;
        let page_height_mm = 297.0;

        let scale_x = Mm(box_width_mm).into_pt().0 / image.width as f32;
        let scale_y = Mm(box_height_mm).into_pt().0 / image.height as f32;

        let scale = scale_x.min(scale_y);

        let actual_height_pt = image.height as f32 * scale;

        let top_y_pt = Mm(page_height_mm - top_margin_mm).into_pt().0;
        let bottom_y_pt = top_y_pt - actual_height_pt;

        let image_id = doc.add_image(&image);
        ctx.current_ops.push(Op::UseXobject {
            id: image_id,
            transform: XObjectTransform {
                translate_x: Some(Mm(left_margin).into_pt()),
                translate_y: Some(Pt(bottom_y_pt)),
                scale_x: Some(scale),
                scale_y: Some(scale),
                dpi: Some(72.0),
                ..Default::default()
            },
        });

        ctx.y = Mm((bottom_y_pt / 2.83465) - 10.0);
    } else {
        ctx.y = Mm(280.0);
    }
    Ok(())
}
