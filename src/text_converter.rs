//! Generic text transformer shared by Office conversion entry points.

/// A caller-supplied text-to-text transformation.
///
/// `TextConverter` deliberately knows nothing about files, ZIP archives,
/// Office documents, EPUB packages, encodings, or command-line arguments. It
/// simply transforms one `&str` into an owned [`String`].
///
/// This makes the same converter reusable by plain-text conversion,
/// Office/EPUB processing, filename conversion, PDF extraction pipelines, and
/// other consumers.
///
/// # Custom converters
///
/// A converter does not have to use OpenCC:
///
/// ```rust
/// # use crate::text_converter::TextConverter;
/// let converter = TextConverter::new(|text: &str| text.replace("汉语", "漢語"));
///
/// assert_eq!(converter.convert("汉语"), "漢語");
/// ```
pub struct TextConverter<F> {
    convert: F,
}

impl<F> TextConverter<F>
where
    F: Fn(&str) -> String,
{
    /// Creates a text converter from a closure or function.
    #[inline]
    pub fn new(convert: F) -> Self {
        Self { convert }
    }

    /// Transforms the supplied text using the wrapped conversion policy.
    #[inline]
    pub fn convert(&self, text: &str) -> String {
        (self.convert)(text)
    }
}
