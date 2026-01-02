use crate::PdfContext;
use crate::draw_line;
use crate::draw_v_line;
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
    let table_top_initial = ctx.y;
    let mut current_page_top = table_top_initial + Mm(4.);

    for p in &invoice.products {
        if ctx.y < Mm(30.0) {
            draw_table_borders(ctx, col1, current_page_top, ctx.y);

            ctx.pages.push(PdfPage::new(
                Mm(210.0),
                Mm(297.0),
                ctx.current_ops.drain(..).collect(),
            ));

            ctx.y = Mm(280.0);
            current_page_top = ctx.y + Mm(5.0);
        }

        let line_total = p.units as f64 * p.cost_per_unit;
        let text_y = ctx.y;

        ctx.write_text_at(&p.units.to_string(), 9.0, Mm(102.0), text_y);
        ctx.write_text_at(
            &format_currency(p.cost_per_unit, &invoice.currency_code, locale),
            9.0,
            Mm(132.0),
            text_y,
        );
        ctx.write_text_at(
            &format_currency(line_total, &invoice.currency_code, locale),
            9.0,
            Mm(167.0),
            text_y,
        );

        ctx.y = ctx.wrap_text_ops(&p.description, 75.0, 9.0, col1 + Mm(2.0));
        ctx.y -= Mm(4.0);
    }

    draw_table_borders(ctx, col1, current_page_top, ctx.y);

    Ok(())
}

fn draw_table_borders(ctx: &mut PdfContext, col1: Mm, top: Mm, mut bottom: Mm) {
    let ops = &mut ctx.current_ops;
    let right_edge = Mm(190.0);

    bottom += Mm(5.);
    draw_line(ops, col1, right_edge, top);
    draw_v_line(ops, col1, top, bottom);
    draw_v_line(ops, Mm(100.0), top, bottom);
    draw_v_line(ops, Mm(130.0), top, bottom);
    draw_v_line(ops, Mm(160.0), top, bottom);
    draw_v_line(ops, right_edge, top, bottom);

    draw_line(ops, col1, right_edge, bottom);
}
