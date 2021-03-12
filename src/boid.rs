use lazy_static::lazy_static;
use na::{Rotation2, Unit, Vector2};
use nalgebra as na;
use rand::{prelude::*, thread_rng};
use yew::web_sys::{CanvasRenderingContext2d as Ctx, HtmlImageElement};

use std::f64;

use crate::{boids::Boids, Model, BUCKET_SIZE, DEFAULT_MAX_SPEED, QR_CODE_SIZE};

lazy_static! {
    pub static ref Y_AXIS: Unit<Vector2<f64>> = Vector2::y_axis();
    pub static ref TRIANGLE_POINTS: [Vector2<f64>; 3] = {
        // The length between the position and the tip of the boid
        let l_to_tip = Vector2::new(0.0, 8.0);
        // The length between the position and the bottom of the boid
        let l_to_bottom = Vector2::new(0.0, -3.0);
        // Half the length of the bottom itself
        let bottom_half = Vector2::new(3.0, 0.0);
        // The points of the triangle
        let tri_a = l_to_bottom - bottom_half;
        let tri_b = l_to_tip;
        let tri_c = l_to_bottom + bottom_half;
        [tri_a, tri_b, tri_c]
    };
}

pub struct Boid {
    pub pos: Vector2<f64>,
    pub vel: Vector2<f64>,
    /// Partition id this boid is contained in
    pub id: (usize, usize),
}

impl Boid {
    pub fn new(width: f64, height: f64) -> Self {
        let mut rng = thread_rng();
        // Use polar coordinates for the velocity generation
        let phi = rng.gen_range(0.0..(2.0 * f64::consts::PI));
        let vel = Vector2::new(phi.cos(), phi.sin());
        let pos = Vector2::new(rng.gen_range(0.0..width), rng.gen_range(0.0..height));
        let id = (pos.x as usize / BUCKET_SIZE, pos.y as usize / BUCKET_SIZE);
        Boid {
            vel: vel * DEFAULT_MAX_SPEED / 2.0,
            pos,
            id,
        }
    }

    pub fn update(curr_idx: usize, boids: &mut Boids, secs: f64) {
        let relevant: Vec<_> = boids.get_weighted_others(curr_idx).collect();
        let align_steer = boids.get_align_steer(&relevant);
        let cohesion_steer = boids.get_cohesion_steer(&relevant, curr_idx);
        let seperation_steer = boids.get_seperation_steer(&relevant, curr_idx);
        let angst_steer = boids.get_angst_steer(curr_idx);
        let return_steer = boids.get_return_steer(curr_idx);
        // Accumulate steer
        let mut steer: Vector2<f64> = na::zero();
        steer += boids.align_factor * align_steer;
        steer += boids.cohesion_factor * cohesion_steer;
        steer += boids.seperation_factor * seperation_steer;
        steer += boids.angst_factor * angst_steer;
        // Limit the steer (acceleration)
        steer = steer.cap_magnitude(boids.max_steer);
        // Add return steer to force them back into the center
        steer += return_steer;
        // Limit the steer (acceleration)
        steer = steer.cap_magnitude(boids.max_steer);
        // Apply steer and limit the velocity
        let vel = &mut boids.boids[curr_idx].vel;
        *vel += steer;
        *vel = vel.cap_magnitude(boids.max_speed);
        // Apply velocity
        let this = &mut boids.boids[curr_idx];
        this.pos += this.vel * secs;
        this.id = (
            this.pos.x as usize / BUCKET_SIZE,
            this.pos.y as usize / BUCKET_SIZE,
        );
    }
    /// Render the boid, this will not actually draw anything,
    /// but create the necessary lines.
    pub fn render(&self, model: &Model, ctx: &Ctx, qrcode: &HtmlImageElement) {
        let (xx, yy) = (self.pos.x, self.pos.y);
        if model.special_mode {
            ctx.draw_image_with_html_image_element_and_dw_and_dh(
                &qrcode,
                xx - QR_CODE_SIZE / 2.0,
                yy - QR_CODE_SIZE / 2.0,
                QR_CODE_SIZE,
                QR_CODE_SIZE,
            )
            .unwrap();
        } else {
            let rot = Rotation2::rotation_between(&Y_AXIS, &self.vel);
            let tri_a_rot = rot.transform_vector(&TRIANGLE_POINTS[0]) + self.pos;
            let tri_b_rot = rot.transform_vector(&TRIANGLE_POINTS[1]) + self.pos;
            let tri_c_rot = rot.transform_vector(&TRIANGLE_POINTS[2]) + self.pos;

            ctx.move_to(tri_a_rot.x, tri_a_rot.y);
            ctx.line_to(tri_b_rot.x, tri_b_rot.y);
            ctx.line_to(tri_c_rot.x, tri_c_rot.y);
            ctx.line_to(tri_a_rot.x, tri_a_rot.y);
        }
    }
}
