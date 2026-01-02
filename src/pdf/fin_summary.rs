use crate::format::format_currency;
use crate::invoice::*;
use crate::pdf::*;
use num_format::Locale;

pub fn draw_financial_summary(
    ctx: &mut PdfContext,
    invoice: &Invoice,
    locale: &Locale,
    subtotal: f64,
    tax_map: &std::collections::BTreeMap<ordered_float::OrderedFloat<f64>, f64>,
    total: f64,
) {
    ctx.y -= Mm(2.0);
    let start_y = ctx.y;

    draw_line(&mut ctx.current_ops, COL_2, Mm(190.0), start_y);

    ctx.y -= Mm(6.0);
    ctx.write_text_at("Subtotal:", 9.0, COL_2 + Mm(2.0), ctx.y);
    ctx.write_text_at(
        &format_currency(subtotal, &invoice.currency_code, locale),
        9.0,
        Mm(165.0),
        ctx.y,
    );

    for (rate, amt) in tax_map {
        ctx.y -= Mm(5.0);
        ctx.write_text_at(
            &format!("Tax ({:.0}%):", rate.0 * 100.0),
            9.0,
            COL_2 + Mm(2.0),
            ctx.y,
        );
        ctx.write_text_at(
            &format_currency(*amt, &invoice.currency_code, locale),
            9.0,
            Mm(165.0),
            ctx.y,
        );
    }

    ctx.y -= Mm(4.0);
    draw_line(&mut ctx.current_ops, COL_2, Mm(190.0), ctx.y);

    ctx.y -= Mm(6.0);
    ctx.write_text_at("TOTAL:", 12.0, COL_2 + Mm(2.0), ctx.y);
    ctx.write_text_at(
        &format_currency(total, &invoice.currency_code, locale),
        12.0,
        Mm(165.0),
        ctx.y,
    );

    let end_y = ctx.y - Mm(3.0);

    draw_v_line(&mut ctx.current_ops, COL_2, start_y, end_y);
    draw_v_line(&mut ctx.current_ops, Mm(160.0), start_y, end_y);
    draw_v_line(&mut ctx.current_ops, Mm(190.0), start_y, end_y);

    draw_line(&mut ctx.current_ops, COL_2, Mm(190.0), end_y);

    ctx.y = end_y;
}
