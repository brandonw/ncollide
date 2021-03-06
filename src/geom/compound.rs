//!
//! Geometry composed from the union of primitives.
//!

use nalgebra::na;
use bounding_volume::{LooseBoundingVolume, AABB, HasAABB};
use ray::Ray;
use partitioning::BVT;
use partitioning::{BoundingVolumeInterferencesCollector, RayInterferencesCollector};
use geom::{Geom, ConcaveGeom};
use math::Matrix;

/// A compound geometry with an aabb bounding volume.
///
/// A compound geometry is a geometry composed of the union of several simpler geometry. This is
/// the main way of creating a concave geometry from convex parts. Each parts can have its own
/// delta transformation to shift or rotate it with regard to the other geometries.
pub struct Compound {
    shapes: Vec<(Matrix, Box<Geom:Send>)>,
    bvt:    BVT<uint, AABB>,
    bvs:    Vec<AABB>
}

impl Clone for Compound {
    fn clone(&self) -> Compound {
        Compound {
            shapes: self.shapes.iter().map(|&(ref m, ref s)| (m.clone(), s.duplicate())).collect(),
            bvt:    self.bvt.clone(),
            bvs:    self.bvs.clone()
        }
    }
}

impl Compound {
    /// Builds a new compound shape from a list of shape with their respective delta
    /// transformation.
    pub fn new(shapes: Vec<(Matrix, Box<Geom:Send>)>) -> Compound {
        let mut bvs    = Vec::new();
        let mut leaves = Vec::new();

        for (i, &(ref delta, ref shape)) in shapes.iter().enumerate() {
            let bv = shape.aabb(delta).loosened(na::cast(0.04)); // loosen for better persistancy

            bvs.push(bv.clone());
            leaves.push((i, bv));
        }

        let bvt = BVT::new_kdtree(leaves);

        Compound {
            shapes: shapes,
            bvt:    bvt,
            bvs:    bvs
        }
    }
}

impl Compound {
    /// The shapes of this compound geometry.
    #[inline]
    pub fn shapes<'r>(&'r self) -> &'r [(Matrix, Box<Geom>)] {
        self.shapes.as_slice()
    }

    /// The optimization structure used by this compound geometry.
    #[inline]
    pub fn bvt<'r>(&'r self) -> &'r BVT<uint, AABB> {
        &'r self.bvt
    }

    /// The shapes bounding volumes.
    #[inline]
    pub fn bounding_volumes<'r>(&'r self) -> &'r [AABB] {
        self.bvs.as_slice()
    }
}

impl ConcaveGeom for Compound {
    #[inline(always)]
    fn map_part_at<T>(&self, i: uint, f: |&Matrix, &Geom| -> T) -> T{
        let &(ref m, ref g) = self.shapes.get(i);

        f(m, *g)
    }

    #[inline(always)]
    fn map_transformed_part_at<T>(&self, m: &Matrix, i: uint, f: |&Matrix, &Geom| -> T) -> T{
        let &(ref lm, ref g) = self.shapes.get(i);

        f(&(m * *lm), *g)
    }

    #[inline]
    fn approx_interferences_with_aabb(&self, aabb: &AABB, out: &mut Vec<uint>) {
        let mut visitor = BoundingVolumeInterferencesCollector::new(aabb, out);
        self.bvt.visit(&mut visitor);
    }

    #[inline]
    fn approx_interferences_with_ray(&self, ray: &Ray, out: &mut Vec<uint>) {
        let mut visitor = RayInterferencesCollector::new(ray, out);
        self.bvt.visit(&mut visitor);
    }

    #[inline]
    fn aabb_at<'a>(&'a self, i: uint) -> &'a AABB {
        self.bvs.get(i)
    }
}
