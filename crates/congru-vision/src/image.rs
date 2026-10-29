/// A grid of pixels, row after row from the top left.
#[derive(Debug, Clone, PartialEq)]
pub struct Image<Pixel> {
    width: usize,
    height: usize,
    pixels: Box<[Pixel]>,
}

/// Depth along the optical axis, in metres; NaN where nothing came back.
pub type DepthImage = Image<f64>;

impl<Pixel: Copy> Image<Pixel> {
    #[must_use]
    pub fn filled(width: usize, height: usize, value: Pixel) -> Self {
        Self {
            width,
            height,
            pixels: vec![value; width * height].into_boxed_slice(),
        }
    }

    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    #[must_use]
    pub fn get(&self, column: usize, row: usize) -> Pixel {
        self.pixels[row * self.width + column]
    }

    pub fn set(&mut self, column: usize, row: usize, value: Pixel) {
        self.pixels[row * self.width + column] = value;
    }

    #[must_use]
    pub fn pixels(&self) -> &[Pixel] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [Pixel] {
        &mut self.pixels
    }

    /// Every `(column, row)`, row after row.
    pub fn positions(&self) -> impl Iterator<Item = (usize, usize)> + use<Pixel> {
        let width = self.width;
        (0..self.height).flat_map(move |row| (0..width).map(move |column| (column, row)))
    }

    #[must_use]
    pub fn map<Other: Copy>(&self, convert: impl Fn(Pixel) -> Other) -> Image<Other> {
        Image {
            width: self.width,
            height: self.height,
            pixels: self.pixels.iter().map(|pixel| convert(*pixel)).collect(),
        }
    }
}
