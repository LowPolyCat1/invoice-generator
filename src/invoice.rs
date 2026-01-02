use ordered_float::OrderedFloat;
use printpdf::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::format::{format_currency, get_locale_by_code};

pub struct Seller {
    pub name: String,
    pub address: String,
    pub vat_id: String,
    pub website: String,
}
pub struct Buyer {
    pub name: String,
    pub address: String,
    pub email: String,
}
pub struct Product {
    pub description: String,
    pub units: u32,
    pub cost_per_unit: f64,
    pub tax_rate: f64,
    pub tax_exempt_reason: Option<String>,
}
pub struct Invoice {
    pub number: String,
    pub date: String,
    pub seller: Seller,
    pub buyer: Buyer,
    pub payment_due: String,
    pub delivery_date: String,
    pub delivery_type: Option<String>,
    pub extra_info: Vec<(String, String)>,
    pub payment_type: Option<String>,
    pub payment_info: Vec<(String, String)>,
    pub products: Vec<Product>,
    pub currency_code: String,
    pub locale_code: String,
}

impl Invoice {
    pub fn calculate_summary(&self) -> (f64, BTreeMap<OrderedFloat<f64>, f64>, f64) {
        let mut subtotal = 0.0;
        let mut tax_totals = BTreeMap::new();
        for product in &self.products {
            let line_total = product.units as f64 * product.cost_per_unit;
            subtotal += line_total;
            if product.tax_rate > 0.0 {
                *tax_totals
                    .entry(OrderedFloat(product.tax_rate))
                    .or_insert(0.0) += line_total * product.tax_rate;
            }
        }
        let total = subtotal + tax_totals.values().sum::<f64>();
        (subtotal, tax_totals, total)
    }
}

pub struct PdfContext {
    pub font_id: FontId,
    pub pages: Vec<PdfPage>,
    pub current_ops: Vec<Op>,
    pub y: Mm,
}

impl PdfContext {
    pub fn new(font_id: FontId) -> Self {
        Self {
            font_id,
            pages: Vec::new(),
            current_ops: Vec::new(),
            y: Mm(280.0),
        }
    }

    pub fn write_text(&mut self, text: &str, size: f32, x: Mm) {
        if text.trim().is_empty() {
            return;
        }
        self.current_ops.push(Op::StartTextSection);
        self.current_ops.push(Op::SetTextCursor {
            pos: Point {
                x: x.into(),
                y: self.y.into(),
            },
        });
        self.current_ops.push(Op::SetFontSize {
            font: self.font_id.clone(),
            size: Pt(size),
        });
        self.current_ops.push(Op::WriteText {
            items: vec![TextItem::Text(text.to_string())],
            font: self.font_id.clone(),
        });
        self.current_ops.push(Op::EndTextSection);
        self.y -= Mm(size * 0.45);
    }

    pub fn write_text_at(&mut self, text: &str, size: f32, x: Mm, y: Mm) {
        self.current_ops.push(Op::StartTextSection);
        self.current_ops.push(Op::SetTextCursor {
            pos: Point {
                x: x.into(),
                y: y.into(),
            },
        });
        self.current_ops.push(Op::SetFontSize {
            font: self.font_id.clone(),
            size: Pt(size),
        });
        self.current_ops.push(Op::WriteText {
            items: vec![TextItem::Text(text.to_string())],
            font: self.font_id.clone(),
        });
        self.current_ops.push(Op::EndTextSection);
    }
}

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
        ctx.y = wrap_text_ops(&mut ctx, &p.description, 75.0, 9.0, col1);
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

pub fn draw_line(ops: &mut Vec<Op>, x1: Mm, x2: Mm, y: Mm) {
    ops.push(Op::SetOutlineColor {
        col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
    });
    ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point {
                        x: x1.into(),
                        y: y.into(),
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: x2.into(),
                        y: y.into(),
                    },
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
}

pub fn wrap_text_ops(ctx: &mut PdfContext, t: &str, max_w: f32, sz: f32, x: Mm) -> Mm {
    let words: Vec<&str> = t.split_whitespace().collect();
    let mut line = String::new();
    for word in words {
        if (line.len() + word.len()) as f32 * (sz * 0.16) > max_w {
            ctx.write_text(&line, sz, x);
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    ctx.write_text(&line, sz, x);
    ctx.y
}
