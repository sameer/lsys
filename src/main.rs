use cairo::Context;
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
            'F' => "F+F-F-F+F".chars().collect(),
            other => vec![other],
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
            'F' => "F-G+F+G-F".chars().collect(),
            'G' => "GG".chars().collect(),
            other => vec![other],
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
            'A' => "B-A-B".chars().collect(),
            'B' => "A+B+A".chars().collect(),
            other => vec![other],
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
            'X' => "X+YF+".chars().collect(),
            'Y' => "-FX-Y".chars().collect(),
            other => vec![other],
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
            'X' => "F-[[X]+X]+F[+FX]-X".chars().collect(),
            'F' => "FF".chars().collect(),
            other => vec![other],
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
            'L' => "-RF+LFL+FR-".chars().collect(),
            'R' => "+LF-RFR-FL+".chars().collect(),
            other => vec![other],
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
            'A' => "-BF+AFA+FB-".chars().collect(),
            'B' => "+AF-BFB-FA+".chars().collect(),
            other => vec![other],
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
            'F' => "FF+F+F+F+FF".chars().collect(),
            other => vec![other],
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
            'F' => "F-F++F-F".chars().collect(),
            other => vec![other],
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
            'X' => "X+YF++YF-FX--FXFX-YF+".chars().collect(),
            'Y' => "-FX+YFYF++YF+FX--FX-Y".chars().collect(),
            other => vec![other],
        },
        std::f64::consts::PI / 3.,
        4,
        Path::new("out/gosper.svg"),
    );
}

fn kolam() {
    run(
        "-D--D",
        &['F'],
        |c: char| match c {
            'A' => "F++FFFF--F--FFFF++F++FFFF--F".chars().collect(),
            'B' => "F--FFFF++F++FFFF--F--FFFF++F".chars().collect(),
            'C' => "BFA--BFA".chars().collect(),
            'D' => "CFC--CFC".chars().collect(),
            other => vec![other],
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
            'F' => "FF+F++F+F".chars().collect(),
            other => vec![other],
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
    F: Fn(char) -> Vec<char> + Copy,
    P: Into<&'a Path>
{
    let surf = cairo::SvgSurface::new(1024.0, 1024.0, Some(path.into())).unwrap();
    let ctx = Context::new(&surf);
    ctx.scale(1024., 1024.);

    ctx.set_line_width(0.001);
    ctx.set_source_rgb(0., 0., 0.);

    let mut state = axiom.to_string();

    for _ in 0..iterations {
        state = state.chars().map(rules).flatten().collect();
    }

    // let segment_count = state.chars().filter(|c| variables.contains(&c)).count();

    ctx.move_to(0.5, 0.5);
    let mut stack: Vec<((f64, f64), f64)> = vec![];
    let mut curangle = -std::f64::consts::PI / 2.0;
    for c in state.chars() {
        match c {
            '+' => {
                // ctx.rotate(angle);
                curangle += angle;
            }
            '-' => {
                // ctx.rotate(-angle);
                curangle -= angle;
            }
            '|' => {
                curangle = -curangle;
            }
            '[' => {
                stack.push((ctx.get_current_point(), curangle));
            }
            ']' => {
                let state = stack.pop().unwrap();
                ctx.move_to((state.0).0, (state.0).1);
                curangle = state.1;
            }
            other => {
                if variables_to_draw.contains(&other) {
                    ctx.rel_line_to(0.005 * f64::cos(curangle), 0.005 * f64::sin(curangle));
                }
            }
        }
    }
    ctx.stroke();
}
