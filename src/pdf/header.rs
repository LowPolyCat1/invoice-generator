use crate::invoice::*;
use crate::pdf::*;

pub fn draw_header_info(ctx: &mut PdfContext, invoice: &Invoice) {
    ctx.write_text_at(
        &format!("INVOICE ID: {}", invoice.number),
        14.0,
        COL_1,
        ctx.y,
    );
    ctx.y -= Mm(6.0);
    ctx.write_text_at(
        &format!("PAYMENT DUE: {}", invoice.payment_due),
        10.0,
        COL_2,
        ctx.y,
    );

    ctx.write_text_at(&format!("DATE: {}", invoice.date), 10.0, COL_1, ctx.y);
    ctx.y -= Mm(6.0);
    ctx.write_text_at(
        &format!("DELIVERY DATE: {}", invoice.delivery_date),
        10.0,
        COL_2,
        ctx.y,
    );

    for (label, value) in &invoice.extra_info {
        ctx.write_text_at(
            &format!("{}: {}", label.to_uppercase(), value),
            10.0,
            COL_1,
            ctx.y,
        );
        ctx.y -= Mm(5.0);
    }

    draw_line(&mut ctx.current_ops, COL_1, Mm(190.0), ctx.y);
}
