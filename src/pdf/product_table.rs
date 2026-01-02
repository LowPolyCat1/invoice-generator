use crate::invoice::*;
use crate::pdf::products::draw_products;
use crate::pdf::*;
use num_format::Locale;

pub fn draw_product_table(
    ctx: &mut PdfContext,
    invoice: &Invoice,
    locale: &Locale,
) -> Result<(), Box<dyn std::error::Error>> {
    ctx.y -= Mm(8.0);
    let table_top = ctx.y + Mm(4.0);

    draw_line(&mut ctx.current_ops, COL_1, Mm(190.0), table_top);

    let header_y = table_top - Mm(5.0);

    ctx.write_text_at("Product", 9.0, COL_1 + Mm(2.0), header_y);
    ctx.write_text_at("Units", 9.0, Mm(102.0), header_y);
    ctx.write_text_at("Unit Cost", 9.0, Mm(132.0), header_y);
    ctx.write_text_at("Total", 9.0, Mm(167.0), header_y);

    let sub_header_y = header_y - Mm(3.0);

    ctx.y = sub_header_y - Mm(5.0);

    draw_v_line(&mut ctx.current_ops, COL_1, table_top, ctx.y);
    draw_v_line(&mut ctx.current_ops, Mm(100.0), table_top, ctx.y);
    draw_v_line(&mut ctx.current_ops, Mm(130.0), table_top, ctx.y);
    draw_v_line(&mut ctx.current_ops, Mm(160.0), table_top, ctx.y);
    draw_v_line(&mut ctx.current_ops, Mm(190.0), table_top, ctx.y);

    draw_products(ctx, invoice, locale, COL_1)?;

    Ok(())
}
