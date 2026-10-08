#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FileImage {
    pub extension: String,
    pub hash: i64,
    pub proccesed: bool,
    pub size_bytes: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FileImageAddErr {
    #[error("ffmpeg err {0}")]
    ReadingResolutionErr(String),

    #[error("invalid resolution {width}x{height}")]
    InvalidResolution { width: u32, height: u32 },

    #[error("io error {0}")]
    IoErr(String),

    #[error("stream error {0}")]
    StreamErr(String),

    #[error("file {image_name} is too big, max file size {max}, stopped upload at: {got}")]
    ImageTooBig {
        image_name: String,
        max: u32,
        got: u32,
    },

    #[error("file \"{0}\" must have extension in their name, such as .png")]
    ImageHasNoExtension(String),

    #[error("file extension {0} is not supported")]
    UnsupportedExtension(String),
}
