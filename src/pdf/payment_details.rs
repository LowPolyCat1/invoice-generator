use crate::invoice::*;
use crate::pdf::cols::col_pos;
use crate::pdf::*;

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
