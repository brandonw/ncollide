//! Enum grouping the most useful geometric shapes.

use std::raw::TraitObject;
use std::intrinsics::TypeId;
use std::mem;
use std::any::{Any, AnyRefExt};
use ray::{Ray, RayCast};
use volumetric::Volumetric;
use bounding_volume::{HasBoundingSphere, HasAABB, AABB};
use math::Matrix;

/// Trait (that should be) implemented by each geometry supported by `ncollide`.
pub trait Geom : Volumetric        +
                 HasAABB           +
                 HasBoundingSphere +
                 RayCast           +
                 Any {
    /// Duplicates (clones) this geometry.
    fn duplicate(&self) -> Box<Geom:Send>;
}

/// Trait implemented by concave, composite geometries supported by `ncollide`.
///
/// A composite geometry is composed of several `Geom`. Typically, it is a convex decomposition of
/// a concave geometry.
pub trait ConcaveGeom : Geom {
    /// Applies a function to each sub-geometry of this concave geometry.
    fn map_part_at<T>(&self, uint, |&Matrix, &Geom| -> T) -> T;
    /// Applies a transformation matrix and a function to each sub-geometry of this concave
    /// geometry.
    fn map_transformed_part_at<T>(&self, m: &Matrix, uint, |&Matrix, &Geom| -> T) -> T;

    // FIXME: replace those by a visitor?
    /// Computes the indices of every sub-geometry which might intersect a given AABB.
    fn approx_interferences_with_aabb(&self, &AABB, &mut Vec<uint>);
    /// Computes the indices of every sub-geometry which might intersect a given Ray.
    fn approx_interferences_with_ray(&self, &Ray, &mut Vec<uint>);
    // FIXME: kind of ad-hoc…
    /// Gets the AABB of the geometry identified by the index `i`.
    fn aabb_at<'a>(&'a self, i: uint) -> &'a AABB;
}

impl<T: 'static + Send + Clone + Volumetric + HasAABB + HasBoundingSphere + RayCast + Any>
Geom for T {
    fn duplicate(&self) -> Box<Geom:Send> {
        (box self.clone()) as Box<Geom:Send>
    }
}
// FIXME: we need to implement that since AnyRefExt is only implemented for Any, and it does not
// seem possible to convert a &Geom to a &Any…
impl<'a> AnyRefExt<'a> for &'a Geom {
    #[inline]
    fn is<T: 'static>(self) -> bool {
        // Get TypeId of the type this function is instantiated with
        let t = TypeId::of::<T>();

        // Get TypeId of the type in the trait object
        let boxed = self.get_type_id();

        // Compare both TypeIds on equality
        t == boxed
    }

    #[inline]
    fn as_ref<T: 'static>(self) -> Option<&'a T> {
        if self.is::<T>() {
            unsafe {
                let to: TraitObject = mem::transmute_copy(&self);

                Some(mem::transmute(to.data))
            }
        } else {
            None
        }
    }
}
