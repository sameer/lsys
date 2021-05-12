use cairo::Context;
use num::rational::Ratio;
use num::BigInt;
use num::ToPrimitive;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::path::Path;

fn main() {
    koch();
    sierpinski();
    arrowhead();
    dragon();
    plant();
    moore();
    hilbert();
    sierpinski_carpet();
    snowflake();
    gosper();
    kolam();
    crystal();
}

fn koch() {
    run(
        "F",
        &['F'],
        |c: char| match c {
            'F' => "F+F-F-F+F",
            _ => unreachable!(),
        },
        std::f64::consts::PI / 2.,
        4,
        Path::new("out/koch.svg"),
    );
}

fn sierpinski() {
    run(
        "F-G-G",
        &['F', 'G'],
        |c: char| match c {
            'F' => "F-G+F+G-F",
            'G' => "GG",
            _ => unreachable!(),
        },
        std::f64::consts::PI * 2. / 3.,
        6,
        Path::new("out/sierpinski.svg"),
    );
}

fn arrowhead() {
    run(
        "A",
        &['A', 'B'],
        |c: char| match c {
            'A' => "B-A-B",
            'B' => "A+B+A",
            _ => unreachable!(),
        },
        std::f64::consts::PI * 1. / 3.,
        6,
        Path::new("out/arrowhead.svg"),
    );
}

fn dragon() {
    run(
        "FX",
        &['F'],
        |c: char| match c {
            'X' => "X+YF+",
            'Y' => "-FX-Y",
            'F' => "F",
            _ => unreachable!(),
        },
        std::f64::consts::PI / 2.,
        12,
        Path::new("out/dragon.svg"),
    );
}

fn plant() {
    run(
        "X",
        &['F'],
        |c: char| match c {
            'X' => "F-[[X]+X]+F[+FX]-X",
            'F' => "FF",
            _ => unreachable!(),
        },
        std::f64::consts::PI * 25.0 / 180.0,
        5,
        Path::new("out/plant.svg"),
    );
}

fn moore() {
    run(
        "LFL+F+LFL",
        &['F'],
        |c: char| match c {
            'L' => "-RF+LFL+FR-",
            'R' => "+LF-RFR-FL+",
            'F' => "F",
            _ => unreachable!(),
        },
        std::f64::consts::PI * 90.0 / 180.0,
        5,
        Path::new("out/moore.svg"),
    );
}

fn hilbert() {
    run(
        "A",
        &['F'],
        |c: char| match c {
            'A' => "-BF+AFA+FB-",
            'B' => "+AF-BFB-FA+",
            'F' => "F",
            _ => unreachable!(),
        },
        std::f64::consts::PI / 2.0,
        6,
        Path::new("out/hilbert.svg"),
    );
}

fn sierpinski_carpet() {
    run(
        "F+F+F+F",
        &['F'],
        |c: char| match c {
            'F' => "FF+F+F+F+FF",
            _ => unreachable!(),
        },
        std::f64::consts::PI / 2.,
        4,
        Path::new("out/sierpinski_carpet.svg"),
    );
}

fn snowflake() {
    run(
        "F++F++F",
        &['F'],
        |c: char| match c {
            'F' => "F-F++F-F",
            _ => unreachable!(),
        },
        std::f64::consts::PI / 3.,
        4,
        Path::new("out/snowflake.svg"),
    );
}

fn gosper() {
    run(
        "XF",
        &['F'],
        |c: char| match c {
            'X' => "X+YF++YF-FX--FXFX-YF+",
            'Y' => "-FX+YFYF++YF+FX--FX-Y",
            'F' => "F",
            _ => unreachable!(),
        },
        std::f64::consts::PI / 3.,
        5,
        Path::new("out/gosper.svg"),
    );
}

fn kolam() {
    run(
        "-D--D",
        &['F'],
        |c: char| match c {
            'A' => "F++FFFF--F--FFFF++F++FFFF--F",
            'B' => "F--FFFF++F++FFFF--F--FFFF++F",
            'C' => "BFA--BFA",
            'D' => "CFC--CFC",
            'F' => "F",
            _ => unreachable!(),
        },
        std::f64::consts::PI / 4.0,
        6,
        Path::new("out/kolam.svg"),
    );
}

