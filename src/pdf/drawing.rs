use printpdf::*;

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

pub fn draw_v_line(ops: &mut Vec<Op>, x: Mm, y1: Mm, y2: Mm) {
    ops.push(Op::SetOutlineColor {
        col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
    });
    ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point {
                        x: x.into(),
                        y: y1.into(),
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: x.into(),
                        y: y2.into(),
                    },
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
}
