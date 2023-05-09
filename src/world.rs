use crate::vulkan::{VertexBuffer, IndexBuffer, NormalBuffer, Uniform};
use crate::{Vertex, Normal};
use crate::vulkan::VulkanState;
use crate::geometry::Triangle;
use crate::geometry::Ray;

use cgmath::prelude::*;

use crate::vulkan::Model;
use crate::kdtree::KDNode;
use crate::kdtree::build_kdtree;
use crate::light::Light;

use std::f32::consts::PI;

use cgmath::{Point3, Vector4, Matrix4, Rad, Vector3, Matrix3};
use cgmath::dot;

#[derive(Debug, PartialEq)]
pub enum Material {
    Solid,
    Mirror,
    Glass(f32),
}

pub struct World {
    // Per entity
    pub models: Vec<Model>,
    pub colors: Vec<Vector4<f32>>,
    pub positions: Vec<Vector3<f32>>,
    pub scales: Vec<Vector3<f32>>,
    pub materials: Vec<Material>,

    // Ray rendering
    pub lines: Vec<Model>,

    // per model
    pub model_idx: Vec<Model>,
    pub model_data: Vec<Triangle>,

    // global
    pub vertex_buffer: Option<VertexBuffer>,
    pub index_buffer: Option<IndexBuffer>,
    pub normal_buffer: Option<NormalBuffer>,

    pub lights: Vec<Light>,

    pub kdtree: Option<KDNode>,

    pub fov: f32,
    pub rotx: f32,
    pub roty: f32,
    pub rotz: f32
}

impl World {
    pub fn new() -> Self {
        World {
            models: vec![],
            colors: vec![],
            positions: vec![],
            scales: vec![],
            materials: vec![],
            
            lines: vec![],

            model_data: vec![],
            model_idx: vec![],
            vertex_buffer: None,
            index_buffer: None,
            normal_buffer: None,

            lights: vec![],
            kdtree: None,

            fov: std::f32::consts::FRAC_PI_2,
            rotx: 0.0,
            roty: 0.0,
            rotz: 0.0,
        }
    }

    pub fn add_entity(
        &mut self,
        model: Model,
        position: Vector3<f32>,
        material: Material,
        scale: Vector3<f32>,
        color: Vector4<f32>,
    ) -> usize {
        self.models.push(model);
        self.colors.push(color);
        self.positions.push(position);
        self.scales.push(scale);
        self.materials.push(material);

        self.models.len() - 1
    }

    pub fn add_model(
        &mut self,
        mut triangles: Vec<Triangle>,
    ) -> Model {
        let start = self.model_data.len();
        let count = triangles.len();
        self.model_data.append(&mut triangles);

        let m = Model {
            count: count as u32,
            index: start as u32
        };
        self.model_idx.push(m.clone());
        m
    }

    pub fn build_kdtree(&mut self) {
        let tris = self.world_tris();
        self.kdtree = Some(build_kdtree(&tris));
    }

    pub fn upload_models(&mut self, vulkan: &mut VulkanState) {
        let vs = self.model_data.iter()
            .map(|t| t.vertices())
            .flatten()
            .collect::<Vec<Vertex>>();

        let is = self.model_data.iter()
            .enumerate()
            .map(|(i, t)| t.indices(i as u32 * 3))
            .flatten()
            .collect::<Vec<u32>>();

        let ns = self.model_data.iter()
            .map(|t| t.normals())
            .flatten()
            .collect::<Vec<Normal>>();

        let (vb, ib, nb) = vulkan.transfer_object_data(vs, is, ns);
        self.vertex_buffer = Some(vb);
        self.index_buffer = Some(ib);
        self.normal_buffer = Some(nb);
    }

    pub fn model_from_tri(&self, tri: u32) -> usize {
        let mut idx = 0;
        for (i, m) in self.models.iter().enumerate() {
            if idx <= tri && tri < idx+m.count {
                return i;
            }

            idx += m.count;
        }

        panic!("no model with tri: {tri}");
    }

    pub fn world_tris(&self) -> Vec<Triangle> {
        let mut tris = vec![];
        for i in 0..self.models.len() {
            let Model { index, count } = self.models[i];

            for j in index..index+count {
                let mut t = self.model_data[j as usize].clone();
                t.v0 += self.positions[i];
                t.v1 += self.positions[i];
                t.v2 += self.positions[i];

                tris.push(t);
            }
        }

        tris
    }

