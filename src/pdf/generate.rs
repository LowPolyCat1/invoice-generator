use crate::invoice::Invoice;
use crate::pdf::context::PdfContext;
use crate::pdf::layout::{
    draw_address_section, draw_financial_summary, draw_header_info, draw_payment_details,
    draw_product_table,
};
use crate::pdf::{LEFT_MARGIN, PAGE_HEIGHT, PAGE_WIDTH};
use printpdf::{Mm, PdfDocument, PdfPage, PdfSaveOptions, ParsedFont};
use std::path::Path;

pub fn generate_invoice_pdf<P: AsRef<Path>>(
    invoice: &Invoice,
    font_path: P,
    logo_path: Option<P>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::new("Invoice");
    let font_bytes = std::fs::read(font_path)?;
    let font_id =
        doc.add_font(&ParsedFont::from_bytes(&font_bytes, 0, &mut Vec::new()).ok_or("Font Error")?);
    let mut ctx = PdfContext::new(font_id);
    let locale = &invoice.locale;

    draw_logo(
        &mut ctx,
        logo_path.as_ref().map(|p| p.as_ref()),
        LEFT_MARGIN,
        &mut doc,
    )?;
    draw_header_info(&mut ctx, invoice);
    draw_address_section(&mut ctx, invoice);

    let (subtotal, tax_totals, total) = invoice.calculate_summary();
    draw_product_table(&mut ctx, invoice, &locale)?;

    let summary_top = ctx.y - Mm(8.0);
    draw_financial_summary(&mut ctx, &locale, subtotal, &tax_totals, total);
    draw_payment_details(&mut ctx, invoice, summary_top);

    ctx.pages.push(PdfPage::new(
        PAGE_WIDTH,
        PAGE_HEIGHT,
        ctx.current_ops.clone(),
    ));
    let pages = ctx.pages.clone();

    let pdf_bytes = doc
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());

    Ok(pdf_bytes)
}

fn draw_logo(
    ctx: &mut PdfContext,
    logo_path: Option<&Path>,
    left_margin: f32,
    doc: &mut PdfDocument,
) -> Result<(), Box<dyn std::error::Error>> {
    use printpdf::*;
    use std::fs::File;
    use std::io::Read;

    if let Some(path) = logo_path {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        let image =
            RawImage::decode_from_bytes(&buf, &mut Vec::new()).map_err(|e| e.to_string())?;

        let max_width_mm = 70.0;
        let max_height_mm = 40.0;
        let page_height_mm = 297.0;
        let top_margin_mm = 10.0;

        let dpi_x = (image.width as f32 / max_width_mm) * 25.4;
        let dpi_y = (image.height as f32 / max_height_mm) * 25.4;

        let target_dpi = dpi_x.max(dpi_y);

        let actual_height_pt = (image.height as f32 * 72.0) / target_dpi;

        let top_y_pt = Mm(page_height_mm - top_margin_mm).into_pt().0;
        let bottom_y_pt = top_y_pt - actual_height_pt;

        let image_id = doc.add_image(&image);
        ctx.current_ops.push(Op::UseXobject {
            id: image_id,
            transform: XObjectTransform {
                translate_x: Some(Mm(left_margin).into_pt()),
                translate_y: Some(Pt(bottom_y_pt)),
                scale_x: None,
                scale_y: None,
                dpi: Some(target_dpi),
                ..Default::default()
            },
        });

        ctx.y = Mm((bottom_y_pt / 2.83465) - 10.0);
    } else {
        ctx.y = Mm(280.0);
    }
    Ok(())
}
