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
    ctx.write_text_at("Product", 9.0, COL_1, ctx.y);
    ctx.write_text_at("Units", 9.0, Mm(100.0), ctx.y);
    ctx.write_text_at("Unit Cost", 9.0, Mm(130.0), ctx.y);
    ctx.write_text_at("Total", 9.0, Mm(165.0), ctx.y);

    ctx.y -= Mm(5.0);
    draw_products(ctx, invoice, locale, COL_1)?;
    Ok(())
}
