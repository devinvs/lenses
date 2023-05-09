use crate::geometry::Triangle;
use std::f32::consts::TAU;
use cgmath::Vector3;

use serde::{Serialize, Deserialize};

const MIN_LENS_WIDTH: f32 = 0.1;
const TESSEL: usize = 20;

#[derive(Serialize, Deserialize)]
pub struct Lens {
    pub radius: f32,
    pub left: LensSide,
    pub right: LensSide,
}

impl Lens {
    pub fn tesselate(&self) -> Vec<Triangle> {
        let mut triangles = vec![];

        let offset = match (&self.left, &self.right) {
            (LensSide::Flat, LensSide::Flat) => MIN_LENS_WIDTH/ 2.0,
            (LensSide::Convex(r), LensSide::Flat) | (LensSide::Flat, LensSide::Convex(r)) => (MIN_LENS_WIDTH - r).max(0.0) / 2.0,
            (LensSide::Convex(r1), LensSide::Convex(r2)) => (MIN_LENS_WIDTH - (r1+r2)).max(0.0) / 2.0,
            (LensSide::Concave(r1), LensSide::Concave(r2)) => (r1+r2+MIN_LENS_WIDTH) / 2.0,
            (LensSide::Concave(r), LensSide::Flat) | (LensSide::Flat, LensSide::Concave(r)) => (r+MIN_LENS_WIDTH) / 2.0,
            (LensSide::Convex(r1), LensSide::Concave(r2)) | (LensSide::Concave(r2), LensSide::Convex(r1)) => (MIN_LENS_WIDTH - (r2-r1)).max(0.0) / 2.0,
        };

        triangles.append(&mut self.left.tesselate(self.radius, offset, false));
        triangles.append(&mut self.right.tesselate(self.radius, offset,  true));

        if offset > 0.0 {
            triangles.append(&mut LensSide::tesselate_cylinder(TESSEL, self.radius, offset, false));
            triangles.append(&mut LensSide::tesselate_cylinder(TESSEL, self.radius, offset, true));
        }

        triangles
    }
}

#[derive(Serialize, Deserialize)]
pub enum LensSide {
    Flat,
    Concave(f32),
    Convex(f32)
}

impl LensSide {
    fn tesselate(&self, lens_radius: f32, offset: f32, flipped: bool) -> Vec<Triangle> {
        match self {
            Self::Flat => Self::tesselate_flat(TESSEL, lens_radius, offset, flipped),
            Self::Convex(h) => Self::tesselate_convex(TESSEL, lens_radius, *h, offset, flipped, false),
            Self::Concave(h) => Self::tesselate_convex(TESSEL, lens_radius, *h, -offset, !flipped, true),
        }
    }

