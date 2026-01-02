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
    cols: &Vec<Mm>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_page_top = ctx.y + Mm(4.0);

    for p in &invoice.products {
        if ctx.y < Mm(30.0) {
            draw_table_borders_dynamic(ctx, cols, current_page_top, ctx.y);
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

        // Use column indices for placement
        ctx.write_text_at(&p.units.to_string(), 9.0, cols[1] + Mm(2.0), text_y);
        ctx.write_text_at(
            &format_currency(p.cost_per_unit, &invoice.currency_code, locale),
            9.0,
            cols[2] + Mm(2.0),
            text_y,
        );

        let tax_val = match &p.tax_exempt_reason {
            Some(reason) => reason.clone(),
            None => format_currency(p.tax_rate, &invoice.currency_code, locale),
        };
        ctx.write_text_at(&tax_val, 9.0, cols[3] + Mm(2.0), text_y);

        ctx.write_text_at(
            &format_currency(line_total + p.tax_rate, &invoice.currency_code, locale),
            9.0,
            cols[4] + Mm(2.0),
            text_y,
        );

        // Wrap the description based on the width of the first column
        let desc_width = (cols[1].0 - cols[0].0) - 4.0; // Subtract padding
        ctx.y = ctx.wrap_text_ops(&p.description, desc_width, 9.0, cols[0] + Mm(2.0));
        ctx.y -= Mm(4.0);
    }

    draw_table_borders_dynamic(ctx, cols, current_page_top, ctx.y);
    Ok(())
}

fn draw_table_borders_dynamic(ctx: &mut PdfContext, cols: &Vec<Mm>, top: Mm, mut bottom: Mm) {
    bottom += Mm(5.0);
    let right_edge = *cols.last().unwrap();
    draw_line(&mut ctx.current_ops, cols[0], right_edge, top);
    draw_line(&mut ctx.current_ops, cols[0], right_edge, bottom);

    for &x in cols {
        draw_v_line(&mut ctx.current_ops, x, top, bottom);
    }
}
