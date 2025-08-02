use ::image::ImageReader;
use ordered_float::OrderedFloat;
use printpdf::*;
use std::fs::File;
use std::collections::BTreeMap;

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
        self.current_layer.use_text(text, size, x, self.y, &self.font);
        self.y -= Mm(size * 0.4);
    }

    pub fn use_text_at(&self, text: &str, size: f32, x: Mm, y: Mm) {
        self.current_layer.use_text(text, size, x, y, &self.font);
    }
}

use printpdf::{ImageXObject, ImageTransform};

pub fn draw_logo(
    context: &mut PdfContext,
    image_path: &str,
    width_mm: f32,
    height_mm: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let dyn_image = ImageReader::open(image_path)?.decode()?.to_rgb8();
    let (img_width, img_height) = dyn_image.dimensions();


    let image = ImageXObject {
        width: Px(img_width as usize),
        height: Px(img_height as usize),
        color_space: printpdf::ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: dyn_image.into_raw(),
        image_filter: None,
        clipping_bbox: Some(CurTransMat::Identity),
        smask: None
    };

    let pdf_image = printpdf::Image::from(image);

    context.y -= Mm(30.0);
    pdf_image.add_to_layer(
        context.current_layer.clone(),
        ImageTransform {
            translate_x: Some(Mm(20.0)),
            translate_y: Some(context.y - Mm(height_mm)),
            rotate: None,
            scale_x: Some(width_mm),
            scale_y: Some(height_mm),
            dpi: Some(300.0),
        },
    );
    context.y += Mm(30.0);

    // Move cursor down
    context.y -= Mm(height_mm + 10.0);

    Ok(())
}


