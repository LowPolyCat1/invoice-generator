use crate::invoice::Invoice;
use crate::pdf::context::PdfContext;
use crate::pdf::drawing::{draw_line, draw_v_line};
use crate::pdf::{PAGE_HEIGHT, PAGE_WIDTH, COL_1};
use locale_rs::currency_formats::ToCurrencyString;
use locale_rs::num_formats::ToFormattedString;
use locale_rs::Locale;
use printpdf::{Mm, PdfPage};

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

fn draw_products(
    ctx: &mut PdfContext,
    invoice: &Invoice,
    locale: &Locale,
    cols: &[Mm],
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

fn draw_table_borders_dynamic(ctx: &mut PdfContext, cols: &[Mm], top: Mm, bottom: Mm) {
    let right_edge = *cols.last().unwrap();

    draw_line(&mut ctx.current_ops, cols[0], right_edge, top);
    draw_line(&mut ctx.current_ops, cols[0], right_edge, bottom);

    for &x in cols {
        draw_v_line(&mut ctx.current_ops, x, top, bottom);
    }
}

fn col_pos(weights: Vec<i16>, page_width: Mm, margin_left: Mm, margin_right: Mm) -> Vec<Mm> {
    let total_weight: i16 = weights.iter().sum();
    let available_width = page_width.0 - margin_left.0 - margin_right.0;

    let mut positions = Vec::new();
    let mut current_x = margin_left.0;

    for weight in weights {
        positions.push(Mm(current_x));

        let col_width = (weight as f32 / total_weight as f32) * available_width;

        current_x += col_width;
    }

    positions.push(Mm(current_x));

    positions
}
