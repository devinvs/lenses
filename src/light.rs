use cgmath::Vector3;
use crate::geometry::Ray;

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum Light {
    Laser([f32; 3], [f32; 3]),
    Point([f32; 3]),
}

impl Light {
    pub fn spawn_from(&self, p: Vector3<f32>) -> Ray {
        match self {
            Self::Point(_) => {
                let mut x = -1.0 + 2.0 * rand::random::<f32>();
                let mut y = -1.0 + 2.0 * rand::random::<f32>();
                let mut z = -1.0 + 2.0 * rand::random::<f32>();
                loop {
                    if x.powi(2) + y.powi(2) + z.powi(2) < 1.0 {break;}

                    x = -1.0 + 2.0 * rand::random::<f32>();
                    y = -1.0 + 2.0 * rand::random::<f32>();
                    z = -1.0 + 2.0 * rand::random::<f32>();
                }

                let dir = Vector3::new(x, y, z);
                Ray::new(p, dir)
            }
            Self::Laser(_, dir) => {
                Ray::new(p, (*dir).into())
            }
        }
    }

    pub fn spawn(&self) -> Ray {
        let p = match self {
            Self::Point(p) => p,
            Self::Laser(p, _) => p,
        };

        self.spawn_from((*p).into())
    }
}
