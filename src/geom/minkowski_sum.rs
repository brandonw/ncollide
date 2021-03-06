use std::num::Zero;
use nalgebra::na::{Dot, Norm, Dim, ApproxEq};
use nalgebra::na;
use geom::Reflection;
use math::{Scalar, Vect, Matrix};

/// Type of an implicit representation of the Configuration Space Obstacle
/// formed by two geometric objects.
pub type CSO<'a, G1, G2> = MinkowskiSum<'a, G1, Reflection<'a, G2>>;
pub type AnnotatedCSO<'a, G1, G2> = AnnotatedMinkowskiSum<'a, G1, Reflection<'a, G2>>;

/**
 * Implicit representation of the Minkowski sum of two geometries.
 *
 * The only way to obtain the sum points is to use its support mapping
 * function.
 *
 *  - `G1`: type of the first object involved on the sum.
 *  - `G2`: type of the second object involved on the sum.
 */
#[deriving(Show)]
pub struct MinkowskiSum<'a, G1, G2> {
    m1: &'a Matrix,
    g1: &'a G1,
    m2: &'a Matrix,
    g2: &'a G2
}

impl<'a, G1, G2> MinkowskiSum<'a, G1, G2> {
    /**
     * Builds the Minkowski sum of two geometries. Since the representation is
     * implicit, this is done in constant time.
     */
    #[inline]
    pub fn new(m1: &'a Matrix, g1: &'a G1, m2: &'a Matrix, g2: &'a G2) -> MinkowskiSum<'a, G1, G2> {
        MinkowskiSum { m1: m1, g1: g1, m2: m2, g2: g2 }
    }

    /// The transformation matrix of the first geometry of this Minkowski Sum.
    #[inline]
    pub fn m1(&self) -> &'a Matrix {
        self.m1
    }

    /// The transformation matrix of the second geometry of this Minkowski Sum.
    #[inline]
    pub fn m2(&self) -> &'a Matrix {
        self.m2
    }

    /// The first geometry of this Minkowski Sum.
    #[inline]
    pub fn g1(&self) -> &'a G1 {
        self.g1
    }

    /// The second geometry of this Minkowski Sum.
    #[inline]
    pub fn g2(&self) -> &'a G2 {
        self.g2
    }
}

/**
 * Same as the MinkowskiSum but with a support mapping which keeps track of the
 * original supports points from the two wrapped geometries.
 *
 * * `G1`: type of the first object involved on the sum.
 * * `G2`: type of the second object involved on the sum.
 */
#[deriving(Show)]
pub struct AnnotatedMinkowskiSum<'a, G1, G2> {
    m1: &'a Matrix,
    g1: &'a G1,
    m2: &'a Matrix,
    g2: &'a G2
}

impl<'a, G1, G2> AnnotatedMinkowskiSum<'a, G1, G2> {
    /**
     * Builds the Minkowski sum of two geometries. Since the representation is
     * implicit, this is done in constant time.
     */
    #[inline]
    pub fn new(m1: &'a Matrix, g1: &'a G1, m2: &'a Matrix, g2: &'a G2) -> AnnotatedMinkowskiSum<'a, G1, G2> {
        AnnotatedMinkowskiSum { m1: m1, g1: g1, m2: m2, g2: g2 }
    }

    /// The transformation matrix of the first geometry of this Minkowski Sum.
    #[inline]
    pub fn m1(&self) -> &'a Matrix {
        self.m1
    }

    /// The transformation matrix of the second geometry of this Minkowski Sum.
    #[inline]
    pub fn m2(&self) -> &'a Matrix {
        self.m2
    }

    /// The first geometry of this Minkowski Sum.
    #[inline]
    pub fn g1(&self) -> &'a G1 {
        self.g1
    }

    /// The second geometry of this Minkowski Sum.
    #[inline]
    pub fn g2(&self) -> &'a G2 {
        self.g2
    }
}

// FIXME: AnnotatedPoint is not a good name.
#[doc(hidden)]
#[deriving(Clone, Show, Encodable, Decodable)]
pub struct AnnotatedPoint {
    orig1: Vect,
    orig2: Vect,
    point: Vect
}

impl AnnotatedPoint {
    #[doc(hidden)]
    #[inline]
    pub fn new(orig1: Vect, orig2: Vect, point: Vect) -> AnnotatedPoint {
        AnnotatedPoint {
            orig1: orig1,
            orig2: orig2,
            point: point
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn point<'r>(&'r self) -> &'r Vect {
        &'r self.point
    }

