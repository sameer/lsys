use clap::Parser;
use lsys::LSystem;
use lsys::SvgOptions;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::path::PathBuf;
use svgtypes::LengthUnit;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Initial string.
    axiom: String,
    /// Variables that should be treated as a stroke and drawn.
    variables_to_draw: String,
    /// Turn angle in degrees.
    angle: Decimal,
    /// Number of times the rules will run.
    iterations: usize,
    /// Rules for replacing characters with a new string (i.e. "F=>F+F").
    rules: Vec<String>,

    /// Width of the SVG Canvas in millimeters.
    #[arg(long)]
    width: Decimal,
    /// Height of the SVG Canvas in millimeters.
    #[arg(long)]
    height: Decimal,

    /// Path to write the SVG to.
    #[arg(short, long, value_name = "FILE")]
    out: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let rules = args
        .rules
        .iter()
        .map(|r| {
            let (c, replacement) = r.split_once("=>").expect("rule contains =>");
            assert_eq!(c.chars().count(), 1, "=> is preceded by a single char");
            (
                c.chars().next().expect("character count is at least 1"),
                replacement,
            )
        })
        .collect::<HashMap<_, _>>();
    let variables_to_draw: HashSet<char> = HashSet::from_iter(args.variables_to_draw.chars());
    for v in variables_to_draw.iter().copied().chain(args.axiom.chars()) {
        if !rules.contains_key(&v) && !matches!(v, '+' | '-' | '|' | '[' | ']') {
            eprintln!(
                r#"There is no replacement rule for `{v}`! Assuming self-replacement ("{v}=>{v}")"#
            )
        }
    }

    let mut writer = args
        .out
        .map(|o| {
            Box::new(File::create(o).expect("valid file path with permissions")) as Box<dyn Write>
        })
        .unwrap_or_else(|| Box::new(std::io::stdout()) as Box<dyn Write>);
    LSystem {
        axiom: args.axiom,
        variables_to_draw,
        // Degrees to radians
        angle: args.angle / Decimal::from_usize(180).expect("180 is a decimal") * Decimal::PI,
        iterations: args.iterations,
        rules,
    }
    .to_svg(
        &SvgOptions {
            width: args.width,
            height: args.height,
            units: LengthUnit::Mm,
        },
        &mut writer,
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harness(
        axiom: &str,
        variables_to_draw: &[char],
        rules: &[&str],
        angle: Decimal,
        iterations: usize,
        expected: &str,
    ) {
        let mut actual = vec![];
        LSystem {
            axiom,
            variables_to_draw: HashSet::from_iter(variables_to_draw.iter().copied()),
            angle,
            iterations,
            rules: rules
                .iter()
                .map(|r| {
                    let (c, r) = r.split_once("=>").unwrap();

                    (c.chars().next().unwrap(), r)
                })
                .collect(),
        }
        .to_svg(
            &SvgOptions {
                width: Decimal::ONE_HUNDRED,
                height: Decimal::ONE_HUNDRED,
                units: LengthUnit::Mm,
            },
            &mut actual,
        )
        .unwrap();

        assert_eq!(
            expected,
            String::from_utf8(actual).expect("cairo writes valid utf8")
        );
    }

    #[test]
    fn koch() {
        harness(
            "F",
            &['F'],
            &["F=>F+F-F-F+F"],
            Decimal::HALF_PI,
            4,
            include_str!("../tests/koch.svg"),
        );
    }

    #[test]
    fn sierpinski() {
        harness(
            "F-G-G",
            &['F', 'G'],
            &["F=>F-G+F+G-F", "G=>GG"],
            Decimal::TWO_PI / Decimal::from_u32(3).unwrap(),
            6,
            include_str!("../tests/sierpinski.svg"),
        );
    }

    #[test]
    fn arrowhead() {
        harness(
            "A",
            &['A', 'B'],
            &["A=>B-A-B", "B=>A+B+A"],
            Decimal::PI / Decimal::from_u32(3).unwrap(),
            7,
            include_str!("../tests/arrowhead.svg"),
        );
    }

    #[test]
    fn dragon() {
        harness(
            "FX",
            &['F'],
            &["X=>X+YF+", "Y=>-FX-Y", "F=>F"],
            Decimal::HALF_PI,
            12,
            include_str!("../tests/dragon.svg"),
        );
    }

    #[test]
    fn plant() {
        harness(
            "X",
            &['F'],
            &["X=>F-[[X]+X]+F[+FX]-X", "F=>FF"],
            Decimal::PI * Decimal::from_u32(25).unwrap() / Decimal::from_u32(180).unwrap(),
            5,
            include_str!("../tests/plant.svg"),
        );
    }

    #[test]
    fn moore() {
        harness(
            "LFL+F+LFL",
            &['F'],
            &["L=>-RF+LFL+FR-", "R=>+LF-RFR-FL+", "F=>F"],
            Decimal::HALF_PI,
            5,
            include_str!("../tests/moore.svg"),
        );
    }

    #[test]
    fn hilbert() {
        harness(
            "A",
            &['F'],
            &["A=>-BF+AFA+FB-", "B=>+AF-BFB-FA+", "F=>F"],
            Decimal::HALF_PI,
            6,
            include_str!("../tests/hilbert.svg"),
        );
    }

    #[test]
    fn sierpinski_carpet() {
        harness(
            "F+F+F+F",
            &['F'],
            &["F=>FF+F+F+F+FF"],
            Decimal::HALF_PI,
            4,
            include_str!("../tests/sierpinski_carpet.svg"),
        );
    }

    #[test]
    fn snowflake() {
        harness(
            "F++F++F",
            &['F'],
            &["F=>F-F++F-F"],
            Decimal::PI / Decimal::from_u32(3).unwrap(),
            4,
            include_str!("../tests/snowflake.svg"),
        );
    }

    #[test]
    fn gosper() {
        harness(
            "XF",
            &['F'],
            &[
                "X=>X+YF++YF-FX--FXFX-YF+",
                "Y=>-FX+YFYF++YF+FX--FX-Y",
                "F=>F",
            ],
            Decimal::PI / Decimal::from_u32(3).unwrap(),
            5,
            include_str!("../tests/gosper.svg"),
        );
    }

    #[test]
    fn kolam() {
        harness(
            "-D--D",
            &['F'],
            &[
                "A=>F++FFFF--F--FFFF++F++FFFF--F",
                "B=>F--FFFF++F++FFFF--F--FFFF++F",
                "C=>BFA--BFA",
                "D=>CFC--CFC",
                "F=>F",
            ],
            Decimal::QUARTER_PI,
            7,
            include_str!("../tests/kolam.svg"),
        );
    }

    #[test]
    fn crystal() {
        harness(
            "F+F+F+F",
            &['F'],
            &["F=>FF+F++F+F"],
            Decimal::HALF_PI,
            4,
            include_str!("../tests/crystal.svg"),
        );
    }
}
