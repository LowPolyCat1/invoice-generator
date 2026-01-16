use crate::invoice::Invoice;
use crate::pdf::context::PdfContext;
use crate::pdf::{COL_1, COL_2, PAGE_HEIGHT, PAGE_WIDTH};
use printpdf::{Mm, PdfPage};

pub fn draw_payment_details(ctx: &mut PdfContext, invoice: &Invoice, target_y: Mm) {
    ctx.y = target_y;

    let weights = vec![1, 1];
    let cols = col_pos(weights, Mm(210.0), COL_1, Mm(20.0));
    let label_w = (cols[1].0 - cols[0].0) - 2.0;

    if let Some(ref p_type) = invoice.payment_type {
        ctx.y = ctx.write_text_at_wrapping(
            &format!("Payment Type: {}", p_type).to_string(),
            10.0,
            cols[0],
            ctx.y,
            label_w,
        ) - Mm(1.);
    }

    for (label, value) in &invoice.payment_info {
        if ctx.y < Mm(20.0) {
            ctx.pages.push(PdfPage::new(
                PAGE_WIDTH,
                PAGE_HEIGHT,
                ctx.current_ops.drain(..).collect(),
            ));
            ctx.y = Mm(280.0);
        }

        ctx.y = ctx.write_text_at_wrapping(
            &format!("{}: {}", label, value),
            9.0,
            cols[0],
            ctx.y,
            label_w,
        ) - Mm(1.);
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
