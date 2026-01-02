use printpdf::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::invoice::*;

use crate::pdf::*;

use crate::format::{format_currency, get_locale_by_code};

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
    let (subtotal, tax_totals, total) = invoice.calculate_summary();
    let locale = get_locale_by_code(&invoice.locale_code);

    if let Some(path) = logo_path {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        let image =
            RawImage::decode_from_bytes(&buf, &mut Vec::new()).map_err(|e| e.to_string())?;
        let image_id = doc.add_image(&image);
        ctx.current_ops.push(Op::UseXobject {
            id: image_id,
            transform: XObjectTransform {
                translate_x: Some(Mm(20.0).into()),
                translate_y: Some(Mm(255.0).into()),
                scale_x: Some(0.12),
                scale_y: Some(0.12),
                ..Default::default()
            },
        });
        ctx.y = Mm(245.0);
    }

    let col1 = Mm(20.0);
    let col2 = Mm(120.0);
    ctx.write_text_at(
        &format!("INVOICE ID: {}", invoice.number),
        14.0,
        col1,
        ctx.y,
    );
    ctx.write_text_at(
        &format!("PAYMENT DUE: {}", invoice.payment_due),
        10.0,
        col2,
        ctx.y,
    );
    ctx.y -= Mm(6.0);
    ctx.write_text_at(&format!("DATE: {}", invoice.date), 10.0, col1, ctx.y);
    ctx.write_text_at(
        &format!("DELIVERY DATE: {}", invoice.delivery_date),
        10.0,
        col2,
        ctx.y,
    );

    for (label, value) in &invoice.extra_info {
        ctx.y -= Mm(5.0);
        ctx.write_text_at(
            &format!("{}: {}", label.to_uppercase(), value),
            10.0,
            col1,
            ctx.y,
        );
    }

    ctx.y -= Mm(8.0);
    draw_line(&mut ctx.current_ops, col1, Mm(190.0), ctx.y);

    ctx.y -= Mm(8.0);
    let addr_y = ctx.y;
    ctx.write_text_at("SOLD BY:", 8.0, col1, addr_y);
    ctx.write_text_at("BILLED TO:", 8.0, col2, addr_y);
    ctx.y = addr_y - Mm(5.0);
    ctx.write_text(&invoice.seller.name, 10.0, col1);
    for line in invoice.seller.address.lines() {
        ctx.write_text(line, 9.0, col1);
    }
    ctx.write_text(&format!("VAT: {}", invoice.seller.vat_id), 9.0, col1);
    let seller_end_y = ctx.y;

    ctx.y = addr_y - Mm(5.0);
    ctx.write_text(&invoice.buyer.name, 10.0, col2);
    for line in invoice.buyer.address.lines() {
        ctx.write_text(line, 9.0, col2);
    }
    ctx.write_text(&invoice.buyer.email, 9.0, col2);
    let buyer_end_y = ctx.y;

    ctx.y = seller_end_y.min(buyer_end_y) - Mm(10.0);
    draw_line(&mut ctx.current_ops, col1, Mm(190.0), ctx.y);

    ctx.y -= Mm(8.0);
    ctx.write_text_at("Product", 9.0, col1, ctx.y);
    ctx.write_text_at("Units", 9.0, Mm(100.0), ctx.y);
    ctx.write_text_at("Unit Cost", 9.0, Mm(130.0), ctx.y);
    ctx.write_text_at("Total", 9.0, Mm(165.0), ctx.y);
    ctx.y -= Mm(5.0);

    for p in &invoice.products {
        if ctx.y < Mm(40.0) {
            ctx.pages.push(PdfPage::new(
                Mm(210.0),
                Mm(297.0),
                ctx.current_ops.drain(..).collect(),
            ));
            ctx.y = Mm(280.0);
        }
        let line_total = p.units as f64 * p.cost_per_unit;
        let row_y = ctx.y;
        ctx.write_text_at(&p.units.to_string(), 9.0, Mm(100.0), row_y);
        ctx.write_text_at(
            &format_currency(p.cost_per_unit, &invoice.currency_code, locale),
            9.0,
            Mm(130.0),
            row_y,
        );
        ctx.write_text_at(
            &format_currency(line_total, &invoice.currency_code, locale),
            9.0,
            Mm(165.0),
            row_y,
        );
        ctx.y = ctx.wrap_text_ops(&p.description, 75.0, 9.0, col1);
        ctx.y -= Mm(4.0);
    }

    ctx.y -= Mm(5.0);
    draw_line(&mut ctx.current_ops, col2, Mm(190.0), ctx.y);
    ctx.y -= Mm(8.0);
    ctx.write_text_at("Subtotal:", 9.0, col2, ctx.y);
    ctx.write_text_at(
        &format_currency(subtotal, &invoice.currency_code, locale),
        9.0,
        Mm(165.0),
        ctx.y,
    );
    for (rate, amt) in tax_totals {
        ctx.y -= Mm(5.0);
        ctx.write_text_at(&format!("Tax ({:.0}%):", *rate * 100.0), 9.0, col2, ctx.y);
        ctx.write_text_at(
            &format_currency(amt, &invoice.currency_code, locale),
            9.0,
            Mm(165.0),
            ctx.y,
        );
    }
    ctx.y -= Mm(8.0);
    ctx.write_text_at("TOTAL:", 12.0, col2, ctx.y);
    ctx.write_text_at(
        &format_currency(total, &invoice.currency_code, locale),
        12.0,
        Mm(165.0),
        ctx.y,
    );

    ctx.y -= Mm(15.0);
    if let Some(ref p_type) = invoice.payment_type {
        ctx.write_text_at(
            &format!("PAYMENT METHOD: {}", p_type.to_uppercase()),
            10.0,
            col1,
            ctx.y,
        );
        ctx.y -= Mm(6.0);
    }

    for (label, value) in &invoice.payment_info {
        if ctx.y < Mm(20.0) {
            ctx.pages.push(PdfPage::new(
                Mm(210.0),
                Mm(297.0),
                ctx.current_ops.drain(..).collect(),
            ));
            ctx.y = Mm(280.0);
        }
        ctx.write_text(&format!("{}: {}", label, value), 9.0, col1);
    }

    ctx.pages
        .push(PdfPage::new(Mm(210.0), Mm(297.0), ctx.current_ops));
    Ok(doc
        .with_pages(ctx.pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new()))
}
