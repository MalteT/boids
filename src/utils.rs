use log::warn;
use wasm_bindgen::{prelude::*, JsCast};
use yew::{
    utils::{document, window},
    web_sys::{HtmlImageElement, Performance, Url},
};

use std::f64;

use crate::{boid::Boid, boids::Boids, QR_CODE_ID};

pub fn update_boids_from_url(boids: &mut Boids) {
    let raw_url = document().url().expect("Failed to get URL");
    let url = Url::new(&raw_url).expect("Failed to create url from string");
    let params = url.search_params();
    macro_rules! parse {
        ($key:literal, $callback:expr) => {
            if let Some(val) = params.get($key) {
                if let Ok(val) = val.parse() {
                    $callback(val)
                } else {
                    warn!("Invalid value for key {} in url", stringify!($key));
                }
            }
        };
    }
    parse!("nr-of-boids", |number: usize| change_number_of_boids(
        boids, number
    ));
    parse!("align-radius", |radius: f64| boids.align_radius_squared =
        radius.powf(2.0));
    parse!("cohesion-radius", |radius: f64| boids
        .cohesion_radius_squared =
        radius.powf(2.0));
    parse!("seperation-radius", |radius: f64| boids
        .seperation_radius_squared =
        radius.powf(2.0));
    parse!("angst-radius", |radius: f64| boids.angst_radius_squared =
        radius.powf(2.0));
    parse!("max-speed", |speed: f64| boids.max_speed = speed);
    parse!("max-steer", |steer: f64| boids.max_steer = steer);
    parse!("align-factor", |fac: f64| boids.align_factor = fac);
    parse!("cohesion-factor", |fac: f64| boids.cohesion_factor = fac);
    parse!("seperation-factor", |fac: f64| boids.seperation_factor =
        fac);
    parse!("angst-factor", |fac: f64| boids.angst_factor = fac);
}

pub fn update_url(url_suffix: &str) {
    window()
        .history()
        .expect("History not found")
        .replace_state_with_url(&JsValue::NULL, "", Some(url_suffix))
        .expect("Replacing state failed");
}

pub fn performance() -> Performance {
    web_sys::window()
        .expect("Could not get window object")
        .performance()
        .expect("Could not get performance object")
}

pub fn update(boids: &mut Boids, time_passed: f64) {
    let secs = time_passed / 1000.0;
    // Iterate over all boid indices
    for idx in 0..boids.boids.len() {
        Boid::update(idx, boids, secs);
    }
}

pub fn change_number_of_boids(boids: &mut Boids, number: usize) {
    while number > boids.boids.len() {
        boids.boids.push(Boid::new(boids.size.0, boids.size.1));
    }
    while number < boids.boids.len() {
        boids.boids.pop();
    }
}

pub fn get_window_size() -> (f64, f64) {
    let window = web_sys::window().expect("Could not get window object");
    let width = window
        .inner_width()
        .expect("Could not get window width")
        .as_f64()
        .expect("Width is not a number");
    let height = window
        .inner_height()
        .expect("Could not get window height")
        .as_f64()
        .expect("Height is not a number");
    (width, height)
}

pub fn get_qrcode() -> HtmlImageElement {
    document()
        .get_element_by_id(QR_CODE_ID)
        .unwrap()
        .dyn_into()
        .unwrap()
}
