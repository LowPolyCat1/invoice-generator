use crate::invoice::*;
use crate::pdf::*;

pub fn draw_payment_details(ctx: &mut PdfContext, invoice: &Invoice, target_y: Mm) {
    ctx.y = target_y;

    if let Some(ref p_type) = invoice.payment_type {
        ctx.write_text_at(&format!("Payment Method: {}", p_type), 10.0, COL_1, ctx.y);
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
        ctx.write_text_at(&format!("{}: {}", label, value), 9.0, COL_1, ctx.y);
        ctx.y -= Mm(5.0);
    }
}
