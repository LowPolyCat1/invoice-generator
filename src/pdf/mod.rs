use printpdf::*;

mod generate;
mod logo;
mod products;
pub use generate::generate_invoice_pdf;
mod addresses;
mod fin_summary;
mod header;
mod payment_details;
mod product_table;

const LEFT_MARGIN: f32 = 20.0;
const COL_1: Mm = Mm(LEFT_MARGIN);
const COL_2: Mm = Mm(120.0);
const PAGE_WIDTH: Mm = Mm(210.0);
const PAGE_HEIGHT: Mm = Mm(297.0);

pub struct PdfContext {
    pub font_id: FontId,
    pub pages: Vec<PdfPage>,
    pub current_ops: Vec<Op>,
    pub y: Mm,
}

impl PdfContext {
    pub fn new(font_id: FontId) -> Self {
        Self {
            font_id,
            pages: Vec::new(),
            current_ops: Vec::new(),
            y: Mm(280.0),
        }
    }

    pub fn write_text(&mut self, text: &str, size: f32, x: Mm) {
        if text.trim().is_empty() {
            return;
        }
        self.current_ops.push(Op::StartTextSection);
        self.current_ops.push(Op::SetTextCursor {
            pos: Point {
                x: x.into(),
                y: self.y.into(),
            },
        });
        self.current_ops.push(Op::SetFontSize {
            font: self.font_id.clone(),
            size: Pt(size),
        });
        self.current_ops.push(Op::WriteText {
            items: vec![TextItem::Text(text.to_string())],
            font: self.font_id.clone(),
        });
        self.current_ops.push(Op::EndTextSection);
        self.y -= Mm(size * 0.45);
    }

    pub fn write_text_at(&mut self, text: &str, size: f32, x: Mm, y: Mm) {
        self.current_ops.push(Op::StartTextSection);
        self.current_ops.push(Op::SetTextCursor {
            pos: Point {
                x: x.into(),
                y: y.into(),
            },
        });
        self.current_ops.push(Op::SetFontSize {
            font: self.font_id.clone(),
            size: Pt(size),
        });
        self.current_ops.push(Op::WriteText {
            items: vec![TextItem::Text(text.to_string())],
            font: self.font_id.clone(),
        });
        self.current_ops.push(Op::EndTextSection);
    }

    pub fn wrap_text_ops(&mut self, t: &str, max_w: f32, sz: f32, x: Mm) -> Mm {
        let words: Vec<&str> = t.split_whitespace().collect();
        let mut line = String::new();
        for word in words {
            if (line.len() + word.len()) as f32 * (sz * 0.16) > max_w {
                self.write_text(&line, sz, x);
                line.clear();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        self.write_text(&line, sz, x);
        self.y
    }
}

pub fn draw_line(ops: &mut Vec<Op>, x1: Mm, x2: Mm, y: Mm) {
    ops.push(Op::SetOutlineColor {
        col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
    });
    ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point {
                        x: x1.into(),
                        y: y.into(),
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: x2.into(),
                        y: y.into(),
                    },
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
}
