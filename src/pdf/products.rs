use crate::invoice::*;
use crate::pdf::PAGE_HEIGHT;
use crate::pdf::PAGE_WIDTH;
use crate::pdf::PdfContext;
use crate::pdf::draw_line;
use crate::pdf::draw_v_line;
use locale_rs::Locale;
use locale_rs::currency_formats::ToCurrencyString;
use locale_rs::num_formats::ToFormattedString;
use printpdf::*;

pub fn draw_products(
    ctx: &mut PdfContext,
    invoice: &Invoice,
    locale: &Locale,
    cols: &Vec<Mm>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_page_top = ctx.y + Mm(4.0);
    let right_edge = *cols.last().unwrap();

    for p in &invoice.products {
        if ctx.y < Mm(30.0) {
            draw_table_borders_dynamic(ctx, cols, current_page_top, ctx.y);

            ctx.pages.push(PdfPage::new(
                PAGE_WIDTH,
                PAGE_HEIGHT,
                ctx.current_ops.drain(..).collect(),
            ));

            ctx.y = Mm(280.0);
            current_page_top = ctx.y + Mm(5.0);
        }

        let start_y = ctx.y;
        let line_total = (p.units as f64 * p.cost_per_unit) + p.tax_rate;

        let units_w = (cols[2].0 - cols[1].0) - 4.0;
        let cost_w = (cols[3].0 - cols[2].0) - 4.0;
        let tax_w = (cols[4].0 - cols[3].0) - 4.0;
        let total_w = (right_edge.0 - cols[4].0) - 4.0;
        let desc_w = (cols[1].0 - cols[0].0) - 4.0;

        let y1 =
            ctx.write_text_at_wrapping(&p.description, 9.0, cols[0] + Mm(2.0), start_y, desc_w);
        let y2 = ctx.write_text_at_wrapping(
            &p.units.to_string(),
            9.0,
            cols[1] + Mm(2.0),
            start_y,
            units_w,
        );
        let y3 = ctx.write_text_at_wrapping(
            &p.cost_per_unit.to_currency(locale),
            9.0,
            cols[2] + Mm(2.0),
            start_y,
            cost_w,
        );

        let tax_val = match &p.tax_exempt_reason {
            Some(reason) => reason.clone(),
            None => p.tax_rate.to_formatted_string(locale),
        };
        let y4 = ctx.write_text_at_wrapping(&tax_val, 9.0, cols[3] + Mm(2.0), start_y, tax_w);
        let y5 = ctx.write_text_at_wrapping(
            &line_total.to_currency(locale),
            9.0,
            cols[4] + Mm(2.0),
            start_y,
            total_w,
        );

        let row_bottom = y1.min(y2).min(y3).min(y4).min(y5);

        ctx.y = row_bottom.min(start_y - Mm(6.0));
    }

    draw_table_borders_dynamic(ctx, cols, current_page_top, ctx.y);

    Ok(())
}

fn draw_table_borders_dynamic(ctx: &mut PdfContext, cols: &Vec<Mm>, top: Mm, bottom: Mm) {
    let right_edge = *cols.last().unwrap();

    draw_line(&mut ctx.current_ops, cols[0], right_edge, top);
    draw_line(&mut ctx.current_ops, cols[0], right_edge, bottom);

    for &x in cols {
        draw_v_line(&mut ctx.current_ops, x, top, bottom);
    }
}
