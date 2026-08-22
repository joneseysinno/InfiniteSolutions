//! The `Glyphs` port.

use crate::core::Rect;

/// Text measurement and rasterization.
///
/// The one measurement the presenter cannot make itself, and the one it must not
/// invent. `hyper-ui` invents it — `char_w = font_size * 0.55`, with the in-source
/// admission *"approximate text extent until glyphon measures precisely"* — and every
/// layout built on that number is wrong by an amount nobody can predict, in a way no
/// test catches, because the test would have to know the right answer too.
///
/// A port instead. The fake returns a declared box, so layout tests assert on
/// arithmetic they control; the facade returns the truth.
pub trait Glyphs {
    /// The ink box of a run at a size, with its origin at the start of the baseline.
    ///
    /// A rectangle rather than a size, because ascent and descent are not symmetric
    /// and a caller that only gets a width and a height has to guess where the
    /// baseline was.
    fn measure(&self, run: &str, size: f64) -> Rect;
}
