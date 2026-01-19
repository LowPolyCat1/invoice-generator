use crate::invoice::Invoice;
use krilla::Document;
use krilla::color::rgb;
use krilla::geom::Point;
use krilla::paint::Stroke;
use krilla::text::Font;
use krilla::text::TextDirection;
use std::path::Path;

pub fn generate_invoice_pdf<P: AsRef<Path>>(
    invoice: &Invoice,
    font_path: P,
    _logo_path: Option<P>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut doc = Document::new();

    let font_bytes = std::fs::read(font_path)?;
    let font = Font::new(krilla::Data::from(font_bytes), 0).ok_or("Failed to load font")?;

    let mut page = doc.start_page();
    let mut surface = page.surface();

    // Page dimensions (A4 in points: 595x842)
    let margin_left = 40.0;
    let margin_top = 40.0;
    let margin_right = 40.0;
    let page_width = 595.0;
    let _page_height = 842.0;

    let mut y = margin_top;

    // Title
    surface.draw_text(
        Point::from_xy(margin_left, y),
        font.clone(),
        28.0,
        "INVOICE",
        false,
        TextDirection::Auto,
    );

    y += 40.0;

    // Invoice details (right aligned)
    let details_x = page_width - margin_right - 150.0;
    surface.draw_text(
        Point::from_xy(details_x, y),
        font.clone(),
        10.0,
        &format!("Invoice #: {}", invoice.number),
        false,
        TextDirection::Auto,
    );

    y += 15.0;
    surface.draw_text(
        Point::from_xy(details_x, y),
        font.clone(),
        10.0,
        &format!("Date: {}", invoice.locale.format_date(&invoice.date)),
        false,
        TextDirection::Auto,
    );

    y += 15.0;
    surface.draw_text(
        Point::from_xy(details_x, y),
        font.clone(),
        10.0,
        &format!("Due: {}", invoice.locale.format_date(&invoice.payment_due)),
        false,
        TextDirection::Auto,
    );

    y += 40.0;

    // Seller and Buyer sections
    let col1_x = margin_left;
    let col2_x = margin_left + 280.0;

    // FROM section
    surface.draw_text(
        Point::from_xy(col1_x, y),
        font.clone(),
        11.0,
        "FROM:",
        false,
        TextDirection::Auto,
    );

    y += 18.0;
    surface.draw_text(
        Point::from_xy(col1_x, y),
        font.clone(),
        10.0,
        &invoice.seller.name,
        false,
        TextDirection::Auto,
    );

    y += 14.0;
    surface.draw_text(
        Point::from_xy(col1_x, y),
        font.clone(),
        9.0,
        &format!(
            "{} {}",
            invoice.seller.address.street, invoice.seller.address.house_number
        ),
        false,
        TextDirection::Auto,
    );

    y += 12.0;
    surface.draw_text(
        Point::from_xy(col1_x, y),
        font.clone(),
        9.0,
        &format!(
            "{} {}",
            invoice.seller.address.code, invoice.seller.address.town
        ),
        false,
        TextDirection::Auto,
    );

    y += 12.0;
    surface.draw_text(
        Point::from_xy(col1_x, y),
        font.clone(),
        9.0,
        &format!("VAT: {}", invoice.seller.vat_id),
        false,
        TextDirection::Auto,
    );

    // BILL TO section (at same height as FROM)
    let mut y_bill_to = margin_top + 40.0;

    surface.draw_text(
        Point::from_xy(col2_x, y_bill_to),
        font.clone(),
        11.0,
        "BILL TO:",
        false,
        TextDirection::Auto,
    );

    y_bill_to += 18.0;
    surface.draw_text(
        Point::from_xy(col2_x, y_bill_to),
        font.clone(),
        10.0,
        &invoice.buyer.name,
        false,
        TextDirection::Auto,
    );

    y_bill_to += 14.0;
    surface.draw_text(
        Point::from_xy(col2_x, y_bill_to),
        font.clone(),
        9.0,
        &format!(
            "{} {}",
            invoice.buyer.address.street, invoice.buyer.address.house_number
        ),
        false,
        TextDirection::Auto,
    );

    y_bill_to += 12.0;
    surface.draw_text(
        Point::from_xy(col2_x, y_bill_to),
        font.clone(),
        9.0,
        &format!(
            "{} {}",
            invoice.buyer.address.code, invoice.buyer.address.town
        ),
        false,
        TextDirection::Auto,
    );

    y_bill_to += 12.0;
    surface.draw_text(
        Point::from_xy(col2_x, y_bill_to),
        font.clone(),
        9.0,
        &invoice.buyer.email,
        false,
        TextDirection::Auto,
    );

    // Move to next section
    y = y.max(y_bill_to) + 30.0;

    // Horizontal line
    draw_line(&mut surface, margin_left, page_width - margin_right, y)?;

    y += 20.0;

    // Table headers
    let col_desc = margin_left;
    let col_qty = margin_left + 320.0;
    let col_price = margin_left + 380.0;
    let col_total = margin_left + 450.0;

    surface.draw_text(
        Point::from_xy(col_desc, y),
        font.clone(),
        10.0,
        "Description",
        false,
        TextDirection::Auto,
    );

    surface.draw_text(
        Point::from_xy(col_qty, y),
        font.clone(),
        10.0,
        "Qty",
        false,
        TextDirection::Auto,
    );

    surface.draw_text(
        Point::from_xy(col_price, y),
        font.clone(),
        10.0,
        "Unit Price",
        false,
        TextDirection::Auto,
    );

    surface.draw_text(
        Point::from_xy(col_total, y),
        font.clone(),
        10.0,
        "Total",
        false,
        TextDirection::Auto,
    );

    y += 12.0;
    draw_line(&mut surface, margin_left, page_width - margin_right, y)?;

    y += 15.0;

    // Products
    for product in &invoice.products {
        let line_total = product.units as f64 * product.cost_per_unit;

        // Truncate long descriptions
        let desc = if product.description.len() > 45 {
            format!("{}...", &product.description[..42])
        } else {
            product.description.clone()
        };

        surface.draw_text(
            Point::from_xy(col_desc, y),
            font.clone(),
            9.0,
            &desc,
            false,
            TextDirection::Auto,
        );

        surface.draw_text(
            Point::from_xy(col_qty, y),
            font.clone(),
            9.0,
            &product.units.to_string(),
            false,
            TextDirection::Auto,
        );

        surface.draw_text(
            Point::from_xy(col_price, y),
            font.clone(),
            9.0,
            &format!("${:.2}", product.cost_per_unit),
            false,
            TextDirection::Auto,
        );

        surface.draw_text(
            Point::from_xy(col_total, y),
            font.clone(),
            9.0,
            &format!("${:.2}", line_total),
            false,
            TextDirection::Auto,
        );

        y += 14.0;
    }

    y += 5.0;
    draw_line(&mut surface, margin_left, page_width - margin_right, y)?;

    // Totals section
    y += 20.0;
    let totals_label_x = col_price - 60.0;
    let totals_value_x = col_total;

    let (subtotal, tax_totals, total) = invoice.calculate_summary();

    surface.draw_text(
        Point::from_xy(totals_label_x, y),
        font.clone(),
        10.0,
        "Subtotal:",
        false,
        TextDirection::Auto,
    );

    surface.draw_text(
        Point::from_xy(totals_value_x, y),
        font.clone(),
        10.0,
        &format!("${:.2}", subtotal),
        false,
        TextDirection::Auto,
    );

    y += 14.0;

    // Tax breakdown
    for (rate, amount) in tax_totals {
        surface.draw_text(
            Point::from_xy(totals_label_x, y),
            font.clone(),
            10.0,
            &format!("Tax ({:.0}%):", rate.0 * 100.0),
            false,
            TextDirection::Auto,
        );

        surface.draw_text(
            Point::from_xy(totals_value_x, y),
            font.clone(),
            10.0,
            &format!("${:.2}", amount),
            false,
            TextDirection::Auto,
        );

        y += 14.0;
    }

    y += 5.0;
    draw_line(&mut surface, totals_label_x, page_width - margin_right, y)?;

    y += 15.0;

    // Total
    surface.draw_text(
        Point::from_xy(totals_label_x, y),
        font.clone(),
        12.0,
        "TOTAL:",
        false,
        TextDirection::Auto,
    );

    surface.draw_text(
        Point::from_xy(totals_value_x, y),
        font.clone(),
        12.0,
        &format!("${:.2}", total),
        false,
        TextDirection::Auto,
    );

    // Payment info
    y += 40.0;
    if let Some(payment_info) = &invoice.payment_info {
        surface.draw_text(
            Point::from_xy(margin_left, y),
            font.clone(),
            10.0,
            "Payment Information:",
            false,
            TextDirection::Auto,
        );

        y += 14.0;
        for (key, value) in payment_info {
            surface.draw_text(
                Point::from_xy(margin_left, y),
                font.clone(),
                9.0,
                &format!("{}: {}", key, value),
                false,
                TextDirection::Auto,
            );
            y += 12.0;
        }
    }

    surface.finish();
    page.finish();

    let pdf_bytes = doc.finish().map_err(|e| format!("{:?}", e))?;
    Ok(pdf_bytes)
}

fn draw_line(
    surface: &mut krilla::surface::Surface,
    x1: f32,
    x2: f32,
    y: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pb = krilla::geom::PathBuilder::new();
    pb.move_to(x1, y);
    pb.line_to(x2, y);
    let path = pb.finish().ok_or("Failed to create line")?;

    surface.set_stroke(Some(Stroke {
        paint: rgb::Color::new(0, 0, 0).into(),
        width: 1.0,
        ..Default::default()
    }));
    surface.draw_path(&path);

    Ok(())
}
