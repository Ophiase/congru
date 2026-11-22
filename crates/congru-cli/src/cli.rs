use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::methods::Method;

#[derive(Debug, Parser)]
#[command(
    name = "congru",
    version,
    about = "Rigid registration of point clouds and meshes. Every answer is JSON on stdout; \
             a failure is {\"error\": \"...\"} with exit code 1."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        about = "Align a source onto a target: the transform takes the source onto the target"
    )]
    Align(Align),
    #[command(about = "Follow a model through a folder of frames, one JSON line per frame")]
    Track(Track),
    #[command(about = "Count the points of a file, its bounds and its size")]
    Describe(Describe),
    #[command(about = "List the alignment methods, with what each needs")]
    Methods,
}

#[derive(Debug, Args)]
pub struct Align {
    #[arg(help = "The scan to move: .xyz or .ply")]
    pub source: PathBuf,
    #[arg(help = "What it is moved onto: .xyz, or .ply as a cloud or a mesh")]
    pub target: PathBuf,
    #[arg(long, value_enum, default_value_t = Method::Guarded)]
    pub method: Method,
    #[arg(
        long,
        help = "A first transform of the source onto the target: 16 numbers, row-major, \
                or a file holding them"
    )]
    pub guess: Option<String>,
    #[arg(
        long,
        help = "Thin both clouds on a grid of this spacing first, in the unit of the points"
    )]
    pub voxel: Option<f64>,
}

#[derive(Debug, Args)]
pub struct Track {
    #[arg(help = "A folder of .xyz or .ply frames, read in the order of their names")]
    pub frames: PathBuf,
    #[arg(
        long,
        help = "The object to follow: .xyz, or .ply as a cloud or a mesh"
    )]
    pub model: PathBuf,
    #[arg(
        long,
        help = "The object's size, which sets every scale; the model's bounding diagonal by default"
    )]
    pub span: Option<f64>,
}

#[derive(Debug, Args)]
pub struct Describe {
    #[arg(help = "A .xyz or .ply file")]
    pub file: PathBuf,
}
