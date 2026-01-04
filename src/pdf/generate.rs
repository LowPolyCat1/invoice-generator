use crate::invoice::*;
use crate::pdf::addresses::draw_address_section;
use crate::pdf::fin_summary::draw_financial_summary;
use crate::pdf::header::draw_header_info;
use crate::pdf::logo::draw_logo;
use crate::pdf::payment_details::draw_payment_details;
use crate::pdf::product_table::draw_product_table;
use crate::pdf::*;
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

    draw_financial_summary(&mut ctx, invoice, &locale, subtotal, &tax_totals, total);

    draw_payment_details(&mut ctx, invoice, summary_top);

    ctx.pages.push(PdfPage::new(
        PAGE_WIDTH,
        PAGE_HEIGHT,
        ctx.current_ops.clone(),
    ));

    let mut pages = ctx.pages.clone();
    let total_pages = pages.len();

    for (i, page) in pages.iter_mut().enumerate() {
        let page_number = i + 1;
        let pagination_text = format!("{} / {}", page_number, total_pages);

        let x_pos = PAGE_WIDTH - RIGHT_MARGIN - Mm(20.0);
        let y_pos = BOTTOM_MARGIN;
        let p_width = Mm(20.0).0;

        ctx.current_ops = Vec::new();

        ctx.write_text_at_wrapping(&pagination_text, 10.0, x_pos, y_pos, p_width);

        page.ops.append(&mut ctx.current_ops);
    }

    Ok(doc
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new()))
}
