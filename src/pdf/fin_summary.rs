use locale_rs::Locale;
use locale_rs::num_formats::ToFormattedString;

use crate::invoice::*;
use crate::pdf::*;

pub fn draw_financial_summary(
    ctx: &mut PdfContext,
    invoice: &Invoice,
    locale: &Locale,
    subtotal: f64,
    tax_map: &std::collections::BTreeMap<ordered_float::OrderedFloat<f64>, f64>,
    total: f64,
) {
    let right_edge = Mm(190.0);
    let label_x = COL_2 + Mm(2.0);
    let value_x = Mm(165.0);
    let label_width = (value_x.0 - label_x.0) - 2.0;
    let val_width = (right_edge.0 - value_x.0) - 2.0;

    let mut total_box_height = Mm(0.0);
    total_box_height += Mm(2.0);

    let sub_text = subtotal.to_formatted_string(locale);
    total_box_height += ctx
        .measure_text_height(&sub_text, 9.0, val_width)
        .max(Mm(6.0));

    for (_rate, amt) in tax_map {
        total_box_height += Mm(2.0);
        let tax_text = amt.to_formatted_string(locale);
        total_box_height += ctx
            .measure_text_height(&tax_text, 9.0, val_width)
            .max(Mm(5.0));
    }

    total_box_height += Mm(2.0);
    let total_text = total.to_formatted_string(locale);
    total_box_height += ctx
        .measure_text_height(&total_text, 12.0, val_width)
        .max(Mm(8.0));
    total_box_height += Mm(3.0);

    if ctx.y - total_box_height < BOTTOM_MARGIN {
        ctx.pages.push(PdfPage::new(
            PAGE_WIDTH,
            PAGE_HEIGHT,
            ctx.current_ops.drain(..).collect(),
        ));
        ctx.y = Mm(280.0);
    }

    ctx.y -= Mm(2.0);
    let start_y = ctx.y;
    draw_line(&mut ctx.current_ops, COL_2, right_edge, start_y);

    ctx.y -= Mm(6.0);
    let sub_y = ctx.y;
    ctx.write_text_at_wrapping("Subtotal:", 9.0, label_x, sub_y, label_width);
    ctx.y = ctx.write_text_at_wrapping(&sub_text, 9.0, value_x, sub_y, val_width);

    for (rate, amt) in tax_map {
        ctx.y -= Mm(2.0);
        let tax_y = ctx.y;
        ctx.write_text_at_wrapping(
            &format!("Tax ({:.0}%):", rate.0 * 100.0),
            9.0,
            label_x,
            tax_y,
            label_width,
        );
        ctx.y = ctx.write_text_at_wrapping(
            &amt.to_formatted_string(locale),
            9.0,
            value_x,
            tax_y,
            val_width,
        );
    }

    ctx.y -= Mm(2.0);
    draw_line(&mut ctx.current_ops, COL_2, right_edge, ctx.y);

    ctx.y -= Mm(6.0);
    let tot_y = ctx.y;
    ctx.write_text_at_wrapping("Total:", 9.0, label_x, tot_y, label_width);
    ctx.y = ctx.write_text_at_wrapping(&total_text, 9.0, value_x, tot_y, val_width);

    let end_y = ctx.y;
    draw_v_line(&mut ctx.current_ops, COL_2, start_y, end_y);
    draw_v_line(&mut ctx.current_ops, Mm(160.0), start_y, end_y);
    draw_v_line(&mut ctx.current_ops, right_edge, start_y, end_y);
    draw_line(&mut ctx.current_ops, COL_2, right_edge, end_y);

    ctx.y = end_y;
}
