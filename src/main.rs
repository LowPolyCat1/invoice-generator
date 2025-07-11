use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use num_format::{Locale, ToFormattedString};


struct Seller {
    name: String,
    address: String,
    vat_id: String,
    website: String,
}

struct Buyer {
    name: String,
    address: String,
    email: String,
}

struct Product {
    description: String,
    units: u32,
    cost_per_unit: f64,
}

struct Invoice {
    number: String,
    date: String,
    seller: Seller,
    buyer: Buyer,
    payment_due: String,
    delivery_date: String,
    delivery_type: Option<String>,
    extra_info: Vec<(String, String)>,
    payment_type: Option<String>,
    payment_info: Vec<(String, String)>,
    tax_rate: f64,
    products: Vec<Product>,
}


fn main() {
    let invoice = Invoice {
        number: "INV-2025-EXAMPLE".to_string(),
        date: "2025-07-15".to_string(),
        seller: Seller {
            name: "Example Corp".to_string(),
            address: "123 Main Street\n90210 Anytown".to_string(),
            vat_id: "VAT-EX-00000000".to_string(),
            website: "examplecorp.com".to_string(),
        },
        buyer: Buyer {
            name: "John Doe".to_string(),
            address: "456 Oak Avenue\n10001 Cityville".to_string(),
            email: "john.doe@example.com".to_string(),
        },
        payment_due: "2025-08-15".to_string(),
        delivery_date: "2025-07-14".to_string(),
        delivery_type: Some("Standard Shipping".to_string()),
        extra_info: vec![
            ("Order Reference".to_string(), "987654321".to_string()),
            ("Project".to_string(), "Example Project".to_string()),
        ],
        payment_type: Some("Bank Transfer".to_string()),
        payment_info: vec![
            ("Account Name".to_string(), "J. Doe".to_string()),
            ("Bank Reference".to_string(), "REF-ABCD-1234".to_string()),
        ],
        tax_rate: 0.19,
        products: vec![
            Product {
                description: "Rusty Widget with very long description that might not fit in a single line".to_string(),
                units: 10,
                cost_per_unit: 9.99,
            },
            Product {
                description: "Gadget Pro".to_string(),
                units: 5,
                cost_per_unit: 19.95,
            },
        ],
    };

    generate_invoice_pdf(&invoice, "invoice.pdf").expect("Failed to create PDF");
}


