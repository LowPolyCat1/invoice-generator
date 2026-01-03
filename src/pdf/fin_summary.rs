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
    let right_edge = Mm(190.0);
    let label_x = COL_2 + Mm(2.0);
    let value_x = Mm(165.0);

    let label_width = (value_x.0 - label_x.0) - 2.0;
    let val_width = (right_edge.0 - value_x.0) - 2.0;

    draw_line(&mut ctx.current_ops, COL_2, right_edge, start_y);

    ctx.y -= Mm(6.0);
    let row_y = ctx.y;

    ctx.write_text_at_wrapping("Subtotal:", 9.0, label_x, row_y, label_width);

    ctx.y = ctx.write_text_at_wrapping(
        &format_currency(subtotal, &invoice.currency_code, locale),
        9.0,
        value_x,
        row_y,
        val_width,
    );

    for (rate, amt) in tax_map {
        ctx.y -= Mm(2.0);
        let tax_row_y = ctx.y;

        ctx.write_text_at_wrapping(
            &format!("Tax ({:.0}%):", rate.0 * 100.0),
            9.0,
            label_x,
            tax_row_y,
            label_width,
        );

        ctx.y = ctx.write_text_at_wrapping(
            &format_currency(*amt, &invoice.currency_code, locale),
            9.0,
            value_x,
            tax_row_y,
            val_width,
        );
    }

    ctx.y -= Mm(2.0);
    draw_line(&mut ctx.current_ops, COL_2, right_edge, ctx.y);

    ctx.y -= Mm(6.0);
    let total_row_y = ctx.y;

    ctx.write_text_at_wrapping("TOTAL:", 12.0, label_x, total_row_y, label_width);

    ctx.y = ctx.write_text_at_wrapping(
        &format_currency(total, &invoice.currency_code, locale),
        12.0,
        value_x,
        total_row_y,
        val_width,
    );

    let end_y = ctx.y - Mm(3.0);

    draw_v_line(&mut ctx.current_ops, COL_2, start_y, end_y);
    draw_v_line(&mut ctx.current_ops, Mm(160.0), start_y, end_y);
    draw_v_line(&mut ctx.current_ops, right_edge, start_y, end_y);

    draw_line(&mut ctx.current_ops, COL_2, right_edge, end_y);

    ctx.y = end_y;
}