    fn tesselate_convex(num: usize, r: f32, h: f32,  offset: f32, flipped: bool, concave: bool) -> Vec<Triangle> {
        let mut tris = vec![];

        let x = (r.powi(2) - h.powi(2)) / (2.0 * h);
        let convex_r = (x.powi(2) + r.powi(2)).sqrt();
        let offset = offset-x;

        let theta = (x/convex_r).acos();

        let start = -theta;
        let end = theta;
        let angle = end-start;
        let delta = angle / (num as f32);

        for zi in 0..num{
            let lng = start + zi as f32 * delta;
            let next_lng = lng + delta;

            for yi in 0..num {
                let lat = start + yi as f32 * delta;
                let next_lat = lat + delta;

                let mut v0 = Self::lng_lat_to_point(lng, lat, convex_r);
                let mut v1 = Self::lng_lat_to_point(lng, next_lat, convex_r);
                let mut v2 = Self::lng_lat_to_point(next_lng, lat, convex_r);
                let mut v3 = Self::lng_lat_to_point(next_lng, next_lat, convex_r);


                v0.x += offset;
                v1.x += offset;
                v2.x += offset;
                v3.x += offset;

                if flipped {
                    v0.x = -v0.x;
                    v1.x = -v1.x;
                    v2.x = -v2.x;
                    v3.x = -v3.x;
                }

                if (flipped && !concave) || (!flipped && concave) {
                    tris.push(Triangle::new(v2, v1, v0));
                    tris.push(Triangle::new(v1, v2, v3));
                } else {
                    tris.push(Triangle::new(v0, v1, v2));
                    tris.push(Triangle::new(v3, v2, v1));
                }
            }
        }

        let v_in_circ = |v: Vector3<f32>| -> bool {
            v.z.powi(2) + v.y.powi(2) <= r.powi(2)
        };

        let v_to_edge = |v: &mut Vector3<f32>| {
            let theta = v.y.atan2(v.z);
            v.z = r * theta.cos();
            v.y = r * theta.sin();
            v.x = convex_r * start.cos() * (start + angle /2.0).cos() + offset;
            if flipped {
                v.x = -v.x;
            }
        };

        // Filter out tris completely out of the cirlce
        //
        // Then take tris that are partially in the circle
        // and move their edges to the nearest points on
        // the edge of the circle (for z, y)
        let tris = tris.into_iter()
            .filter(|t| {
                v_in_circ(t.v0)
                    || v_in_circ(t.v1)
                    || v_in_circ(t.v2)
            })
            .map(|mut t| {
                if !v_in_circ(t.v0) {
                    v_to_edge(&mut t.v0);
                }

                if !v_in_circ(t.v1) {
                    v_to_edge(&mut t.v1);
                }

                if !v_in_circ(t.v2) {
                    v_to_edge(&mut t.v2);
                }

                t
            }).collect::<Vec<_>>();


        tris
    }

    fn tesselate_flat(num: usize, radius: f32, offset: f32, flipped: bool) -> Vec<Triangle> {
        let mut tris = vec![];

        let offset = if flipped {-offset} else {offset};

        // Create circle by creating triangle for every pizza slize
        let angle = TAU / num as f32;
        for i in 0..num {
            let theta = i as f32 * angle;

            let v0 = Vector3::new(offset, 0.0, 0.0);
            let v1 = Vector3::new(offset, radius * theta.sin(), radius * theta.cos());
            let v2 = Vector3::new(offset, radius * (theta+angle).sin(), radius * (theta+angle).cos());

            if flipped {
                tris.push(Triangle::new(v2, v1, v0))
            } else {
                tris.push(Triangle::new(v0, v1, v2))
            }
        }



        tris
    }

    fn tesselate_cylinder(num: usize, radius: f32, offset: f32, flipped: bool) -> Vec<Triangle> {
        let mut tris = vec![];

        // Build cylinder for the offset
        let angle = TAU / num as f32;
        for i in 0..num {
            let theta = i as f32 * angle;

            let x0 = if flipped {-offset} else {offset};
            let x1 = 0.0;

            let y0 = radius * theta.sin();
            let y1 = radius * (theta+angle).sin();

            let z0 = radius * theta.cos();
            let z1 = radius * (theta+angle).cos();

            if flipped {
                tris.push(Triangle::new(
                    Vector3::new(x0, y0, z0),
                    Vector3::new(x0, y1, z1),
                    Vector3::new(x1, y1, z1),
                ));
                tris.push(Triangle::new(
                    Vector3::new(x0, y0, z0),
                    Vector3::new(x1, y1, z1),
                    Vector3::new(x1, y0, z0),
                ));

            } else {
                tris.push(Triangle::new(
                    Vector3::new(x1, y1, z1),
                    Vector3::new(x0, y1, z1),
                    Vector3::new(x0, y0, z0),
                ));
                tris.push(Triangle::new(
                    Vector3::new(x1, y0, z0),
                    Vector3::new(x1, y1, z1),
                    Vector3::new(x0, y0, z0),
                ));
            }
        }


        tris
    }

    fn lng_lat_to_point(lng: f32, lat: f32, radius: f32) -> Vector3<f32> {
        let x = radius * lat.cos() * lng.cos();
        let y = radius * lat.cos() * lng.sin();
        let z = radius * lat.sin();

        Vector3::new(x, y, z)
    }
}
