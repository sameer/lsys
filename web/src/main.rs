#![allow(non_snake_case)]

use std::{
    collections::{HashMap, HashSet},
    iter::once,
};

use base64::Engine;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use lsys::{LSystem, SvgOptions};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use svgtypes::LengthUnit;
use wasm_bindgen::JsCast;
use web_sys::window;

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

fn App() -> Element {
    let mut axiom = use_signal(|| "F".to_string());
    let mut rules = use_signal(|| "F=>F+F-F-F+F".to_string());
    let mut variables_to_draw = use_signal(|| "F".to_string());
    let mut angle = use_signal(|| 90.to_string());
    let mut iterations = use_signal(|| 4.to_string());
    let mut svg: Signal<Option<String>> = use_signal(Option::default);
    let mut copied: Signal<bool> = use_signal(|| false);
    let mut examples_open: Signal<bool> = use_signal(|| false);

    let rules_result: Result<HashMap<char, String>, &'static str> = rules
        .read()
        .lines()
        .map(|r| {
            let (c, replacement) = r.split_once("=>").ok_or("each rule must contain =>")?;
            if c.chars().count() != 1 {
                return Err("=> must be preceded by a single char");
            }
            if replacement.is_empty() {
                return Err("=> must be followed by a replacement string");
            }
            Ok((c.chars().next().unwrap(), replacement.to_string()))
        })
        .collect::<Result<HashMap<_, _>, _>>()
        .and_then(|rules| {
            if !axiom
                .read()
                .chars()
                .filter(|c| !matches!(c, '+' | '-' | '|' | '[' | ']'))
                .all(|c| rules.contains_key(&c))
            {
                Err("missing rule")
            } else {
                Ok(rules)
            }
        });
    let variables_to_draw_result = match rules_result.as_ref() {
        Ok(rules) => {
            let variables = axiom
                .read()
                .chars()
                .chain(rules.keys().copied())
                .chain(rules.values().flat_map(|s| s.chars()))
                .collect::<HashSet<_>>();

            let mut res = Ok(variables_to_draw.read().clone());
            for c in variables_to_draw.read().chars() {
                if !variables.contains(&c) {
                    res = Err(format!("unknown variable: {c}"));
                    break;
                }
            }
            res
        }
        Err(_) => Ok(variables_to_draw.read().clone()),
    };
    let angle_result = angle.read().parse::<Decimal>();
    let iterations_result = dbg!(iterations.read()).parse::<usize>();

    let l_system = match (
        rules_result.as_ref(),
        variables_to_draw_result.as_ref(),
        angle_result.as_ref(),
        iterations_result.as_ref(),
    ) {
        (Ok(rules), Ok(variables_to_draw), Ok(angle), Ok(iterations)) => Some(LSystem {
            axiom: axiom.read().clone(),
            rules: rules.clone(),
            variables_to_draw: variables_to_draw.chars().collect(),
            angle: angle.clone() / Decimal::from_usize(180).expect("180 is a decimal")
                * Decimal::PI,
            iterations: *iterations,
        }),
        _ => None,
    };

    let copy_onclick = {
        move |_| {
            spawn(async move {
                let _ = wasm_bindgen_futures::JsFuture::from(
                    window()
                        .unwrap()
                        .navigator()
                        .clipboard()
                        .write_text(svg.read().as_ref().unwrap()),
                )
                .await;
                copied.set(true);
            });
        }
    };
    rsx! {
        main {
            form {
                onsubmit: move |_| {
                    let mut acc = vec![];
                    l_system
                        .as_ref()
                        .expect("checked for errors")
                        .to_svg(
                            &SvgOptions {
                                width: Decimal::try_from(500.).unwrap(),
                                height: Decimal::try_from(500.).unwrap(),
                                units: LengthUnit::Px,
                            },
                            &mut acc,
                        )
                        .unwrap();
                    svg.set(Some(String::from_utf8(acc).unwrap()));
                    copied.set(false);
                },
                fieldset { class: "grid",
                    label {
                        "Axiom"
                        input {
                            name: "axiom",
                            r#type: "text",
                            autocomplete: "off",
                            value: axiom,
                            oninput: move |event| axiom.set(event.value())
                        }
                    }
                    label {
                        "Rules"
                        textarea {
                            name: "rules",
                            value: rules,
                            aria_invalid: rules_result.is_err(),
                            aria_describedby: "rules-helper",
                            oninput: move |event| rules.set(event.value())
                        }
                        if let Err(err) = rules_result.as_ref() {
                            small { id: "rules-helper", {err} }
                        }
                    }
                    label {
                        "Variables to draw"
                        input {
                            name: "variables_to_draw",
                            r#type: "text",
                            autocomplete: "off",
                            value: variables_to_draw,
                            aria_invalid: variables_to_draw_result.is_err(),
                            aria_describedby: "variables-to-draw-helper",
                            oninput: move |event| variables_to_draw.set(event.value())
                        }
                        if let Err(err) = variables_to_draw_result.as_ref() {
                            small { id: "variables-to-draw-helper", {err.clone()} }
                        }
                    }
                }
                fieldset { class: "grid",
                    label {
                        "Angle"
                        input {
                            name: "angle",
                            r#type: "text",
                            autocomplete: "off",
                            value: angle,
                            aria_invalid: angle_result.is_err(),
                            aria_describedby: "angle-helper",
                            oninput: move |event| angle.set(event.value())
                        }
                        if let Err(err) = angle_result.as_ref() {
                            small { id: "angles-helper", {err.to_string()} }
                        } else {
                            small { id: "angles-helper", "in degrees" }
                        }
                    }
                    label {
                        "Iterations"
                        input {
                            name: "iterations",
                            r#type: "text",
                            autocomplete: "off",
                            value: iterations,
                            aria_invalid: iterations_result.is_err(),
                            aria_describedby: "iterations-helper",
                            oninput: move |event| iterations.set(event.value())
                        }
                        if let Err(err) = iterations_result.as_ref() {
                            small { id: "iterations-helper", {err.to_string()} }
                        }
                    }
                }
                fieldset { class: "grid",
                    input {
                        r#type: "submit",
                        value: "Generate",
                        disabled: rules_result.is_err() || variables_to_draw_result.is_err() || angle_result.is_err()
                            || iterations_result.is_err()
                    }
                    a {
                        onclick: move |_| examples_open.set(true),
                        role: "button",
                        style: "margin-bottom:var(--pico-spacing)",
                        "Examples"
                    }
                }
            }

            if let Some(svg) = svg.read().as_ref() {
                article {
                    div { dangerous_inner_html: lsys_svg_to_color_scheme_aware_svg(svg) }
                    footer {
                        div { class: "grid",
                            a { role: "button", onclick: copy_onclick,
                                if *copied.read() {
                                    "Copied ✅"
                                } else {
                                    "Copy 📋"
                                }
                            }
                            a {
                                role: "button",
                                href: format!(
                                    "data:image/svg+xml;base64,{}",
                                    base64::engine::general_purpose::STANDARD_NO_PAD.encode(svg),
                                ),
                                download: "lsys.svg",
                                "Download ⬇️"
                            }
                        }
                    }
                }
            }

            dialog {
                open: examples_open,
                onclick: move |event| {
                    let data = event.data();
                    let inner = data.downcast::<web_sys::MouseEvent>().expect("using dioxus web");
                    let examples_article = window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .query_selector("#examples-article")
                        .unwrap()
                        .unwrap();
                    if let Some(target) = inner
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    {
                        if !examples_article.contains(Some(&target)) {
                            examples_open.set(false);
                        }
                    }
                },
                article { id: "examples-article",
                    header {
                        a {
                            role: "button",
                            aria_label: "Close",
                            rel: "prev",
                            onclick: move |_| examples_open.set(false)
                        }
                        p { "Example L-systems" }
                    }
                    div {
                        for examples in EXAMPLES.chunks(3) {
                            div { class: "grid",
                                for example in examples {
                                    a {
                                        role: "button",
                                        onclick: move |_| {
                                            axiom.set(example.axiom.to_string());
                                            rules.set(example.rules.join("\n"));
                                            variables_to_draw.set(example.variables_to_draw.to_string());
                                            angle.set(example.angle.to_string());
                                            iterations.set(example.iterations.to_string());
                                            examples_open.set(false);
                                        },
                                        {example.name}
                                    }
                                }
                                for _ in examples.len()..3 {
                                    div {}
                                }
                            }
                            br {}
                        }
                    }
                }
            }
        }
    }
}

