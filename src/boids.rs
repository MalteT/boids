use na::{Point2, Vector2};
use nalgebra as na;
use rand::{prelude::*, thread_rng};

use std::f64;

pub const DEFAULT_NR_OF_BOIDS: usize = 100;
pub const DEFAULT_MAX_SPEED: f64 = 300.0;
pub const DEFAULT_MAX_STEER: f64 = 30.0;
pub const DEFAULT_ALIGN_RADIUS: f64 = 50.0;
pub const DEFAULT_COHESION_RADIUS: f64 = 70.0;
pub const DEFAULT_SEPERATION_RADIUS: f64 = 15.0;
pub const DEFAULT_ANGST_RADIUS: f64 = 100.0;
pub const DEFAULT_ALIGN_FACTOR: f64 = 1.0 / 8.0;
pub const DEFAULT_CENTER_FACTOR: f64 = 1.0 / 50.0;
pub const DEFAULT_SEPERATION_FACTOR: f64 = 1.0;
pub const DEFAULT_ANGST_FACTOR: f64 = 2000.0;
pub const RETURN_STEER_VAL: f64 = 5.0;

pub struct Boids {
    pub boids: Vec<Boid>,
    pub size: (f64, f64),
    pub align_radius_squared: f64,
    pub cohesion_radius_squared: f64,
    pub seperation_radius_squared: f64,
    pub angst_radius_squared: f64,
    pub debug_mode: bool,
    pub predator: Vector2<f64>,
    pub max_speed: f64,
    pub max_steer: f64,
    pub align_factor: f64,
    pub cohesion_factor: f64,
    pub seperation_factor: f64,
    pub angst_factor: f64,
}

impl Boids {
    pub fn new(width: f64, height: f64) -> Self {
        let boids = (0..DEFAULT_NR_OF_BOIDS)
            .map(|_| Boid::new(width, height))
            .collect();
        let size = (width, height);
        let align_radius_squared = DEFAULT_ALIGN_RADIUS.powf(2.0);
        let cohesion_radius_squared = DEFAULT_COHESION_RADIUS.powf(2.0);
        let seperation_radius_squared = DEFAULT_SEPERATION_RADIUS.powf(2.0);
        let angst_radius_squared = DEFAULT_ANGST_RADIUS.powf(2.0);
        let debug_mode = false;
        let predator = Vector2::new(width / 2.0, height / 2.0);
        let max_speed = DEFAULT_MAX_SPEED;
        let max_steer = DEFAULT_MAX_STEER;
        let align_factor = DEFAULT_ALIGN_FACTOR;
        let center_factor = DEFAULT_CENTER_FACTOR;
        let seperation_factor = DEFAULT_SEPERATION_FACTOR;
        let angst_factor = DEFAULT_ANGST_FACTOR;
        Boids {
            boids,
            size,
            align_radius_squared,
            cohesion_radius_squared,
            seperation_radius_squared,
            angst_radius_squared,
            predator,
            debug_mode,
            max_speed,
            max_steer,
            align_factor,
            cohesion_factor: center_factor,
            seperation_factor,
            angst_factor,
        }
    }
    pub fn scatter(&mut self) {
        for boid in &mut self.boids {
            *boid = Boid::new(self.size.0, self.size.1)
        }
    }
}

pub struct Boid {
    pub pos: Vector2<f64>,
    pub vel: Vector2<f64>,
}

impl Boid {
    pub fn new(width: f64, height: f64) -> Self {
        let mut rng = thread_rng();
        // Use polar coordinates for the velocity generation
        let phi = rng.gen_range(0.0..(2.0 * f64::consts::PI));
        let vel = Vector2::new(phi.cos(), phi.sin());
        Boid {
            pos: Vector2::new(rng.gen_range(0.0..width), rng.gen_range(0.0..height)),
            vel: vel * DEFAULT_MAX_SPEED / 2.0,
        }
    }

