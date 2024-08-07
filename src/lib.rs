//! A crate for visualizing 2D [L-systems](https://en.wikipedia.org/wiki/L-system) with SVGs.

use cairo::Context;
use cairo::StreamWithError;
use cairo::SvgUnit;
use num::rational::Ratio;
use num::BigInt;
use num::ToPrimitive;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;

/// Parameters for the L-system
#[derive(Debug, Clone)]
pub struct LSystem<A: AsRef<str>, R: AsRef<str>> {
    /// Initial string.
    pub axiom: A,
    /// Variables that should be treated as a stroke and drawn.
    pub variables_to_draw: HashSet<char>,
    /// Turn angle in degrees.
    pub angle: f64,
    /// Number of times the rules will run.
    pub iterations: usize,
    /// Rules for replacing characters with a new string.
    pub rules: HashMap<char, R>,
}

/// Options to control the SVG created using [cairo](https://www.cairographics.org/).
#[derive(Debug, Clone)]
pub struct SvgOptions {
    /// Width in [`Self::units`].
    pub width: f64,
    /// Height in [`Self::units`].
    pub height: f64,
    /// Units used by the SVG
    ///
    /// <https://www.w3.org/TR/SVG/coords.html#Units>
    pub units: SvgUnit,
}

/// Error type for [`LSystem::to_svg`].
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Error in Cairo operation: {0}")]
    Cairo(#[from] cairo::Error),
    #[error("Cannot convert float to a rational number: {0}")]
    NoRationalRepr(f64),
    #[error("Cannot convert rational number to a float: {0}")]
    NoFloatRepr(num::BigRational),
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
    ///
    /// Safety: there is an `unsafe` block in this method to allow arbitrary writer lifetimes.
    /// It is safe because the [`cairo::svg::SvgSurface`] stays within this method.
    pub fn to_svg<W>(
        &self,
        SvgOptions {
            width,
            height,
            units,
        }: &SvgOptions,
        writer: &mut W,
    ) -> Result<(), RenderError>
    where
        W: Write + 'static,
    {
        let final_state = self.calculate_final_state();

        let mut current_position = (Ratio::from(BigInt::from(0)), Ratio::from(BigInt::from(0)));
        let mut current_angle = -std::f64::consts::PI / 2.0;
        let mut strokes: Vec<((Ratio<BigInt>, Ratio<BigInt>), bool)> = vec![];
        let mut stack: Vec<((Ratio<BigInt>, Ratio<BigInt>), f64)> = vec![];
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
                    stack.push((current_position.clone(), current_angle));
                }
                ']' => {
                    let state = stack.pop().expect("equal number of [ and ]");
                    current_position = state.0;
                    current_angle = state.1;
                    strokes.push((current_position.clone(), true));
                }
                other if self.variables_to_draw.contains(&other) => {
                    let cos = f64::cos(current_angle);
                    let sin = f64::sin(current_angle);
                    current_position = (
                        current_position.0
                            + Ratio::from_float(cos).ok_or(RenderError::NoRationalRepr(cos))?,
                        current_position.1
                            + Ratio::from_float(sin).ok_or(RenderError::NoRationalRepr(sin))?,
                    );
                    strokes.push((current_position.clone(), false));
                }
                _ => {}
            }
        }

        let min_width_height: f64 = width.min(*height);

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
        let range = ((max.0 - &min.0), (max.1 - &min.1));
        let min_to_zero_adjustment = (-min.0.clone(), -min.1.clone());

        let mut surf = unsafe { cairo::SvgSurface::for_raw_stream(*width, *height, writer)? };
        surf.set_document_unit(*units);
        let ctx = Context::new(&surf)?;
        ctx.scale(min_width_height, min_width_height);

        // 1 unit
        ctx.set_line_width(1. / min_width_height);
        // black line
        ctx.set_source_rgb(0., 0., 0.);

        // convert to Cairo coordinates
        let offset = (
            (width - min_width_height) / min_width_height / 2.,
            (height - min_width_height) / min_width_height / 2.,
        );
        let cairo_offset = (
            Ratio::from_float(offset.0).ok_or(RenderError::NoRationalRepr(offset.0))?,
            Ratio::from_float(offset.1).ok_or(RenderError::NoRationalRepr(offset.1))?,
        );
        strokes.iter_mut().for_each(|segment| {
            *segment = (
                (
                    (segment.0 .0.clone() + &min_to_zero_adjustment.0) / &range.0 + &cairo_offset.0,
                    (segment.0 .1.clone() + &min_to_zero_adjustment.1) / &range.1 + &cairo_offset.1,
                ),
                segment.1,
            );
        });
        if let Some(((first_segment_x, first_segment_y), _)) = strokes.first() {
            ctx.move_to(
                first_segment_x
                    .to_f64()
                    .ok_or_else(|| RenderError::NoFloatRepr(first_segment_x.clone()))?,
                first_segment_y
                    .to_f64()
                    .ok_or_else(|| RenderError::NoFloatRepr(first_segment_y.clone()))?,
            );
        }
        for ((segment_x, segment_y), is_move) in strokes.drain(1..) {
            if is_move {
                ctx.stroke()?;
                ctx.move_to(
                    segment_x
                        .to_f64()
                        .ok_or_else(|| RenderError::NoFloatRepr(segment_x.clone()))?,
                    segment_y
                        .to_f64()
                        .ok_or_else(|| RenderError::NoFloatRepr(segment_y.clone()))?,
                );
            } else {
                ctx.line_to(
                    segment_x
                        .to_f64()
                        .ok_or_else(|| RenderError::NoFloatRepr(segment_x.clone()))?,
                    segment_y
                        .to_f64()
                        .ok_or_else(|| RenderError::NoFloatRepr(segment_y.clone()))?,
                );
            }
        }
        ctx.stroke()?;

        // Safety: explicitly release the writer
        surf.finish_output_stream()
            .map_err(|StreamWithError { error, .. }: StreamWithError| RenderError::from(error))?;

        Ok(())
    }
}
