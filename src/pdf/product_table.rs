use crate::invoice::*;
use crate::pdf::cols::col_pos;
use crate::pdf::products::draw_products;
use crate::pdf::*;
use num_format::Locale;

pub fn draw_product_table(
    ctx: &mut PdfContext,
    invoice: &Invoice,
    locale: &Locale,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define Column Layout
    // Description: 40%, Units: 10%, Cost: 15%, Tax: 15%, Total: 20%
    let weights = vec![5, 1, 2, 2, 2];
    let cols = col_pos(weights, Mm(210.0), COL_1, Mm(20.0));
    let right_edge = cols[5];

    ctx.y -= Mm(8.0);
    let table_top = ctx.y + Mm(4.0);

    // Header logic
    let header_y = table_top - Mm(5.0);
    ctx.write_text_at("Product", 9.0, cols[0] + Mm(2.0), header_y);
    ctx.write_text_at("Units", 9.0, cols[1] + Mm(2.0), header_y);
    ctx.write_text_at("Unit Cost", 9.0, cols[2] + Mm(2.0), header_y);
    ctx.write_text_at("Tax", 9.0, cols[3] + Mm(2.0), header_y);
    ctx.write_text_at("Total", 9.0, cols[4] + Mm(2.0), header_y);

    ctx.y = header_y - Mm(8.0);

    // Draw header lines using the dynamic positions
    for &x_pos in &cols {
        draw_v_line(&mut ctx.current_ops, x_pos, table_top, ctx.y);
    }
    draw_line(&mut ctx.current_ops, COL_1, right_edge, table_top);

    // Pass the calculated columns to the product drawer
    draw_products(ctx, invoice, locale, &cols)?;

    Ok(())
}
