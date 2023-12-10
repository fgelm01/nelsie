mod common;
mod model;
mod render;

use crate::common::fileutils::ensure_directory;
use crate::model::{Slide, SlideDeck};
use crate::render::{
    check_fonts, load_image_in_deck, render_slide_step, GlobalResources, PdfBuilder, RenderConfig,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use thiserror::Error;
use usvg::fontdb;

#[derive(Debug, Error)]
pub enum NelsieError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    XmlError(#[from] roxmltree::Error),
    #[error(transparent)]
    SvgError(#[from] usvg::Error),
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Error: {0}")]
    GenericError(String),
}

pub struct OutputConfig<'a> {
    pub output_pdf: Option<&'a Path>,
    pub output_svg: Option<&'a Path>,
    pub output_png: Option<&'a Path>,
}

pub type Result<T> = std::result::Result<T, NelsieError>;

impl From<serde_json::error::Error> for NelsieError {
    fn from(e: serde_json::error::Error) -> Self {
        Self::DeserializationError(e.to_string())
    }
}

// impl From<roxmltree::Error> for NelsieError {
//     fn from(e: roxmltree::Error) -> Self {
//         Self::XmlError(e)
//     }
// }

fn parse_slide_deck(data: &str) -> Result<SlideDeck> {
    serde_json::from_str(data).map_err(|e| e.into())
}