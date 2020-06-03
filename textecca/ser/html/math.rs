use super::super::SerializerError;

pub enum MathMode {
    Inline,
    Display,
}

pub fn render_tex(tex: &str, mode: MathMode) -> Result<String, SerializerError> {
    let opts = katex::OptsBuilder::default()
        .display_mode(match mode {
            MathMode::Inline => false,
            MathMode::Display => true,
        })
        .build()
        .unwrap();
    katex::render_with_opts(&tex, opts).map_err(|e| SerializerError::Other(Box::new(e)))
}
