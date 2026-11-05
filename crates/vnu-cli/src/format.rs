#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum OutputFormat {
    #[default]
    Gnu,
    Text,
    Json,
    Xml,
}