    pub fn update(curr_idx: usize, boids: &mut Boids, secs: f64) {
        let relevant: Vec<_> = get_weighted_others(curr_idx, boids).collect();
        let align_steer = get_align_steer(&relevant, boids);
        let cohesion_steer = get_cohesion_steer(&relevant, curr_idx, boids);
        let seperation_steer = get_seperation_steer(&relevant, curr_idx, boids);
        let angst_steer = get_angst_steer(curr_idx, boids);
        let return_steer = get_return_steer(curr_idx, boids);
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
    }
}

/// Get an iterator of relevant neighbours and their distance to the current boid.
/// A boid is not relevant if the distance to it is larger than the largest
/// distance considered.
fn get_weighted_others<'a>(
    curr_idx: usize,
    boids: &'a Boids,
) -> impl Iterator<Item = (&'a Boid, f64)> + 'a {
    let max_radius_squared = boids.align_radius_squared.max(boids.cohesion_radius_squared).max(boids.seperation_radius_squared);
    // The current element and it's position
    let this = &boids.boids[curr_idx];
    let this_pos = Point2::new(this.pos.x, this.pos.y);
    // Filter the rest and map add the distance to them
    boids
        .boids
        .iter()
        .enumerate()
        .filter(move |(idx, _)| *idx != curr_idx)
        .filter_map(move |(_, other)| {
            let pos = Point2::new(other.pos.x, other.pos.y);
            let dist_squared = na::distance_squared(&this_pos, &pos);
            if dist_squared < max_radius_squared {
                Some((other, dist_squared))
            } else {
                None
            }
        })
}

fn get_return_steer(curr_idx: usize, boids: &Boids) -> Vector2<f64> {
    let size = boids.size;
    let curr = &boids.boids[curr_idx];
    let mut steer: Vector2<_> = na::zero();
    let wall = 100.0;
    if curr.pos.x < 0.0 + wall {
        steer.x += RETURN_STEER_VAL;
    } else if curr.pos.x > size.0 - wall {
        steer.x -= RETURN_STEER_VAL;
    }
    if curr.pos.y < 0.0 + wall {
        steer.y += RETURN_STEER_VAL;
    } else if curr.pos.y > size.1 - wall {
        steer.y -= RETURN_STEER_VAL;
    }
    steer
}

fn get_angst_steer(curr_idx: usize, boids: &Boids) -> Vector2<f64> {
    let this_pos = Point2::origin() + boids.boids[curr_idx].pos;
    let pred_pos = Point2::origin() + boids.predator;
    let dist_sq = na::distance_squared(&this_pos, &pred_pos);
    if dist_sq > boids.angst_radius_squared {
        na::zero()
    } else if (dist_sq).abs() > f64::EPSILON {
        (boids.boids[curr_idx].pos - boids.predator) / dist_sq
    } else {
        boids.boids[curr_idx].pos - boids.predator
    }
}

fn get_cohesion_steer(relevant: &Vec<(&Boid, f64)>, curr_idx: usize, boids: &Boids) -> Vector2<f64> {
    let (sum, count): (Vector2<f64>, usize) = relevant.iter()
        .filter(|(_, dist)| *dist <= boids.cohesion_radius_squared)
        .map(|(boid, _)| boid.pos)
        .fold((na::zero(), 0), |(sum, count), pos| (sum + &pos, count + 1));
    let target_pos = if count != 0 { sum / count as f64 } else { sum };
    target_pos - boids.boids[curr_idx].pos
}

fn get_seperation_steer(relevant: &Vec<(&Boid, f64)>, curr_idx: usize, boids: &Boids) -> Vector2<f64> {
    relevant.iter()
        .filter(|(_, dist)| *dist <= boids.seperation_radius_squared)
        .map(|(boid, _)| boid.pos - boids.boids[curr_idx].pos)
        .fold(na::zero(), |sum: Vector2<f64>, el| sum - el)
}

fn get_align_steer(relevant: &Vec<(&Boid, f64)>, boids: &Boids) -> Vector2<f64> {
    // We're not our friend, no filtering necessary
    let (sum, count) = relevant.iter()
        .filter(|(_, dist)| *dist <= boids.align_radius_squared)
        .map(|(boid, _)| boid.vel)
        .fold((na::zero(), 0), |(sum, count), vel| (sum + vel, count + 1));
    if count != 0 {
        sum / count as f64
    } else {
        sum
    }
}