    pub fn draw(&mut self, vs: &mut VulkanState) {
        // Calculate uniforms for each object
        let ext = vs.logical_size();
        let aspect_ratio = ext.0 as f32 / ext.1 as f32;

        let proj = cgmath::perspective(
            Rad(self.fov),
            aspect_ratio,
            0.01,
            100.0
        );

        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 1.0, -6.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        );

        let scale = Matrix4::from_scale(1.00);

        let rotation =
            Matrix3::from_angle_x(Rad(self.rotx)) *
            Matrix3::from_angle_y(Rad(self.roty)) *
            Matrix3::from_angle_z(Rad(self.rotz));


        let mut uniforms = Vec::with_capacity(self.models.len());

        for _ in 0..self.lines.len() {
            let world = Matrix4::from(rotation);
            let uniform_data = Uniform {
                world: world.into(),
                view: (view * scale).into(),
                proj: proj.into(),
                o_color: Vector4::new(1.0, 1.0, 1.0, 1.0).into()
            };
            uniforms.push(uniform_data);
        }

        for i in 0..self.models.len() {
            let mscale = self.scales[i];
            let mscale = Matrix4::from_nonuniform_scale(mscale.x, mscale.y, mscale.z);

            let world = 
                Matrix4::from(rotation)
                * Matrix4::from_translation(self.positions[i])
                * mscale;

            let uniform_data = Uniform {
                world: world.into(),
                view: (view * scale).into(),
                proj: proj.into(),
                o_color: self.colors[i].into()
            };
            uniforms.push(uniform_data);
        }

        let models = self.lines.iter().chain(self.models.iter());

        vs.draw(
            self.vertex_buffer.as_ref().unwrap().clone(),
            self.index_buffer.as_ref().unwrap().clone(),
            self.normal_buffer.as_ref().unwrap().clone(),
            models,
            &uniforms
        );
    }

    pub fn zoom(&mut self, amt: f32) {
        self.fov = (self.fov + amt)
            .max(0.00001)
            .min(3.0*PI/4.0);
    }

    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.rotx += x;
        self.roty += y;
        self.rotz += z;
    }

    pub fn add_light(&mut self, l: Light) {
        self.lights.push(l);
    }

    pub fn intersect(&self, ray: &Ray, ts: &Vec<Triangle>) -> Option<(usize, usize, f32)> {
        let kdtree = self.kdtree.as_ref().unwrap();
        let (ti, d) = kdtree.intersect(ray, ts)?;

        let mi = self.model_from_tri(ti as u32);
        Some((mi, ti, d))
    }

    pub fn trace(&mut self) {
        let tris = self.world_tris();

        for light in self.lights.clone().into_iter() {
            match light {
                Light::Laser(_, _) => {
                    // For a laser spawn a single ray and always render it
                    let r = light.spawn();
                    self.trace_ray(&r, &tris);
                }
                Light::Point(_) => {
                    // for a point light shoot out a bunch of rays, only
                    // displaying those that hit a lens or reflector
                    for _ in 0..1000 {
                        let r = light.spawn();
                        if let Some((mi, _, _)) = self.intersect(&r, &tris) {
                            match self.materials[mi] {
                                Material::Glass(_) | Material::Mirror => self.trace_ray(&r, &tris),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    // trace a ray, always adding it when it hits something
    fn trace_ray(&mut self, r: &Ray, tris: &Vec<Triangle>) {
        if let Some((mi, ti, d)) = self.intersect(r, tris) {
            self.add_ray(r, d);
            let t = &tris[ti];
            let n = t.normals()[0];
            let n: Vector3<f32> = n.normal.into();

            let inside = r.inside;

            // Trace the rest
            match self.materials[mi] {
                Material::Glass(eta) => {
                    let cos_theta = dot(-r.dir, n).max(1.0);
                    let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();

                    let etai_over_etat = if inside {
                        eta
                    } else {
                        1.0 / eta
                    };

                    if sin_theta * etai_over_etat > 1.0 {
                        return;
                    }

                    let r_out_perp = etai_over_etat * (r.dir + cos_theta*n);
                    let r_out_para = -(1.0 - r_out_perp.magnitude2()).abs().sqrt() * n;

                    let mut r = Ray::new(
                        r.origin + r.dir * d,
                        r_out_perp + r_out_para
                    );
                    r.inside = !inside;

                    self.trace_ray(&r, tris);
                }
                Material::Mirror => {}
                Material::Solid => {return;}
            };
        }
    }

    fn add_ray(&mut self, r: &Ray, d: f32) {
        // When adding a ray, create a new model with the tesselated ray
        // and then add the model to the lines array
        let ts = r.tesselate(d);
        let m = self.add_model(ts);
        self.lines.push(m)
    }
}
