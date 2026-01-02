use crate::PdfContext;
use crate::format_currency;
use crate::invoice::*;
use num_format::Locale;
use printpdf::*;

pub fn draw_products(
    ctx: &mut PdfContext,
    invoice: &Invoice,
    locale: &Locale,
    col1: Mm,
) -> Result<(), Box<dyn std::error::Error>> {
    for p in &invoice.products {
        if ctx.y < Mm(40.0) {
            ctx.pages.push(PdfPage::new(
                Mm(210.0),
                Mm(297.0),
                ctx.current_ops.drain(..).collect(),
            ));
            ctx.y = Mm(280.0);
        }
        let line_total = p.units as f64 * p.cost_per_unit;
        let row_y = ctx.y;
        ctx.write_text_at(&p.units.to_string(), 9.0, Mm(100.0), row_y);
        ctx.write_text_at(
            &format_currency(p.cost_per_unit, &invoice.currency_code, locale),
            9.0,
            Mm(130.0),
            row_y,
        );
        ctx.write_text_at(
            &format_currency(line_total, &invoice.currency_code, locale),
            9.0,
            Mm(165.0),
            row_y,
        );
        ctx.y = ctx.wrap_text_ops(&p.description, 75.0, 9.0, col1);
        ctx.y -= Mm(4.0);
    }
    Ok(())
}
