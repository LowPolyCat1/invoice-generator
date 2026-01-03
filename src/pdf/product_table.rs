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
    let weights = vec![5, 1, 2, 2, 2];
    let cols = col_pos(weights, Mm(210.0), COL_1, Mm(20.0));
    let right_edge = *cols.last().unwrap();

    ctx.y -= Mm(8.0);
    let table_top = ctx.y + Mm(4.0);
    let header_y = table_top - Mm(5.0);

    let w0 = (cols[1].0 - cols[0].0) - 2.0;
    let w1 = (cols[2].0 - cols[1].0) - 2.0;
    let w2 = (cols[3].0 - cols[2].0) - 2.0;
    let w3 = (cols[4].0 - cols[3].0) - 2.0;
    let w4 = (right_edge.0 - cols[4].0) - 2.0;

    let y0 = ctx.write_text_at_wrapping("Product", 9.0, cols[0] + Mm(2.0), header_y, w0);
    let y1 = ctx.write_text_at_wrapping("Units", 9.0, cols[1] + Mm(2.0), header_y, w1);
    let y2 = ctx.write_text_at_wrapping("Unit Cost", 9.0, cols[2] + Mm(2.0), header_y, w2);
    let y3 = ctx.write_text_at_wrapping("Tax", 9.0, cols[3] + Mm(2.0), header_y, w3);
    let y4 = ctx.write_text_at_wrapping("Total", 9.0, cols[4] + Mm(2.0), header_y, w4);

    ctx.y = y0.min(y1).min(y2).min(y3).min(y4) - Mm(3.0);

    for &x_pos in &cols {
        draw_v_line(&mut ctx.current_ops, x_pos, table_top, ctx.y);
    }
    draw_line(&mut ctx.current_ops, COL_1, right_edge, table_top);

    draw_products(ctx, invoice, locale, &cols)?;

    Ok(())
}
