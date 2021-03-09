use na::Vector2;
use nalgebra as na;
use rand::{prelude::*, thread_rng};

use std::f64;

use crate::boids::{Boids, BUCKET_SIZE, DEFAULT_MAX_SPEED};

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
        this.id = (this.pos.x as usize / BUCKET_SIZE, this.pos.y as usize / BUCKET_SIZE);
    }
}
