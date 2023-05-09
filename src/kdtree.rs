use crate::geometry::{AABB, Axis, Ray, Triangle};
use cgmath::Vector3 as Vec3;

const MAX_DEPTH: usize = 20;
const NUM_POLYGONS: usize = 1;

#[derive(Debug)]
pub enum KDNode {
    /// Decision Branch on Axis = f32, id for lef
    Branch(Axis, f32, AABB, Box<KDNode>, Box<KDNode>),
    Leaf(AABB, Vec<usize>)
}

impl KDNode {
    fn aabb(&self) -> &AABB {
        match self {
            KDNode::Branch(_, _, b, _, _) => b,
            KDNode::Leaf(b, _) => b
        }
    }
}

impl KDNode {
    pub fn intersect(&self, r: &Ray, ts: &Vec<Triangle>) -> Option<(usize, f32)> {
        if !self.aabb().intersect(r) {
            return None;
        }

        match self {
            KDNode::Branch(a, v, _, left, right) => {
                let (dist, dir) = match a {
                    Axis::X => (r.origin.x-v, r.dir.x),
                    Axis::Y => (r.origin.y-v, r.dir.y),
                    Axis::Z => (r.origin.z-v, r.dir.z)
                };

                match (dist < 0.0, dir < 0.0) {
                    // Only need to check left side
                    (true, true) => left.intersect(r, ts),
                    // Only need to check right side
                    (false, false) => right.intersect(r, ts),
                    // Check left and then right
                    (true, false) => {
                        let res = left.intersect(r, ts);
                        if res.is_none() {
                            right.intersect(r, ts)
                        } else {
                            res
                        }
                    }
                    // Check right and then left
                    (false, true) => {
                        let res = right.intersect(r, ts);
                        if res.is_none() {
                            left.intersect(r, ts)
                        } else {
                            res
                        }
                    }
                }
            }
            KDNode::Leaf(_, objs) => {
                objs.iter()
                    .filter_map(|&i| ts[i].intersect(r).map(|d| (i, d)))
                    .filter(|(_, d)| { *d > 0.000001 })
                    .min_by(|a, b| {
                        a.1.partial_cmp(&b.1).unwrap()
                    })
            }
        }
    }
}

pub fn build_kdtree(g: &Vec<Triangle>) -> KDNode {
    let aabb = g.iter()
        .fold(
            AABB { min: Vec3::new(0.0, 0.0, 0.0), max: Vec3::new(0.0, 0.0, 0.0) },
            |a, b| a.union(b.fit())
        );

    build_kdtree_h(g.iter().enumerate().collect(), aabb, Axis::X, 0)
}

fn build_kdtree_h<'a>(g: Vec<(usize, &'a Triangle)>, aabb: AABB, axis: Axis, depth: usize) -> KDNode {
    // If we have reached our max depth return a leaf node containing the rest of the geometry
    if depth >= MAX_DEPTH {
        return KDNode::Leaf(aabb, g.iter().map(|a| a.0).collect());
    }

    // If there are few enough polygons also just create a leaf node
    if g.len() <= NUM_POLYGONS {
        return KDNode::Leaf(aabb, g.iter().map(|a| a.0).collect());
    }

    // Now just subdivide by the axis and recur
    let (l, r, d) = aabb.split(axis);

    let left: Vec<_> = g.iter().filter(|(_, t)| t.left_of(axis, d)).map(|a| *a).collect();
    let right: Vec<_> = g.iter().filter(|(_, t)| t.right_of(axis, d)).map(|a| *a).collect();

    // If right and left have the same number as the parent just return a leaf node, don't recur
    if left.len() == right.len() && left.len() == g.len() {
        return KDNode::Leaf(aabb, g.iter().map(|a| a.0).collect());
    }

    let new_axis = match axis {
        Axis::X => Axis::Y,
        Axis::Y => Axis::Z,
        Axis::Z => Axis::X
    };

    KDNode::Branch(
        axis,
        d,
        aabb,
        Box::new(build_kdtree_h(left, l, new_axis, depth+1)),
        Box::new(build_kdtree_h(right, r, new_axis, depth+1))
    )
}
