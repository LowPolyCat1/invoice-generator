use printpdf::*;

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

    pub fn write_text_at_wrapping(
        &mut self,
        text: &str,
        size: f32,
        x: Mm,
        y: Mm,
        max_w: f32,
    ) -> Mm {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_y = y;
        let line_height = Mm::from(Pt(size * 1.2));
        let mut line = String::new();

        for word in words {
            let estimated_width = (line.len() + word.len() + 1) as f32 * (size * 0.16);

            if estimated_width > max_w && !line.is_empty() {
                self.write_text_at(&line, size, x, current_y);
                current_y -= line_height;
                line.clear();
            }

            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }

        if !line.is_empty() {
            self.write_text_at(&line, size, x, current_y);
            current_y -= line_height;
        }

        current_y
    }

    pub fn measure_text_height(&self, text: &str, font_size: f32, max_w: f32) -> Mm {
        let avg_char_width = font_size;
        let chars_per_line = (max_w / (avg_char_width * 0.3527)).floor().max(1.0);
        let line_count = (text.len() as f32 / chars_per_line).ceil().max(1.0);

        let line_height_pts = font_size * 1.2;
        Mm(line_count * (line_height_pts * 0.352778))
    }
}