fn lsys_svg_to_color_scheme_aware_svg(svg: &str) -> String {
    let style = "<style>\
        @media (prefers-color-scheme: light) { path { stroke: black; } }\
        @media (prefers-color-scheme: dark) { path { stroke: white; } }\
    </style>";
    svg.lines()
        .take(2)
        .chain(once(style))
        .chain(svg.lines().skip(2))
        .collect()
}

struct Example {
    name: &'static str,
    axiom: &'static str,
    variables_to_draw: &'static str,
    rules: &'static [&'static str],
    angle: f64,
    iterations: usize,
}
const EXAMPLES: &[Example] = &[
    Example {
        name: "Koch",
        axiom: "F",
        variables_to_draw: "F",
        rules: &["F=>F+F-F-F+F"],
        angle: 90.,
        iterations: 4,
    },
    Example {
        name: "Sierpinski Triangle",
        axiom: "F-G-G",
        variables_to_draw: "FG",
        rules: &["F=>F-G+F+G-F", "G=>GG"],
        angle: 120.,
        iterations: 6,
    },
    Example {
        name: "Sierpinski Arrowhead",
        axiom: "A",
        variables_to_draw: "AB",
        rules: &["A=>B-A-B", "B=>A+B+A"],
        angle: 60.,
        iterations: 7,
    },
    Example {
        name: "Dragon",
        axiom: "FX",
        variables_to_draw: "F",
        rules: &["X=>X+YF+", "Y=>-FX-Y", "F=>F"],
        angle: 90.,
        iterations: 4,
    },
    Example {
        name: "Plant",
        axiom: "X",
        variables_to_draw: "F",
        rules: &["X=>F-[[X]+X]+F[+FX]-X", "F=>FF"],
        angle: 25.,
        iterations: 5,
    },
    Example {
        name: "Moore",
        axiom: "LFL+F+LFL",
        variables_to_draw: "F",
        rules: &["L=>-RF+LFL+FR-", "R=>+LF-RFR-FL+", "F=>F"],
        angle: 90.,
        iterations: 5,
    },
    Example {
        name: "Hilbert",
        axiom: "A",
        variables_to_draw: "F",
        rules: &["A=>-BF+AFA+FB-", "B=>+AF-BFB-FA+", "F=>F"],
        angle: 90.,
        iterations: 6,
    },
    Example {
        name: "Sierpinski Carpet",
        axiom: "F+F+F+F",
        variables_to_draw: "F",
        rules: &["F=>FF+F+F+F+FF"],
        angle: 90.,
        iterations: 4,
    },
    Example {
        name: "Snowflake",
        axiom: "F++F++F",
        variables_to_draw: "F",
        rules: &["F=>F-F++F-F"],
        angle: 60.,
        iterations: 4,
    },
    Example {
        name: "Gosper",
        axiom: "XF",
        variables_to_draw: "F",
        rules: &[
            "X=>X+YF++YF-FX--FXFX-YF+",
            "Y=>-FX+YFYF++YF+FX--FX-Y",
            "F=>F",
        ],
        angle: 60.,
        iterations: 5,
    },
    Example {
        name: "Kolam",
        axiom: "-D--D",
        variables_to_draw: "F",
        rules: &[
            "A=>F++FFFF--F--FFFF++F++FFFF--F",
            "B=>F--FFFF++F++FFFF--F--FFFF++F",
            "C=>BFA--BFA",
            "D=>CFC--CFC",
            "F=>F",
        ],
        angle: 45.,
        iterations: 7,
    },
];
