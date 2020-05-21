/// A length, either relative or absolute. Generally compatible with [CSS
/// lengths].
///
/// Several [TeX units] were ommitted for being obscure and useless: traditional
/// points (1/72.27 in), (new) didots, (new) cieros, scaled points.
///
/// [CSS lengths]: https://developer.mozilla.org/en-US/docs/Web/CSS/length
/// [TeX units]: https://en.wikibooks.org/wiki/LaTeX/Lengths#Units
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Absolute(AbsLength),
    Relative(RelLength),
}

/// An absolute length, i.e. resolvable immediately to points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AbsLength {
    /// Point = 1/72 in.
    ///
    /// More formally, this is a "big point". Traditionally, a point has measured
    /// 1/72.27 inches.
    Pt(f64),
    /// Pica = 12pt = 1/6 in.
    Pc(f64),
    /// Inch.
    In(f64),
    /// Centimeter = 1/100 m.
    Cm(f64),
    /// Millimeter = 1/10 cm = 1/1000 m.
    Mm(f64),
}

/// A point, 1/72 inch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point(f64);

impl From<AbsLength> for Point {
    fn from(len: AbsLength) -> Self {
        // Computed with GNU units.
        Point(match len {
            AbsLength::Pt(l) => l,
            AbsLength::Pc(l) => l * 12.0,
            AbsLength::In(l) => l * 72.0,
            AbsLength::Cm(l) => l * 28.346_457,
            AbsLength::Mm(l) => l * 2.834_645_7,
        })
    }
}

/// A length, computed relatively to the current font, base font-size, viewport,
/// or elsewhere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelLength {
    /// Relative to font size.
    Em(f64),
    /// Width of the glyph `0`., or otherwise 0.5em.
    Ch(f64),
    /// Lowercase x-height.
    Ex(f64),
    /// Root font-size.
    Rem(f64),
    /// 1% the viewport' height.
    Vh(f64),
    /// 1% the viewport's width.
    Vw(f64),
    /// Smaller of Vw and Vh.
    Vmin(f64),
    /// Larger of Vw and Vh.
    Vmax(f64),
}
