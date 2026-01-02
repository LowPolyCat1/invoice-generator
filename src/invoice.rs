use ::image::ImageReader;
use num_format::Locale;
use ordered_float::OrderedFloat;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::{collections::BTreeMap, vec};

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

    fn seller_as_lines(&self) -> Vec<&str> {
        let mut lines = Vec::new();
        lines.push(self.seller.name.as_str());
        for line in self.seller.address.lines() {
            lines.push(line);
        }
        lines.push(self.seller.vat_id.as_str());
        lines.push(self.seller.website.as_str());
        lines
    }

    fn buyer_as_lines(&self) -> Vec<&str> {
        let mut lines = Vec::new();
        lines.push(self.buyer.name.as_str());
        for line in self.buyer.address.lines() {
            lines.push(line);
        }
        lines.push(self.buyer.email.as_str());
        lines
    }
}

pub struct PdfContext<'a> {
    pub doc: &'a PdfDocumentReference,
    pub font: IndirectFontRef,
    pub current_layer: PdfLayerReference,
    pub y: Mm,
}

impl<'a> PdfContext<'a> {
    pub fn check_page_break(&mut self, required_space: Mm) {
        if self.y < required_space {
            let (page, layer) = self.doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            self.current_layer = self.doc.get_page(page).get_layer(layer);
            self.y = Mm(270.0);
        }
    }

    pub fn use_text(&mut self, text: &str, size: f32, x: Mm) {
        self.current_layer
            .use_text(text, size, x, self.y, &self.font);
        self.y -= Mm(size * 0.4);
    }

    pub fn use_text_at(&self, text: &str, size: f32, x: Mm, y: Mm) {
        self.current_layer.use_text(text, size, x, y, &self.font);
    }
}

