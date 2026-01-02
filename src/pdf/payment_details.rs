use crate::invoice::*;
use crate::pdf::cols::col_pos;
use crate::pdf::*;

pub fn draw_payment_details(ctx: &mut PdfContext, invoice: &Invoice, target_y: Mm) {
    ctx.y = target_y;

    // Label: 1 (e.g. 30%), Value: 2 (e.g. 70%)
    let weights = vec![1, 1];
    let cols = col_pos(weights, Mm(210.0), COL_1, Mm(20.0));

    if let Some(ref p_type) = invoice.payment_type {
        ctx.write_text_at("Payment Method:", 10.0, cols[0], ctx.y);
        ctx.write_text_at(p_type, 10.0, cols[1], ctx.y);
        ctx.y -= Mm(6.0);
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

        ctx.write_text_at(&format!("{}:", label), 9.0, cols[0], ctx.y);

        // Use wrapping for the value in case bank details/IBANs are long
        let val_width = cols[2].0 - cols[1].0;
        ctx.y = ctx.wrap_text_ops(value, val_width, 9.0, cols[1]);
        ctx.y -= Mm(2.0);
    }
}
