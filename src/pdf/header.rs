use crate::invoice::*;
use crate::pdf::*;

pub fn draw_header_info(ctx: &mut PdfContext, invoice: &Invoice) {
    ctx.write_text_at(
        &format!("Invoice Id: {}", invoice.number),
        14.0,
        COL_1,
        ctx.y,
    );
    ctx.y -= Mm(6.0);
    ctx.write_text_at(
        &format!("Payment Due: {}", invoice.payment_due),
        10.0,
        COL_2,
        ctx.y,
    );

    ctx.write_text_at(&format!("Date: {}", invoice.date), 10.0, COL_1, ctx.y);
    ctx.y -= Mm(6.0);
    ctx.write_text_at(
        &format!("Delivery Date: {}", invoice.delivery_date),
        10.0,
        COL_2,
        ctx.y,
    );

    for (label, value) in &invoice.extra_info {
        ctx.write_text_at(&format!("{}: {}", label, value), 10.0, COL_1, ctx.y);
        ctx.y -= Mm(5.0);
    }

    draw_line(&mut ctx.current_ops, COL_1, Mm(190.0), ctx.y);
}
