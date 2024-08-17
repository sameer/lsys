//! A crate for visualizing 2D [L-systems](https://en.wikipedia.org/wiki/L-system) with SVGs.

use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;
use svgtypes::LengthUnit;

/// Parameters for the L-system
#[derive(Debug, Clone)]
pub struct LSystem<A: AsRef<str>, R: AsRef<str>> {
    /// Initial string.
    pub axiom: A,
    /// Variables that should be treated as a stroke and drawn.
    pub variables_to_draw: HashSet<char>,
    /// Turn angle in radians.
    pub angle: Decimal,
    /// Number of times the rules will run.
    pub iterations: usize,
    /// Rules for replacing characters with a new string.
    pub rules: HashMap<char, R>,
}

/// Options to control the SVG created using [cairo](https://www.cairographics.org/).
#[derive(Debug, Clone)]
pub struct SvgOptions {
    /// Width in [`Self::units`].
    pub width: Decimal,
    /// Height in [`Self::units`].
    pub height: Decimal,
    /// Units used by the SVG
    ///
    /// <https://www.w3.org/TR/SVG/coords.html#Units>
    pub units: LengthUnit,
}

/// Error type for [`LSystem::to_svg`].
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("I/O error while writing SVG: {0}")]
    Io(#[from] std::io::Error),
}

impl<A, R> LSystem<A, R>
where
    A: AsRef<str>,
    R: AsRef<str>,
{
    /// Runs the L-system, returning its final state.
    pub fn calculate_final_state(&self) -> String {
        let mut state = self.axiom.as_ref().to_string();
        for _ in 0..self.iterations {
            state = state
                .chars()
                .flat_map(|c| {
                    match c {
                        '+' => "+",
                        '-' => "-",
                        '|' => "|",
                        '[' => "[",
                        ']' => "]",
                        letter => self
                            .rules
                            .get(&letter)
                            .expect("rule exists for every letter")
                            .as_ref(),
                    }
                    .chars()
                })
                .collect();
        }

        state
    }

    /// Run the L-system and convert it into an SVG.
    pub fn to_svg<W>(
        &self,
        SvgOptions {
            width,
            height,
            units,
        }: &SvgOptions,
        mut writer: W,
    ) -> Result<(), RenderError>
    where
        W: Write,
    {
        let final_state = self.calculate_final_state();

        let mut current_position = (Decimal::ZERO, Decimal::ZERO);
        let mut current_angle = -Decimal::HALF_PI;
        let mut strokes: Vec<((Decimal, Decimal), bool)> = vec![];
        let mut stack: Vec<((Decimal, Decimal), Decimal)> = vec![];
        for c in final_state.chars() {
            match c {
                '+' | '-' | '|' => {
                    current_angle = match c {
                        '+' => current_angle + self.angle,
                        '-' => current_angle - self.angle,
                        '|' => -current_angle,
                        _ => unreachable!(),
                    };
                }
                '[' => {
                    stack.push((current_position, current_angle));
                }
                ']' => {
                    let state = stack.pop().expect("equal number of [ and ]");
                    current_position = state.0;
                    current_angle = state.1;
                    strokes.push((current_position, true));
                }
                other if self.variables_to_draw.contains(&other) => {
                    let cos = current_angle.cos();
                    let sin = current_angle.sin();
                    current_position = (current_position.0 + cos, current_position.1 + sin);
                    strokes.push((current_position, false));
                }
                _ => {}
            }
        }

        let max = (
            strokes
                .iter()
                .max_by_key(|((x, _y), _move)| x)
                .cloned()
                .expect("at least one stroke")
                .0
                 .0,
            strokes
                .iter()
                .max_by_key(|((_x, y), _move)| y)
                .cloned()
                .expect("at least one stroke")
                .0
                 .1,
        );
        let min = (
            strokes
                .iter()
                .min_by_key(|((x, _y), _move)| x)
                .cloned()
                .expect("at least one stroke")
                .0
                 .0,
            strokes
                .iter()
                .min_by_key(|((_x, y), _move)| y)
                .cloned()
                .expect("at least one stroke")
                .0
                 .1,
        );

        let units = match units {
            LengthUnit::None => "",
            LengthUnit::Em => "em",
            LengthUnit::Ex => "ex",
            LengthUnit::Px => "px",
            LengthUnit::In => "in",
            LengthUnit::Cm => "cm",
            LengthUnit::Mm => "mm",
            LengthUnit::Pt => "pt",
            LengthUnit::Pc => "pc",
            LengthUnit::Percent => "%",
        };
        writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;

        writeln!(
            writer,
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{width}{units}" height="{height}{units}" viewBox="0 0 {width} {height}">"#
        )?;

        // 1 unit
        let stroke_width = Decimal::ONE / width.min(height);
        write!(
            writer,
            r#"<path fill="none" stroke-width="{stroke_width}" stroke-linecap="butt" stroke-linejoin="miter" stroke="rgb(0%, 0%, 0%)" stroke-opacity="1" stroke-miterlimit="10" d=""#
        )?;

        let range = ((max.0 - min.0), (max.1 - min.1));
        strokes.iter_mut().for_each(|segment| {
            *segment = (
                (
                    (segment.0 .0 - min.0) / range.0,
                    (segment.0 .1 - min.1) / range.1,
                ),
                segment.1,
            );
        });

        strokes.iter_mut().for_each(|((x, y), _)| {
            *x = x.round_dp(7);
            *y = y.round_dp(7);
        });

        if let Some(((first_segment_x, first_segment_y), _)) = strokes.pop() {
            write!(writer, "M {first_segment_x} {first_segment_y}",)?;
        }
        for ((segment_x, segment_y), is_move) in strokes {
            write!(
                writer,
                " {} {segment_x} {segment_y}",
                if is_move { 'M' } else { 'L' },
            )?;
        }

        writeln!(
            writer,
            "\" transform=\"matrix({width}, 0, 0, {height}, 0, 0)\"/>",
        )?;

        writeln!(writer, "</svg>")?;

        Ok(())
    }
}