    #[doc(hidden)]
    #[inline]
    pub fn orig1<'r>(&'r self) -> &'r Vect {
        &'r self.orig1
    }

    #[doc(hidden)]
    #[inline]
    pub fn orig2<'r>(&'r self) -> &'r Vect {
        &'r self.orig2
    }
}

impl AnnotatedPoint {
    #[doc(hidden)]
    #[inline]
    pub fn new_invalid(point: Vect) -> AnnotatedPoint {
        AnnotatedPoint {
            orig1: na::zero(),
            orig2: na::zero(),
            point: point
        }
    }
}


impl Zero for AnnotatedPoint {
    #[inline]
    fn zero() -> AnnotatedPoint {
        AnnotatedPoint::new(na::zero(), na::zero(), na::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.point.is_zero()
    }
}

impl Sub<AnnotatedPoint, AnnotatedPoint> for AnnotatedPoint {
    #[inline]
    fn sub(&self, other: &AnnotatedPoint) -> AnnotatedPoint {
        AnnotatedPoint::new(self.orig1 - other.orig1,
        self.orig2 - other.orig2,
        self.point - other.point)
    }
}

impl Add<AnnotatedPoint, AnnotatedPoint> for AnnotatedPoint {
    #[inline]
    fn add(&self, other: &AnnotatedPoint) -> AnnotatedPoint {
        AnnotatedPoint::new(self.orig1 + other.orig1,
        self.orig2 + other.orig2,
        self.point + other.point)
    }
}

impl Neg<AnnotatedPoint> for AnnotatedPoint {
    #[inline]
    fn neg(&self) -> AnnotatedPoint {
        AnnotatedPoint::new(-self.orig1, -self.orig2, -self.point)
    }
}

impl Dim for AnnotatedPoint {
    #[inline]
    fn dim(_: Option<AnnotatedPoint>) -> uint {
        na::dim::<Vect>()
    }
}

impl Dot<Scalar> for AnnotatedPoint {
    #[inline]
    fn dot(a: &AnnotatedPoint, b: &AnnotatedPoint) -> Scalar {
        na::dot(&a.point, &b.point)
    }

    #[inline]
    fn sub_dot(a: &AnnotatedPoint, b: &AnnotatedPoint, c: &AnnotatedPoint) -> Scalar {
        na::sub_dot(&a.point, &b.point, &c.point)
    }
}

impl Norm<Scalar> for AnnotatedPoint {
    #[inline]
    fn norm(v: &AnnotatedPoint) -> Scalar {
        na::norm(&v.point)
    }

    #[inline]
    fn sqnorm(v: &AnnotatedPoint) -> Scalar {
        na::sqnorm(&v.point)
    }

    /// Be careful: only the `point` is normalized, not `orig1` nor `orig2`.
    #[inline]
    fn normalize_cpy(v: &AnnotatedPoint) -> AnnotatedPoint {
        AnnotatedPoint::new(v.orig1.clone(), v.orig2.clone(), na::normalize(&v.point))
    }

    /// Be careful: only the `point` is normalized, not `orig1` nor `orig2`.
    #[inline]
    fn normalize(&mut self) -> Scalar {
        self.point.normalize()
    }
}

impl Div<Scalar, AnnotatedPoint> for AnnotatedPoint {
    #[inline]
    fn div(&self, n: &Scalar) -> AnnotatedPoint {
        AnnotatedPoint::new(self.orig1 / *n, self.orig2 / *n, self.point / *n)
    }
}

impl Mul<Scalar, AnnotatedPoint> for AnnotatedPoint {
    #[inline]
    fn mul(&self, n: &Scalar) -> AnnotatedPoint {
        AnnotatedPoint::new(self.orig1 * *n, self.orig2 * *n, self.point * *n)
    }
}

impl PartialEq for AnnotatedPoint {
    #[inline]
    fn eq(&self, other: &AnnotatedPoint) -> bool {
        self.point == other.point
    }

    #[inline]
    fn ne(&self, other: &AnnotatedPoint) -> bool {
        self.point != other.point
    }
}

impl ApproxEq<Scalar> for AnnotatedPoint {
    #[inline]
    fn approx_epsilon(_: Option<AnnotatedPoint>) -> Scalar {
        ApproxEq::approx_epsilon(None::<Scalar>)
    }

    #[inline]
    fn approx_eq_eps(a: &AnnotatedPoint, b: &AnnotatedPoint, eps: &Scalar) -> bool {
        na::approx_eq_eps(&a.point, &b.point, eps)
    }
}