pub fn generate_invoice_pdf<P: AsRef<Path>>(
    invoice: &Invoice,
    font_path: P,
    logo_path: Option<P>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (doc, page1, layer1) = PdfDocument::new("Invoice", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_external_font(File::open(font_path)?)?;

    let mut ctx = PdfContext {
        doc: &doc,
        font,
        current_layer: doc.get_page(page1).get_layer(layer1),
        y: Mm(290.0),
    };

    let locale = get_locale_by_code(&invoice.locale_code);
    let (subtotal, tax_totals, total) = invoice.calculate_summary();

    if let Some(path) = logo_path {
        draw_logo(&mut ctx, path.as_ref(), 1.0, 1.0)?;
    }

    let margin_left = Mm(20.0);
    let col2_x = Mm(110.0);
    let font_text = 10.0;
    let font_sub = 14.0;

    let info_y_start = ctx.y;
    ctx.use_text_at(
        &format!("Payment Due: {}", invoice.payment_due),
        font_text,
        col2_x,
        info_y_start,
    );
    ctx.use_text_at(
        &format!("Delivery Date: {}", invoice.delivery_date),
        font_text,
        col2_x,
        info_y_start - Mm(6.0),
    );

    ctx.y -= Mm(15.0);
    draw_line(&ctx.current_layer, margin_left, Mm(190.0), ctx.y);

    ctx.y -= Mm(10.0);
    ctx.use_text_at(
        &format!("Date: {}", invoice.date),
        font_text,
        margin_left,
        ctx.y,
    );
    ctx.use_text_at(
        &format!("Invoice ID: {}", invoice.number),
        font_text,
        col2_x,
        ctx.y,
    );

    ctx.y -= Mm(10.0);
    draw_line(&ctx.current_layer, margin_left, Mm(190.0), ctx.y);

    ctx.y -= Mm(10.0);
    ctx.use_text_at("Sold by", font_sub, margin_left, ctx.y);
    ctx.use_text_at("Billed to", font_sub, col2_x, ctx.y);
    ctx.y -= Mm(10.0);

    let party_y = ctx.y;
    draw_address_block(&mut ctx, &invoice.seller_as_lines(), margin_left);
    let seller_end = ctx.y;
    ctx.y = party_y;
    draw_address_block(&mut ctx, &invoice.buyer_as_lines(), col2_x);
    ctx.y = ctx.y.min(seller_end) - Mm(15.0);

    draw_table_header(&mut ctx, margin_left);
    for product in &invoice.products {
        draw_product_row(&mut ctx, product, invoice, locale, margin_left);
    }

    ctx.y -= Mm(10.0);
    ctx.use_text(
        &format!(
            "Subtotal: {}",
            format_currency(subtotal, &invoice.currency_code, locale)
        ),
        font_text,
        Mm(150.0),
    );
    for (rate, amt) in &tax_totals {
        ctx.use_text(
            &format!(
                "Tax ({:.0}%): {}",
                rate.into_inner() * 100.0,
                format_currency(*amt, &invoice.currency_code, locale)
            ),
            font_text,
            Mm(150.0),
        );
    }
    ctx.use_text(
        &format!(
            "Total: {}",
            format_currency(total, &invoice.currency_code, locale)
        ),
        font_text,
        Mm(150.0),
    );

    let mut buffer = Vec::new();
    doc.save(&mut BufWriter::new(&mut buffer))?;
    Ok(buffer)
}

pub fn draw_logo(
    ctx: &mut PdfContext,
    path: &Path,
    _sw: f32,
    _sh: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let dyn_image = ImageReader::open(path)?.decode()?;
    let (w_px, h_px) = (dyn_image.width(), dyn_image.height());
    let rgba_img = dyn_image.to_rgba8();

    let mut rgb_data = Vec::with_capacity((w_px * h_px * 3) as usize);
    let mut alpha_data = Vec::with_capacity((w_px * h_px) as usize);

    for y in 0..h_px {
        for x in 0..w_px {
            let pixel = rgba_img.get_pixel(x, y);
            rgb_data.push(pixel[0]);
            rgb_data.push(pixel[1]);
            rgb_data.push(pixel[2]);
        }
    }

    for y in 0..h_px {
        for x in 0..w_px {
            let pixel = rgba_img.get_pixel(x, y);
            alpha_data.push(pixel[3] as i64);
        }
    }

    let smask = SMask {
        width: w_px as i64,
        height: h_px as i64,
        interpolate: false,
        bits_per_component: -1,
        matte: alpha_data,
    };

    let image = ImageXObject {
        width: Px(w_px as usize),
        height: Px(h_px as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: false,
        image_data: rgb_data,
        image_filter: None,
        clipping_bbox: Some(CurTransMat::Identity),
        smask: Some(smask),
    };

    let target_width_mm: f32 = 100.0;
    let aspect_ratio = h_px as f32 / w_px as f32;
    let target_height_mm = target_width_mm * aspect_ratio;

    let pt_per_mm = 72.0 / 25.4;
    let top_edge_y = Mm(287.0);
    let bottom_edge_y = top_edge_y - Mm(target_height_mm);

    printpdf::Image::from(image).add_to_layer(
        ctx.current_layer.clone(),
        ImageTransform {
            translate_x: Some(Mm(20.0)),
            translate_y: Some(bottom_edge_y),
            rotate: None,
            scale_x: Some((target_width_mm * pt_per_mm) / w_px as f32),
            scale_y: Some((target_height_mm * pt_per_mm) / h_px as f32),
            dpi: Some(72.0),
        },
    );

    ctx.y = bottom_edge_y - Mm(10.0);
    Ok(())
}

fn draw_address_block(ctx: &mut PdfContext, lines: &[&str], x: Mm) {
    for line in lines {
        ctx.use_text_at(line, 10.0, x, ctx.y);
        ctx.y -= Mm(5.0);
    }
}

fn draw_table_header(ctx: &mut PdfContext, x: Mm) {
    ctx.use_text_at("Product", 10.0, x, ctx.y);
    ctx.use_text_at("Total", 10.0, Mm(160.0), ctx.y);
    ctx.y -= Mm(5.0);
    draw_line(&ctx.current_layer, x, Mm(190.0), ctx.y);
    ctx.y -= Mm(7.0);
}

fn draw_product_row(ctx: &mut PdfContext, p: &Product, inv: &Invoice, loc: Locale, x: Mm) {
    ctx.check_page_break(Mm(20.0));
    let start_y = ctx.y;
    let y_after = wrap_text(
        &p.description,
        80.0,
        10.0,
        &ctx.font,
        &ctx.current_layer,
        x,
        ctx.y,
    );
    let total = (p.units as f64 * p.cost_per_unit) * (1.0 + p.tax_rate);
    ctx.use_text_at(
        &format_currency(total, &inv.currency_code, loc),
        10.0,
        Mm(160.0),
        start_y,
    );
    ctx.y = y_after.min(ctx.y - Mm(2.0));
}

pub fn draw_line(layer: &PdfLayerReference, x1: Mm, x2: Mm, y: Mm) {
    layer.add_line(Line {
        points: vec![(Point::new(x1, y), false), (Point::new(x2, y), false)],
        is_closed: false,
    });
}

pub fn wrap_text(
    t: &str,
    max: f32,
    sz: f32,
    f: &IndirectFontRef,
    l: &PdfLayerReference,
    x: Mm,
    mut y: Mm,
) -> Mm {
    let words: Vec<&str> = t.split_whitespace().collect();
    let mut line = String::new();
    for word in words {
        if (line.len() + word.len()) as f32 * (sz * 0.35) > max {
            l.use_text(&line, sz, x, y, f);
            y -= Mm(sz * 0.4);
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    l.use_text(&line, sz, x, y, f);
    y - Mm(sz * 0.4)
}
