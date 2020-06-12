#[derive(thiserror::Error, Debug)]
pub enum ClmanError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Yaml Error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Git Error: {0}")]
    Git(#[from] git2::Error),
    #[error("Command Error: {stderr:?}")]
    Command { stderr: String },
    #[error("GPU Error: {0}")]
    Gpu(ocl::Error),
}
pub type ClmanResult<T> = std::result::Result<T, ClmanError>;

impl From<ocl::Error> for ClmanError {
    fn from(err: ocl::Error) -> Self {
        Self::Gpu(err)
    }
}
