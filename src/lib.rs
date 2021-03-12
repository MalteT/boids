#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod boid;
mod boids;
mod debug;
mod model;
mod utils;

use model::Model;

const CANVAS_ID: &str = "canvas";

const QR_CODE_ID: &str = "qrcode";
const QR_CODE_LOCATION: &str = "qrcode.png";
const QR_CODE_SIZE: f64 = 30.0;

const DEFAULT_NR_OF_BOIDS: usize = 100;
const DEFAULT_MAX_SPEED: f64 = 300.0;
const DEFAULT_MAX_STEER: f64 = 30.0;
const DEFAULT_ALIGN_RADIUS: f64 = 50.0;
const DEFAULT_COHESION_RADIUS: f64 = 70.0;
const DEFAULT_SEPERATION_RADIUS: f64 = 15.0;
const DEFAULT_ANGST_RADIUS: f64 = 100.0;
const DEFAULT_ALIGN_FACTOR: f64 = 1.0 / 8.0;
const DEFAULT_COHESION_FACTOR: f64 = 1.0 / 100.0;
const DEFAULT_SEPERATION_FACTOR: f64 = 1.0;
const DEFAULT_ANGST_FACTOR: f64 = 2000.0;

const BG_COLOR: &str = "#d8dee9";
const BOID_COLOR: &str = "#bf616a";
const ALIGN_RADIUS_COLOR: &str = "red";
const COHESION_RADIUS_COLOR: &str = "green";
const SEPERATION_RADIUS_COLOR: &str = "blue";
const VELOCITY_COLOR: &str = "white";
const PREDATOR_RADIUS_COLOR: &str = "red";
const BORDER_COLOR: &str = "green";
const BUCKET_GRID_COLOR: &str = "#4c566a33";
const STATISTICS_COLOR: &str = "#666666";

const RETURN_STEER_VAL: f64 = 10.0;
const WALL_SIZE: f64 = 100.0;
const BUCKET_SIZE: usize = 50;

/// Start here!
#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Model>::new().mount_to_body();
}
