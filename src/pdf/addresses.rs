use crate::invoice::*;
use crate::pdf::*;
pub fn draw_address_section(ctx: &mut PdfContext, invoice: &Invoice) {
    ctx.y -= Mm(8.0);
    let addr_y = ctx.y;

    ctx.write_text_at("SOLD BY:", 8.0, COL_1, addr_y);
    ctx.write_text_at("BILLED TO:", 8.0, COL_2, addr_y);

    ctx.y = addr_y - Mm(5.0);
    ctx.write_text(&invoice.seller.name, 10.0, COL_1);
    for line in invoice.seller.address.lines() {
        ctx.write_text(line, 9.0, COL_1);
    }
    ctx.write_text(&format!("VAT: {}", invoice.seller.vat_id), 9.0, COL_1);
    let seller_end_y = ctx.y;

    ctx.y = addr_y - Mm(5.0);
    ctx.write_text(&invoice.buyer.name, 10.0, COL_2);
    for line in invoice.buyer.address.lines() {
        ctx.write_text(line, 9.0, COL_2);
    }
    ctx.write_text(&invoice.buyer.email, 9.0, COL_2);
    let buyer_end_y = ctx.y;

    ctx.y = seller_end_y.min(buyer_end_y);
}
