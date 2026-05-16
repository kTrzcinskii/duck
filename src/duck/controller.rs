use glam::{Quat, Vec3};
use rand::RngExt;

pub struct BSpline {
    control_points: Vec<Vec3>,
    knots: Vec<f32>,
    t_start: f32,
    t_end: f32,
}

impl BSpline {
    const DEGREE: usize = 3;
    const NUM_POINTS: usize = 8;

    const MAX_COORD: f32 = 0.8;

    pub fn new_random(start: Vec3, start_tangent: Option<Vec3>) -> Self {
        let mut rng = rand::rng();

        let mut control_points: Vec<Vec3> = (0..Self::NUM_POINTS)
            .map(|_| {
                Vec3::new(
                    rng.random_range(-Self::MAX_COORD..Self::MAX_COORD),
                    0.0,
                    rng.random_range(-Self::MAX_COORD..Self::MAX_COORD),
                )
            })
            .collect();
        control_points[0] = start;

        if let Some(tangent) = start_tangent {
            control_points[1] = start + tangent * 0.3;
            control_points[1] = control_points[1].clamp(
                Vec3::new(-Self::MAX_COORD, 0.0, -Self::MAX_COORD),
                Vec3::new(Self::MAX_COORD, 0.0, Self::MAX_COORD),
            );
        }

        let mut knots = vec![0.0; Self::DEGREE + 1];
        let internal_knots = Self::NUM_POINTS - Self::DEGREE;
        for i in 1..internal_knots {
            knots.push(i as f32);
        }
        knots.extend(vec![internal_knots as f32; Self::DEGREE + 1]);

        let t_start = 0.0;
        let t_end = internal_knots as f32;

        Self {
            control_points,
            knots,
            t_start,
            t_end,
        }
    }

    pub fn evaluate(&self, t: f32) -> Vec3 {
        let t = t.clamp(self.t_start, self.t_end - 0.0001);
        let n = self.control_points.len();

        let mut k = Self::DEGREE;
        for i in Self::DEGREE..n {
            if self.knots[i] <= t && t < self.knots[i + 1] {
                k = i;
                break;
            }
        }

        let mut points: Vec<Vec3> = (0..=Self::DEGREE)
            .map(|j| self.control_points[k - Self::DEGREE + j])
            .collect();

        for r in 1..=Self::DEGREE {
            for j in (r..=Self::DEGREE).rev() {
                let i = k - Self::DEGREE + j;
                let denom = self.knots[i + Self::DEGREE - r + 1] - self.knots[i];
                let alpha = if denom.abs() < 1e-6 {
                    0.0
                } else {
                    (t - self.knots[i]) / denom
                };
                points[j] = points[j - 1].lerp(points[j], alpha);
            }
        }

        points[Self::DEGREE]
    }

    pub fn tangent(&self, t: f32) -> Vec3 {
        let eps = 0.001;
        let t = t.clamp(self.t_start, self.t_end - 0.0001);
        let t1 = (t + eps).min(self.t_end - 0.0001);
        let t0 = (t - eps).max(self.t_start);
        (self.evaluate(t1) - self.evaluate(t0)).normalize()
    }

    pub fn t_start(&self) -> f32 {
        self.t_start
    }

    pub fn t_end(&self) -> f32 {
        self.t_end
    }
}

pub struct DuckController {
    spline: BSpline,
    t: f32,
    speed: f32,
}

impl DuckController {
    pub fn update(&mut self, dt: f32) -> (Vec3, Quat) {
        self.t += self.speed * dt;

        let range = self.spline.t_end() - self.spline.t_start();

        if self.t >= range {
            self.t -= range;
            let end_pos = self.spline.evaluate(self.spline.t_end() - 0.0001);
            let end_tangent = self.spline.tangent(self.spline.t_end() - 0.0001);
            self.spline = BSpline::new_random(end_pos, Some(end_tangent));
        }

        let t_param = self.t_param();

        let position = self.spline.evaluate(t_param);
        let tangent = self.spline.tangent(t_param);
        let rotation = if tangent.length() > 0.001 {
            Quat::from_rotation_arc(Vec3::NEG_X, tangent)
        } else {
            Quat::IDENTITY
        };

        (position, rotation)
    }

    pub fn grid_position(&self) -> (u32, u32) {
        let t_param = self.t_param();
        let pos = self.spline.evaluate(t_param);
        let col = ((pos.x + 1.0) * 0.5 * 255.0) as u32;
        let row = ((pos.z + 1.0) * 0.5 * 255.0) as u32;
        (col.min(255), row.min(255))
    }

    fn t_param(&self) -> f32 {
        self.spline.t_start() + self.t
    }
}

impl Default for DuckController {
    fn default() -> Self {
        Self {
            spline: BSpline::new_random(Vec3::ZERO, None),
            t: 0.0,
            speed: 0.3,
        }
    }
}
