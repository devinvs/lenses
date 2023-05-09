use cgmath::Vector3 as Vec3;
use cgmath::dot;
use cgmath::InnerSpace;
use cgmath::Vector3;

use crate::Vertex;
use crate::Normal;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub v0: Vec3<f32>,
    pub v1: Vec3<f32>,
    pub v2: Vec3<f32>,
}

impl Triangle {
    pub fn new(v0: Vec3<f32>, v1: Vec3<f32>, v2: Vec3<f32>) -> Triangle {
        Triangle { v0, v1, v2 }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<f32> {
        let v1v0 = self.v1 - self.v0;
        let v2v0 = self.v2 - self.v0;
        let rov0 = ray.origin - self.v0;
        let n = v1v0.cross(v2v0);
        let q = rov0.cross(ray.dir);
        let ang = dot(ray.dir, n);
        if ang.abs() == 0.0 {
            return None;
        }

        let d = 1.0 / ang;
        let u = d * dot(-q, v2v0);
        let v = d * dot(q, v1v0);
        let t = d * dot(-n, rov0);

        if u<0.0 || v<0.0 || (u+v)>1.0 || t<0.0 {
            None
        } else {
            Some(t)
        }
    }

    pub fn normals(&self) -> [Normal; 3] {
        let v1 = self.v1-self.v0;
        let v2 = self.v2-self.v0;

        let n = -v1.cross(v2).normalize();
        let n = Normal {
            normal: n.into()
        };

        [n, n, n]
    }

    pub fn vertices(&self) -> [Vertex; 3] {
        [Vertex {
            position: self.v0.into()
        }, Vertex {
            position: self.v1.into()
        }, Vertex {
            position: self.v2.into()
        }]
    }

    pub fn indices(&self, start: u32) -> [u32; 3] {
        [start, start+1, start+2]
    }

    pub fn left_of(&self, axis: Axis, v: f32) -> bool {
        match axis {
            Axis::X => self.v0.x <= v || self.v1.x <= v || self.v2.x <= v,
            Axis::Y => self.v0.y <= v || self.v1.y <= v || self.v2.y <= v,
            Axis::Z => self.v0.z <= v || self.v1.z <= v || self.v2.z <= v,
        }
    }

    pub fn right_of(&self, axis: Axis, v: f32) -> bool {
        match axis {
            Axis::X => self.v0.x >= v || self.v1.x >= v || self.v2.x >= v,
            Axis::Y => self.v0.y >= v || self.v1.y >= v || self.v2.y >= v,
            Axis::Z => self.v0.z >= v || self.v1.z >= v || self.v2.z >= v,
        }
    }

    pub fn fit(&self) -> AABB {
        let v0 = self.v0;
        let v1 = self.v1;
        let v2 = self.v2;

        let minx = v0.x.min(v1.x).min(v2.x);
        let miny = v0.y.min(v1.y).min(v2.y);
        let minz = v0.z.min(v1.z).min(v2.z);

        let maxx = v0.x.max(v1.x).max(v2.x);
        let maxy = v0.y.max(v1.y).max(v2.y);
        let maxz = v0.z.max(v1.z).max(v2.z);

        AABB {
            min: Vec3::new(minx, miny, minz),
            max: Vec3::new(maxx, maxy, maxz)
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Vec3<f32>,
    pub dir: Vec3<f32>,
    pub inside: bool
}


impl Ray {
    pub fn new(origin: Vec3<f32>, dir: Vec3<f32>) -> Self {
        Self { origin, dir: dir.normalize(), inside: false }
    }

    pub fn inside(origin: Vec3<f32>, dir: Vec3<f32>) -> Self {
        Self { origin, dir: dir.normalize(), inside: true }
    }

    pub fn from_points(a: Vec3<f32>, b: Vec3<f32>) -> Self {
        Self {
            origin: a,
            dir: (b-a).normalize(),
            inside: false
        }
    }

    pub fn tesselate(&self, d: f32) -> Vec<Triangle> {
        let f = self.origin;
        let t = self.origin + self.dir * d;

        let w = 0.005;

        let a = Vector3::new(f.x, f.y+w, f.z+w);
        let b = Vector3::new(f.x, f.y+w, f.z-w);
        let c = Vector3::new(f.x, f.y-w, f.z+w);
        let d = Vector3::new(f.x, f.y-w, f.z-w);

        let e = Vector3::new(t.x, t.y+w, t.z+w);
        let f = Vector3::new(t.x, t.y+w, t.z-w);
        let g = Vector3::new(t.x, t.y-w, t.z+w);
        let h = Vector3::new(t.x, t.y-w, t.z-w);

        vec![
            // Left Side
            Triangle {
                v2: a,
                v1: b,
                v0: c
            },
            Triangle {
                v2: c,
                v1: b,
                v0: d
            },
            // Right side
            Triangle {
                v0: e,
                v1: f,
                v2: g
            },
            Triangle {
                v0: g,
                v1: f,
                v2: h
            },
            // Top
            Triangle {
                v2: a,
                v1: e,
                v0: f
            },
            Triangle {
                v2: a,
                v1: f,
                v0: b
            },
            // Bottom
            Triangle {
                v0: c,
                v1: g,
                v2: h
            },
            Triangle {
                v0: c,
                v1: h,
                v2: d
            },
            // Left
            Triangle {
                v0: c,
                v1: e,
                v2: g
            },
            Triangle {
                v0: c,
                v1: a,
                v2: e
            },
            // Right
            Triangle {
                v2: d,
                v1: f,
                v0: h
            },
            Triangle {
                v2: d,
                v1: b,
                v0: f
            },
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Axis { X, Y, Z }

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3<f32>,
    pub max: Vec3<f32>
}

impl AABB {
    pub fn union(self, other: Self) -> Self {
        Self {
            min: Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z)
            ),
            max: Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z)
            )
        }
    }

    pub fn split(self, axis: Axis) -> (Self, Self, f32) {
        match axis {
            Axis::X => {
                let mid = (self.min.x + self.max.x) / 2.0;

                let l = AABB {
                    min: self.min,
                    max: Vec3::new(mid, self.max.y, self.max.z)
                };
                let r = AABB {
                    min: Vec3::new(mid, self.min.y, self.min.z),
                    max: self.max
                };

                (l, r, mid)
            }
            Axis::Y => {
                let mid = (self.min.y + self.max.y) / 2.0;

                let l = AABB {
                    min: self.min,
                    max: Vec3::new(self.max.x, mid, self.max.z)
                };
                let r = AABB {
                    min: Vec3::new(self.min.x, mid, self.min.z),
                    max: self.max
                };

                (l, r, mid)
            }
            Axis::Z => {
                let mid = (self.min.z + self.max.z) / 2.0;

                let l = AABB {
                    min: self.min,
                    max: Vec3::new(self.max.x, self.max.y, mid)
                };
                let r = AABB {
                    min: Vec3::new(self.min.x, self.min.y, mid),
                    max: self.max
                };

                (l, r, mid)
            }
        }
    }

    pub fn intersect(&self, ray: &Ray) -> bool {
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;


        if ray.dir.x == 0.0 {
            if ray.origin.x < self.min.x || ray.origin.x > self.max.x {
                return false;
            }
        } else {
            let t1 = (self.min.x - ray.origin.x) / ray.dir.x;
            let t2 = (self.max.x - ray.origin.x) / ray.dir.x;
            let (t1, t2) = if t1 > t2 { (t2, t1) } else { (t1, t2) };
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return false;
            }
        }

        if ray.dir.y == 0.0 {
            if ray.origin.y < self.min.y || ray.origin.y > self.max.y {
                return false;
            }
        } else {
            let t1 = (self.min.y - ray.origin.y) / ray.dir.y;
            let t2 = (self.max.y - ray.origin.y) / ray.dir.y;
            let (t1, t2) = if t1 > t2 { (t2, t1) } else { (t1, t2) };
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return false;
            }
        }

        if ray.dir.z == 0.0 {
            if ray.origin.z < self.min.z || ray.origin.z > self.max.z {
                return false;
            }
        } else {
            let t1 = (self.min.z - ray.origin.z) / ray.dir.z;
            let t2 = (self.max.z - ray.origin.z) / ray.dir.z;
            let (t1, t2) = if t1 > t2 { (t2, t1) } else { (t1, t2) };
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return false;
            }
        }

        true
    }
}
