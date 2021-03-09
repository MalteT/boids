use wasm_bindgen::prelude::*;
use yew::web_sys::CanvasRenderingContext2d;

use std::f64;

use crate::{
    boid::Boid,
    boids::{Boids, BUCKET_SIZE, WALL_SIZE},
    Model,
};

const ALIGN_RADIUS_COLOR: &str = "red";
const COHESION_RADIUS_COLOR: &str = "green";
const SEPERATION_RADIUS_COLOR: &str = "blue";
const VELOCITY_COLOR: &str = "white";
const PREDATOR_RADIUS_COLOR: &str = "red";
const BORDER_COLOR: &str = "green";
const BUCKET_GRID_COLOR: &str = "#333333";
const STATISTICS_COLOR: &str = "#666666";

type Ctx = CanvasRenderingContext2d;

pub fn render_debug_info(ctx: &Ctx, model: &Model) {
    let boids = &model.boids;
    if let Some(first) = &boids.boids.first() {
        draw_align_radius(ctx, boids, first);
        draw_cohesion_radius(ctx, boids, first);
        draw_seperation_radius(ctx, boids, first);
        draw_velocity(ctx, first);
    }
    draw_predator_radius(ctx, boids);
    draw_border(ctx, boids);
    draw_bucket_grid(ctx, boids);
    draw_statistics(ctx, model);
}

fn draw_align_radius(ctx: &Ctx, boids: &Boids, first: &Boid) {
    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from_str(ALIGN_RADIUS_COLOR));
    let align_radius = boids.align_radius_squared.sqrt();
    ctx.move_to(first.pos.x + align_radius, first.pos.y);
    ctx.arc(
        first.pos.x,
        first.pos.y,
        align_radius,
        0.0,
        2.0 * f64::consts::PI,
    )
    .expect("Failed to draw boid align radius");
    ctx.stroke();
}

fn draw_cohesion_radius(ctx: &Ctx, boids: &Boids, first: &Boid) {
    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from_str(COHESION_RADIUS_COLOR));
    let cohesion_radius = boids.cohesion_radius_squared.sqrt();
    ctx.move_to(first.pos.x + cohesion_radius, first.pos.y);
    ctx.arc(
        first.pos.x,
        first.pos.y,
        cohesion_radius,
        0.0,
        2.0 * f64::consts::PI,
    )
    .expect("Failed to draw boid cohesion radius");
    ctx.stroke();
}

fn draw_seperation_radius(ctx: &Ctx, boids: &Boids, first: &Boid) {
    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from_str(SEPERATION_RADIUS_COLOR));
    let seperation_radius = boids.seperation_radius_squared.sqrt();
    ctx.move_to(first.pos.x + seperation_radius, first.pos.y);
    ctx.arc(
        first.pos.x,
        first.pos.y,
        seperation_radius,
        0.0,
        2.0 * f64::consts::PI,
    )
    .expect("Failed to draw boid seperation radius");
    ctx.stroke();
}

fn draw_velocity(ctx: &Ctx, first: &Boid) {
    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from_str(VELOCITY_COLOR));
    ctx.move_to(first.pos.x, first.pos.y);
    ctx.line_to(first.pos.x + first.vel.x, first.pos.y + first.vel.y);
    ctx.stroke();
}

fn draw_predator_radius(ctx: &Ctx, boids: &Boids) {
    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from_str(PREDATOR_RADIUS_COLOR));
    let angst_radius = boids.angst_radius_squared.sqrt();
    ctx.arc(
        boids.predator.x,
        boids.predator.y,
        angst_radius,
        0.0,
        2.0 * f64::consts::PI,
    )
    .expect("Failed to draw boid seperation radius");
    ctx.stroke();
}

fn draw_border(ctx: &Ctx, boids: &Boids) {
    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from_str(BORDER_COLOR));
    ctx.move_to(WALL_SIZE, WALL_SIZE);
    ctx.line_to(boids.size.0 - WALL_SIZE, WALL_SIZE);
    ctx.line_to(boids.size.0 - WALL_SIZE, boids.size.1 - WALL_SIZE);
    ctx.line_to(WALL_SIZE, boids.size.1 - WALL_SIZE);
    ctx.line_to(WALL_SIZE, WALL_SIZE);
    ctx.stroke();
}

fn draw_bucket_grid(ctx: &Ctx, boids: &Boids) {
    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from_str(BUCKET_GRID_COLOR));
    for x in 0..=(boids.size.0 as usize / BUCKET_SIZE) {
        ctx.move_to((x * BUCKET_SIZE) as f64, 0.0);
        ctx.line_to((x * BUCKET_SIZE) as f64, boids.size.1);
    }
    for y in 0..=(boids.size.1 as usize / BUCKET_SIZE) {
        ctx.move_to(0.0, (y * BUCKET_SIZE) as f64);
        ctx.line_to(boids.size.0, (y * BUCKET_SIZE) as f64);
    }
    ctx.stroke();
}

fn draw_statistics(ctx: &Ctx, model: &Model) {
    ctx.begin_path();
    ctx.set_fill_style(&JsValue::from_str(STATISTICS_COLOR));
    let fps = 1000.0 / model.last_time_passed;
    let text = format!("{:.2}", fps);
    ctx.fill_text(&text, 10.0, model.boids.size.1 - 30.0).expect("Failed to draw fps");
    ctx.stroke();

}