pub fn generate_invoice_pdf(invoice: &Invoice) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (doc, page1, layer1) = PdfDocument::new("Invoice", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_external_font(File::open("fonts/OpenSans-Medium.ttf")?)?;

    let mut context = PdfContext {
        doc: &doc,
        font: font.clone(),
        current_layer: doc.get_page(page1).get_layer(layer1),
        y: Mm(270.0),
    };

    let locale = get_locale_by_code(&invoice.locale_code);

    let margin_left = Mm(20.0);
    let col2_x = Mm(110.0);
    let font_size_title = 22.0;
    let font_size_subtitle = 14.0;
    let font_size_text = 10.0;

    // Draw logo instead of seller name
    draw_logo(&mut context, "D:\\VSC\\Rust\\Projects\\current\\invoice\\res\\logo.jpg", 1.00, 1.0)?;

    let mut info_y = context.y;
    context.use_text_at(&format!("Payment Due: {}", invoice.payment_due), font_size_text, col2_x, info_y);
    info_y -= Mm(6.0);
    context.use_text_at(&format!("Delivery Date: {}", invoice.delivery_date), font_size_text, col2_x, info_y);
    info_y -= Mm(6.0);
    if let Some(delivery_type) = &invoice.delivery_type {
        context.use_text_at(&format!("Delivery Type: {}", delivery_type), font_size_text, col2_x, info_y);
        info_y -= Mm(6.0);
    }

    context.y = info_y - Mm(5.0);
    draw_horizontal_line(&context.current_layer, margin_left, Mm(190.0), context.y);
    context.y -= Mm(10.0);

    let row_y = context.y;
    context.use_text_at(&format!("Date: {}", invoice.date), font_size_text, margin_left, row_y);
    context.use_text_at(&format!("Invoice ID: {}", invoice.number), font_size_text, col2_x, row_y);
    context.y -= Mm(6.0);

    let row2_y = context.y;
    if let Some((k, v)) = invoice.extra_info.get(0) {
        context.use_text_at(&format!("{}: {}", k, v), font_size_text, margin_left, row2_y);
    }
    if let Some((k, v)) = invoice.extra_info.get(1) {
        context.use_text_at(&format!("{}: {}", k, v), font_size_text, col2_x, row2_y);
    }
    context.y -= Mm(10.0);

    draw_horizontal_line(&context.current_layer, margin_left, Mm(190.0), context.y);
    context.y -= Mm(10.0);

    // Seller & Buyer
    let header_y = context.y;
    context.use_text_at("Sold by", font_size_subtitle, margin_left, header_y);
    context.use_text_at("Billed to", font_size_subtitle, col2_x, header_y);
    context.y -= Mm(12.0);

    let mut left_y = context.y;
    let mut right_y = header_y - Mm(12.0);

    for line in invoice.seller.name.lines()
        .chain(invoice.seller.address.lines())
        .chain(std::iter::once(invoice.seller.vat_id.as_str()))
        .chain(std::iter::once(invoice.seller.website.as_str())) {
        context.use_text_at(line, font_size_text, margin_left, left_y);
        left_y -= Mm(6.0);
    }

    for line in invoice.buyer.name.lines()
        .chain(invoice.buyer.address.lines())
        .chain(std::iter::once(invoice.buyer.email.as_str())) {
        context.use_text_at(line, font_size_text, col2_x, right_y);
        right_y -= Mm(6.0);
    }

    context.y = left_y.min(right_y) - Mm(15.0);
    context.check_page_break(Mm(20.0));

    // Product table
    let col_product = margin_left;
    let col_units = Mm(70.0);
    let col_unit_cost = Mm(100.0);
    let col_tax = Mm(130.0);
    let col_total = Mm(160.0);

    context.use_text_at("Product", font_size_text, col_product, context.y);
    context.use_text_at("Units", font_size_text, col_units, context.y);
    context.use_text_at("Unit Cost", font_size_text, col_unit_cost, context.y);
    context.use_text_at("Tax", font_size_text, col_tax, context.y);
    context.use_text_at("Total", font_size_text, col_total, context.y);
    context.y -= Mm(8.0);
    draw_horizontal_line(&context.current_layer, margin_left, Mm(col_total.0 + 30.0), context.y);
    context.y -= Mm(6.0);

    let mut subtotal = 0.0;
    let mut tax_totals: BTreeMap<OrderedFloat<f64>, f64> = BTreeMap::new();

    fn count_lines_used(y_start: Mm, y_end: Mm, font_size: f32) -> usize {
        (((y_start.0 - y_end.0) / (font_size as f32 * 0.4)).round()) as usize
    }

    for product in &invoice.products {
        context.check_page_break(Mm(12.0));

        let y_before_wrap = context.y;

        let y_after_desc = wrap_text(
            &product.description,
            80.0,
            font_size_text,
            &context.font,
            &context.current_layer,
            col_product,
            context.y,
        );

        let tax_label = if product.tax_rate == 0.0 {
            product.tax_exempt_reason.clone().unwrap_or_else(|| "0%".to_string())
        } else {
            format!("{:.0}%", product.tax_rate * 100.0)
        };

        let max_tax_width = (col_total.0 - col_tax.0) as f32;
        let y_after_tax = wrap_text(
            &tax_label,
            max_tax_width,
            font_size_text,
            &context.font,
            &context.current_layer,
            col_tax,
            context.y,
        );

        let desc_lines = count_lines_used(context.y, y_after_desc, font_size_text);
        let tax_lines = count_lines_used(context.y, y_after_tax, font_size_text);
        let lines_used = desc_lines.max(tax_lines);

        context.use_text_at(&product.units.to_string(), font_size_text, col_units, y_before_wrap);

        let unit_str = format_currency(product.cost_per_unit, &invoice.currency_code, locale);
        context.use_text_at(&unit_str, font_size_text, col_unit_cost, y_before_wrap);

        let line_total = product.units as f64 * product.cost_per_unit;
        let line_tax = line_total * product.tax_rate;
        subtotal += line_total;
        if product.tax_rate > 0.0 {
            *tax_totals.entry(OrderedFloat(product.tax_rate)).or_insert(0.0) += line_tax;
        }

        let total_str = format_currency(line_total + line_tax, &invoice.currency_code, locale);
        context.use_text_at(&total_str, font_size_text, col_total, y_before_wrap);

        context.y -= Mm(lines_used as f32 * (font_size_text as f32 * 0.4));
    }

    context.y -= Mm(10.0);
    if let Some(payment_type) = &invoice.payment_type {
        context.check_page_break(Mm(8.0));
        context.use_text(&format!("Payment Type: {}", payment_type), font_size_text, margin_left);
        for (k, v) in &invoice.payment_info {
            context.check_page_break(Mm(6.0));
            context.use_text(&format!("{}: {}", k, v), font_size_text, margin_left);
        }
    }

    context.y -= Mm(10.0);
    let total: f64 = subtotal + tax_totals.values().sum::<f64>();

    let subtotal_str = format_currency(subtotal, &invoice.currency_code, locale);
    context.use_text(&format!("Subtotal: {}", subtotal_str), font_size_text, col_total);

    for (rate, amount) in &tax_totals {
        let rate_val = rate.into_inner();
        let tax_str = format_currency(*amount, &invoice.currency_code, locale);
        context.use_text(&format!("Tax ({:.0}%): {}", rate_val * 100.0, tax_str), font_size_text, col_total);
    }

    let total_str = format_currency(total, &invoice.currency_code, locale);
    context.use_text(&format!("Total: {}", total_str), font_size_text, col_total);

    let mut buffer = Vec::new();
    {
        let mut writer = std::io::BufWriter::new(&mut buffer);
        doc.save(&mut writer)?;
    }
    Ok(buffer)
}

pub fn draw_horizontal_line(layer: &PdfLayerReference, start_x: Mm, end_x: Mm, y: Mm) {
    use printpdf::{Point, Line};
    let line = Line {
        points: vec![
            (Point::new(start_x, y), false),
            (Point::new(end_x, y), false),
        ],
        is_closed: false,
    };
    layer.add_line(line);
}

pub fn wrap_text(
    text: &str,
    max_width_mm: f32,
    font_size: f32,
    font: &IndirectFontRef,
    layer: &PdfLayerReference,
    x: Mm,
    mut y: Mm,
) -> Mm {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut line = String::new();
    let mut line_width = 0.0;
    let char_width = font_size * 0.35;

    for word in words {
        let word_width = (word.len() as f32) * char_width;

        if line_width + word_width > max_width_mm {
            layer.use_text(&line, font_size, x, y, font);
            y -= Mm(font_size * 0.4);
            line.clear();
            line_width = 0.0;
        }

        if !line.is_empty() {
            line.push(' ');
            line_width += char_width;
        }

        line.push_str(word);
        line_width += word_width;
    }

    if !line.is_empty() {
        layer.use_text(&line, font_size, x, y, font);
        y -= Mm(font_size * 0.4) + Mm(2.0);
    }

    y
}
