use printpdf::*;
use std::fs::File;

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
    pub tax_rate: f64,
    pub products: Vec<Product>,

    /// Added: currency & locale for formatting
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

    // Seller title
    context.use_text_at(&invoice.seller.name, font_size_title, margin_left, context.y);
    context.y -= Mm(20.0);

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
    let col_units = Mm(90.0);
    let col_unit_cost = Mm(120.0);
    let col_total = Mm(160.0);

    context.use_text_at("Product", font_size_text, col_product, context.y);
    context.use_text_at("Units", font_size_text, col_units, context.y);
    context.use_text_at("Unit Cost", font_size_text, col_unit_cost, context.y);
    context.use_text_at("Total", font_size_text, col_total, context.y);
    context.y -= Mm(8.0);
    draw_horizontal_line(&context.current_layer, margin_left, Mm(col_total.0 + 30.0), context.y);
    context.y -= Mm(6.0);

    let mut subtotal = 0.0;
    for product in &invoice.products {
        context.check_page_break(Mm(12.0));

        let y_before_wrap = context.y;
        context.y = wrap_text(
            &product.description,
            100.0,
            font_size_text,
            &context.font,
            &context.current_layer,
            col_product,
            context.y,
        );

        // numbers
        context.use_text_at(&product.units.to_string(), font_size_text, col_units, y_before_wrap);

        let unit_str = format_currency(product.cost_per_unit, &invoice.currency_code, locale);
        let total_val = product.units as f64 * product.cost_per_unit;
        let total_str = format_currency(total_val, &invoice.currency_code, locale);

        context.use_text_at(&unit_str, font_size_text, col_unit_cost, y_before_wrap);
        context.use_text_at(&total_str, font_size_text, col_total, y_before_wrap);

        subtotal += total_val;
        context.y -= Mm(4.0);
    }

    // Payment info
    context.y -= Mm(10.0);
    if let Some(payment_type) = &invoice.payment_type {
        context.check_page_break(Mm(8.0));
        context.use_text(&format!("Payment Type: {}", payment_type), font_size_text, margin_left);
        for (k, v) in &invoice.payment_info {
            context.check_page_break(Mm(6.0));
            context.use_text(&format!("{}: {}", k, v), font_size_text, margin_left);
        }
    }

    // Totals
    context.y -= Mm(10.0);
    let tax_amount = subtotal * invoice.tax_rate;
    let total = subtotal + tax_amount;

    let subtotal_str = format_currency(subtotal, &invoice.currency_code, locale);
    let tax_str = format_currency(tax_amount, &invoice.currency_code, locale);
    let total_str = format_currency(total, &invoice.currency_code, locale);

    context.use_text(&format!("Subtotal: {}", subtotal_str), font_size_text, col_total);
    context.use_text(
        &format!("Tax ({}%): {}", (invoice.tax_rate * 100.0) as u8, tax_str),
        font_size_text,
        col_total,
    );
    context.use_text(&format!("Total: {}", total_str), font_size_text, col_total);

    // Return as Vec<u8>
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
        y -= Mm(font_size * 0.4);
    }

    y
}
