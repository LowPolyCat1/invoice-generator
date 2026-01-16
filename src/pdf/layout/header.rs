use crate::invoice::Invoice;
use crate::pdf::context::PdfContext;
use crate::pdf::drawing::draw_line;
use crate::pdf::{COL_1, COL_2};
use printpdf::Mm;

pub fn draw_header_info(ctx: &mut PdfContext, invoice: &Invoice) {
    let col_width = (COL_2.0 - COL_1.0) - 5.0;

    ctx.y = ctx.write_text_at_wrapping(
        &format!("Invoice Id: {}", invoice.number),
        14.0,
        COL_1,
        ctx.y,
        col_width,
    );

    ctx.y -= Mm(4.0);

    let row_1_y = ctx.y;
    let left_y = ctx.write_text_at_wrapping(
        &invoice.locale.format_date(&invoice.date),
        10.0,
        COL_1,
        row_1_y,
        col_width,
    );
    let right_y = ctx.write_text_at_wrapping(
        &format!(
            "Payment Due: {}",
            &invoice.locale.format_date(&invoice.payment_due)
        ),
        10.0,
        COL_2,
        row_1_y,
        col_width,
    );
    ctx.y = left_y.min(right_y) - Mm(1.);

    let row_2_y = ctx.y;

    if let Some(extra_info_iter) = invoice.extra_info.iter().next() {}

    let mut extra_info_iter = invoice.extra_info.iter();

    // 1. Collect all "extra" fields into a single vector of (Label, Value)
let mut metadata = Vec::new();

if let Some(ref d_date) = invoice.delivery_date {
    metadata.push(("Delivery Date".to_string(), invoice.locale.format_date(d_date)));
}

if let Some(ref d_type) = invoice.delivery_type {
    metadata.push(("Delivery Type".to_string(), d_type.clone()));
}

if let Some(ref payment) = invoice.payment_type {
    metadata.push(("Payment Type".to_string(), payment.clone()));
}

// Add the extra_info pairs if they exist
if let Some(ref extra) = invoice.extra_info {
    metadata.extend(extra.iter().cloned());
}

// 2. Iterate through the collected metadata two-by-two
let mut metadata_iter = metadata.into_iter();

while let Some((label1, value1)) = metadata_iter.next() {
    let current_row_y = ctx.y;

    // Left Column
    let left_y = ctx.write_text_at_wrapping(
        &format!("{}: {}", label1, value1),
        10.0,
        COL_1,
        current_row_y,
        col_width,
    );

    // Right Column (if there's another item)
    let mut right_y = left_y;
    if let Some((label2, value2)) = metadata_iter.next() {
        right_y = ctx.write_text_at_wrapping(
            &format!("{}: {}", label2, value2),
            10.0,
            COL_2,
            current_row_y,
            col_width,
        );
    }

    // Update Y position to the lowest point of the current row
    ctx.y = left_y.min(right_y) - Mm(1.0);
}

    draw_line(&mut ctx.current_ops, COL_1, Mm(190.0), ctx.y);
}