fn crystal() {
    run(
        "F+F+F+F",
        &['F'],
        |c: char| match c {
            'F' => "FF+F++F+F",
            _ => unreachable!(),
        },
        std::f64::consts::PI / 2.,
        4,
        Path::new("out/crystal.svg"),
    );
}

fn run<'a, F, P>(
    axiom: &str,
    variables_to_draw: &[char],
    rules: F,
    angle: f64,
    iterations: usize,
    path: P,
) where
    F: Fn(char) -> &'static str + Copy,
    P: Into<&'a Path>,
{
    let variables_to_draw: HashSet<char> = HashSet::from_iter(variables_to_draw.iter().copied());
    let mut state = axiom.chars().collect::<Vec<_>>();
    for _ in 0..iterations {
        state = state
            .iter()
            .map(|c| match c {
                '+' => "+",
                '-' => "-",
                '|' => "|",
                '[' => "[",
                ']' => "]",
                letter => rules(*letter),
            })
            .map(|res| res.chars())
            .flatten()
            .collect();
    }

    let mut current_position = (Ratio::from(BigInt::from(0)), Ratio::from(BigInt::from(0)));
    let mut current_angle = -std::f64::consts::PI / 2.0;
    let mut stroke: Vec<(Ratio<BigInt>, Ratio<BigInt>)> = vec![];
    let mut stack: Vec<((Ratio<BigInt>, Ratio<BigInt>), f64)> = vec![];
    for c in state.iter() {
        match c {
            '+' => {
                current_angle += angle;
            }
            '-' => {
                current_angle -= angle;
            }
            '|' => {
                current_angle = -current_angle;
            }
            '[' => {
                stack.push((current_position.clone(), current_angle));
            }
            ']' => {
                let state = stack.pop().unwrap();
                current_position = state.0;
                current_angle = state.1;
            }
            other if variables_to_draw.contains(&other) => {
                current_position = (
                    current_position.0 + Ratio::from_float(f64::cos(current_angle)).unwrap(),
                    current_position.1 + Ratio::from_float(f64::sin(current_angle)).unwrap(),
                );
                stroke.push(current_position.clone());
            }
            _ => {}
        }
    }

    const WIDTH: f64 = 1920.;
    const HEIGHT: f64 = 1080.;
    let min_width_height: f64 = WIDTH.min(HEIGHT);

    let max = (
        stroke.iter().max_by_key(|(x, _y)| x).cloned().unwrap().0,
        stroke.iter().max_by_key(|(_x, y)| y).cloned().unwrap().1,
    );
    let min = (
        stroke.iter().min_by_key(|(x, _y)| x).cloned().unwrap().0,
        stroke.iter().min_by_key(|(_x, y)| y).cloned().unwrap().1,
    );
    let range = ((max.0 - &min.0), (max.1 - &min.1));
    let min_to_zero_adjustment = (-min.0.clone(), -min.1.clone());

    let surf = cairo::SvgSurface::new(WIDTH, HEIGHT, Some(path.into())).unwrap();
    let ctx = Context::new(&surf);
    ctx.scale(min_width_height, min_width_height);

    // black background
    ctx.set_source_rgb(0., 0., 0.);
    ctx.rectangle(0., 0., WIDTH / min_width_height, HEIGHT / min_width_height);
    ctx.fill();
    // 1 px
    ctx.set_line_width(0.001); //1. / (min_width_height / 2.));
                               // white line
    ctx.set_source_rgb(1., 1., 1.);

    // convert to Cairo coordinates
    let cairo_offset = (
        Ratio::from_float((WIDTH - min_width_height) / min_width_height / 2.).unwrap(),
        Ratio::from_float((HEIGHT - min_width_height) / min_width_height / 2.).unwrap(),
    );
    stroke.iter_mut().for_each(|segment| {
        *segment = (
            (segment.0.clone() + &min_to_zero_adjustment.0) / &range.0 + &cairo_offset.0,
            (segment.1.clone() + &min_to_zero_adjustment.1) / &range.1 + &cairo_offset.1,
        );
    });
    if let Some(first_segment) = stroke.first() {
        ctx.move_to(
            first_segment.0.to_f64().unwrap(),
            first_segment.1.to_f64().unwrap(),
        );
    }
    for segment in stroke.drain(1..) {
        ctx.line_to(segment.0.to_f64().unwrap(), segment.1.to_f64().unwrap());
    }
    ctx.stroke();
}