fn generate_invoice_pdf(invoice: &Invoice, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (doc, page1, layer1) = PdfDocument::new("Invoice", Mm(210.0), Mm(297.0), "Layer 1");

    let current_layer = doc.get_page(page1).get_layer(layer1);
    let font = doc.add_external_font(std::fs::File::open("fonts/DejaVuSans.ttf")?)?;

    let margin_left = Mm(20.0);
    let margin_right = Mm(190.0);
    let mut current_y = Mm(270.0);

    let font_size_title = 22.0;
    let font_size_subtitle = 14.0;
    let font_size_text = 10.0;

    let use_bold_text = |text: &str, x: Mm, y: Mm| {
        current_layer.use_text(text, font_size_subtitle as f32, x, y, &font);
    };

    let use_text = |text: &str, x: Mm, y: Mm| {
        current_layer.use_text(text, font_size_text as f32, x, y, &font);
    };

    current_layer.use_text(&invoice.seller.name, font_size_title as f32, margin_left, current_y, &font);
    current_y -= Mm(18.0);

    current_y -= Mm(8.0);

    let info_x = Mm(130.0);
    use_text(&format!("Payment Due: {}", invoice.payment_due), info_x, current_y);
    current_y -= Mm(6.0);
    use_text(&format!("Delivery Date: {}", invoice.delivery_date), info_x, current_y);
    current_y -= Mm(6.0);

    if let Some(delivery_type) = &invoice.delivery_type {
        use_text(&format!("Delivery Type: {}", delivery_type), info_x, current_y);
        current_y -= Mm(6.0);
    }

    current_y -= Mm(6.0);
    draw_horizontal_line(&current_layer, margin_left, margin_right, current_y);
    current_y -= Mm(10.0);

    let col1_x = margin_left;
    let col2_x = Mm(margin_left.0 + 80.0);
    let row1_y = current_y;
    let row_height = Mm(6.5);

    use_text(&format!("Date: {}", invoice.date), col1_x, row1_y);
    use_text(&format!("Invoice ID: {}", invoice.number), col2_x, row1_y);

    let row2_y = row1_y - row_height;
    use_text(
        &format!(
            "{}: {}",
            invoice.extra_info.get(0).map(|(k, _)| k).unwrap_or(&"".to_string()),
            invoice.extra_info.get(0).map(|(_, v)| v).unwrap_or(&"".to_string())
        ),
        col1_x,
        row2_y,
    );
    use_text(
        &format!(
            "{}: {}",
            invoice.extra_info.get(1).map(|(k, _)| k).unwrap_or(&"".to_string()),
            invoice.extra_info.get(1).map(|(_, v)| v).unwrap_or(&"".to_string())
        ),
        col2_x,
        row2_y,
    );

    let separator_y = row2_y - Mm(5.0);
    draw_horizontal_line(&current_layer, margin_left, margin_right, separator_y);

    current_y = separator_y - Mm(15.0);

    let col_left = margin_left;
    let col_right = Mm(110.0);
    let label_offset = Mm(12.0);

    use_bold_text("Sold by", col_left, current_y);
    use_bold_text("Billed to", col_right, current_y);
    current_y -= label_offset;

    for line in invoice.seller.name.lines()
        .chain(invoice.seller.address.lines())
        .chain(std::iter::once(invoice.seller.vat_id.as_str()))
        .chain(std::iter::once(invoice.seller.website.as_str())) {
        use_text(line, col_left, current_y);
        current_y -= Mm(6.0);
    }

    let mut buyer_y = current_y + Mm(6.0) * 4.0 + Mm(6.0);
    for line in invoice.buyer.name.lines()
        .chain(invoice.buyer.address.lines())
        .chain(std::iter::once(invoice.buyer.email.as_str())) {
        use_text(line, col_right, buyer_y);
        buyer_y -= Mm(6.0);
    }

    current_y = if buyer_y < current_y { buyer_y } else { current_y };
    current_y -= Mm(15.0);

    let col_product = margin_left;
    let col_units = Mm(90.0);
    let col_unit_cost = Mm(120.0);
    let col_total = Mm(160.0);

    use_text("Product", col_product, current_y);
    use_text("Units", col_units, current_y);
    use_text("Unit Cost", col_unit_cost, current_y);
    use_text("Total", col_total, current_y);
    current_y -= Mm(8.0);

    draw_horizontal_line(&current_layer, margin_left, Mm(col_total.0 + 30.0), current_y);
    current_y -= Mm(6.0);

    let mut subtotal = 0.0;
    for product in &invoice.products {
        let total = product.units as f64 * product.cost_per_unit;
        subtotal += total;

        current_y = wrap_text(&product.description, 100.0, font_size_text as f32, &font, &current_layer, col_product, current_y);
        use_text(&product.units.to_string(), col_units, current_y);
        use_text(&format!("{} €", format_currency(product.cost_per_unit)), col_unit_cost, current_y);
        use_text(&format!("{} €", format_currency(total)), col_total, current_y);

        current_y -= Mm(6.0);
    }

    current_y -= Mm(10.0);

    let payment_info_x = margin_left;
    let mut payment_info_y = current_y;

    if let Some(payment_type) = &invoice.payment_type {
        use_text(&format!("Payment Type: {}", payment_type), payment_info_x, payment_info_y);
        payment_info_y -= Mm(8.0);

        for (k, v) in &invoice.payment_info {
            use_text(&format!("{}: {}", k, v), payment_info_x, payment_info_y);
            payment_info_y -= Mm(6.0);
        }
    }

    let totals_x_label = col_unit_cost;
    let totals_x_value = col_total;
    let mut totals_y = current_y;

    use_text("Subtotal:", totals_x_label, totals_y);
    use_text(&format!("{} €", format_currency(subtotal)), totals_x_value, totals_y);
    totals_y -= Mm(8.0);

    let tax_amount = subtotal * invoice.tax_rate;
    use_text(&format!("Tax ({}%):", (invoice.tax_rate * 100.0) as u8), totals_x_label, totals_y);
    use_text(&format!("{} €", format_currency(tax_amount)), totals_x_value, totals_y);
    totals_y -= Mm(8.0);

    let total_amount = subtotal + tax_amount;
    use_text("Total:", totals_x_label, totals_y);
    use_text(&format!("{} €", format_currency(total_amount)), totals_x_value, totals_y);

    doc.save(&mut BufWriter::new(File::create(filename)?))?;
    println!("Invoice saved to '{}'", filename);
    Ok(())
}

fn draw_horizontal_line(layer: &PdfLayerReference, start_x: Mm, end_x: Mm, y: Mm) {
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

fn wrap_text(text: &str, max_width_mm: f32, font_size: f32, font: &IndirectFontRef, layer: &PdfLayerReference, x: Mm, mut y: Mm) -> Mm {
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

fn format_currency(value: f64) -> String {
    let cents = (value * 100.0).round() as u64;
    let euros = cents / 100;
    let cent_part = cents % 100;
    format!("{}, {:02}", euros.to_formatted_string(&Locale::de), cent_part)
}