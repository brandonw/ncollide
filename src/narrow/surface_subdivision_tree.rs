//! Tree used to cache subdivisions of surfaces.

use collections::HashMap;
use sync::{Arc, RWLock};
use geom::BezierSurface;

/*
 * FIXME:
 *
 * This file contains three things that could be generalized:
 *   * A cache `SurfaceSubdivisionTreeCache` which is nothing more than a HashMap that tracks how
 *   many people use its data, and automatically realase them when nobody use them. *Yes* this
 *   sounds a lot like Rc, but, here we have to handle an association table.
 *
 *   * A `SurfaceSubdivisionTreeRef` that is basically an Arc with a custom Drop.
 *   * A `SurfaceSubdivisionTree` that is just a binary tree.
 */

/// A referenece to an element of the subdivision cache.
///
/// Each time an element is added to the cache, one of thoses references are created.
/// The element will be kept in cache as long as at least one of those references exists.
pub struct SurfaceSubdivisionTreeRef<D> {
    parent_cache: Arc<RWLock<SurfaceSubdivisionTreeCache<D>>>,
    value:        Arc<RWLock<SurfaceSubdivisionTree<D>>>,
    key:          uint
}

impl<D: Send + Share> Clone for SurfaceSubdivisionTreeRef<D> {
    fn clone(&self) -> SurfaceSubdivisionTreeRef<D> {
        self.parent_cache.write().inc_ref_count(self.key);

        SurfaceSubdivisionTreeRef {
            parent_cache: self.parent_cache.clone(),
            value:        self.value.clone(),
            key:          self.key
        }
    }
}

impl<D> SurfaceSubdivisionTreeRef<D> {
    /// Tests if this references the subdivision tree of the bézier surface `b`.
    pub fn is_the_subdivision_tree_of(&self, b: &BezierSurface) -> bool {
        self.key == (b as *BezierSurface as uint)
    }
}

impl<D> Deref<Arc<RWLock<SurfaceSubdivisionTree<D>>>> for SurfaceSubdivisionTreeRef<D> {
    fn deref<'a>(&'a self) -> &'a Arc<RWLock<SurfaceSubdivisionTree<D>>> {
        &'a self.value
    }
}

#[unsafe_destructor]
impl<D: Send + Share> Drop for SurfaceSubdivisionTreeRef<D> {
    fn drop(&mut self) {
        self.parent_cache.write().release_key(self.key)
    }
}

/// A cache that keeps track of parametric surface subdivision trees.
///
/// This cache allows only insersion. Deletion is automatic.
pub struct SurfaceSubdivisionTreeCache<D> {
    // FIXME: we need a way to accesse the refcount to remove trees that are not used any more.
    cache: HashMap<uint, (uint, Arc<RWLock<SurfaceSubdivisionTree<D>>>)>
}

// FIXME: could this kind of cache be useful elsewhere?
impl<D: Send + Share> SurfaceSubdivisionTreeCache<D> {
    /// Creates a new surface subdivision tree cache.
    pub fn new() -> SurfaceSubdivisionTreeCache<D> {
        SurfaceSubdivisionTreeCache {
            cache: HashMap::new()
        }
    }

    /// Removes everything from this cache.
    pub fn clear(&mut self) {
        self.cache.clear()
    }

    // FIXME: it would be much nicer to be able to specify the type of `self` explicitly.
    /// Gets from the cache `cache`, the subdivision tree for the surface `b`.
    pub fn find_or_insert_with(cache: &mut Arc<RWLock<SurfaceSubdivisionTreeCache<D>>>,
                               b:     &BezierSurface,
                               data:  || -> D)
                               -> SurfaceSubdivisionTreeRef<D> {
        let key = b as *BezierSurface as uint;

        let parent_cache = cache.clone();

        let mut wcache = cache.write();
        let elt        = wcache.cache.find_or_insert_with(
            key,
            |_| (0, Arc::new(RWLock::new(SurfaceSubdivisionTree::new_orphan(b.clone(), data(), 1)))));

        // augment the ref-count.
        *elt.mut0() += 1;

        SurfaceSubdivisionTreeRef {
            parent_cache: parent_cache,
            value:        elt.ref1().clone(),
            key:          key
        }
    }

    fn inc_ref_count(&mut self, key: uint) {
        let _ = self.cache.find_mut(&key).map(|v| *v.mut0() += 1);
    }

    fn release_key(&mut self, key: uint) {
        let is_removable = match self.cache.find_mut(&key) {
            Some(ref mut elt) => {
                let new_count = *elt.ref0() - 1;
                *elt.mut0()   = new_count;
                new_count == 0
            },
            None => false,
        };

        if is_removable {
            let _ = self.cache.remove(&key);
        }
    }
}

// FIXME: this could be a generic implementation of a binary tree.
/// A shareable binary tree with a pointer to its parent.
pub struct SurfaceSubdivisionTree<D> {
    rchild:    Option<Arc<RWLock<SurfaceSubdivisionTree<D>>>>,
    lchild:    Option<Arc<RWLock<SurfaceSubdivisionTree<D>>>>,
    timestamp: uint,
    data:      D,
    surface:   BezierSurface
}

impl<D: Send + Share> SurfaceSubdivisionTree<D> {
    /// Creates a new tree with no parent nor children.
    #[inline]
    pub fn new_orphan(b: BezierSurface, data: D, timestamp: uint) -> SurfaceSubdivisionTree<D> {
        SurfaceSubdivisionTree {
            rchild:    None,
            lchild:    None,
            timestamp: timestamp,
            surface:   b,
            data:      data,
        }
    }

    /// The surface contained by this node.
    #[inline]
    pub fn surface<'a>(&'a self) -> &'a BezierSurface {
        &'a self.surface
    }

    /// Reference to the data contained by this node.
    #[inline]
    pub fn data<'a>(&'a self) -> &'a D {
        &'a self.data
    }

    /// Mutable reference to the data contained by this node.
    #[inline]
    pub fn data_mut<'a>(&'a mut self) -> &'a mut D {
        &'a mut self.data
    }

    /// The timestamp of this tree node.
    #[inline]
    pub fn timestamp(&self) -> uint {
        self.timestamp
    }

    /// Sets the timestamp of this tree node.
    #[inline]
    pub fn set_timestamp(&mut self, timestamp: uint) {
        self.timestamp = timestamp
    }

    /// Whether or not this node has a left child.
    #[inline]
    pub fn has_left_child(&self) -> bool {
        self.lchild.is_some()
    }

    /// Whether or not this node has a right child.
    #[inline]
    pub fn has_right_child(&self) -> bool {
        self.rchild.is_some()
    }

    /// A copy of this node right child.
    #[inline]
    pub fn right_child(&self) -> Option<Arc<RWLock<SurfaceSubdivisionTree<D>>>> {
        self.rchild.clone()
    }

    /// A copy of this node left child.
    #[inline]
    pub fn left_child(&self) -> Option<Arc<RWLock<SurfaceSubdivisionTree<D>>>> {
        self.lchild.clone()
    }

    /// A reference to this node right child.
    #[inline]
    pub fn right_child_ref<'a>(&'a self) -> Option<&'a Arc<RWLock<SurfaceSubdivisionTree<D>>>> {
        self.rchild.as_ref()
    }

    /// A reference to this node left child.
    #[inline]
    pub fn left_child_ref<'a>(&'a self) -> Option<&'a Arc<RWLock<SurfaceSubdivisionTree<D>>>> {
        self.lchild.as_ref()
    }

    /// Sets the right child of this node.
    #[inline]
    pub fn set_right_child(&mut self, child: SurfaceSubdivisionTree<D>) {
        assert!(self.rchild.is_none());
        self.rchild = Some(Arc::new(RWLock::new(child)));
    }

    /// Sets the left child of this node.
    #[inline]
    pub fn set_left_child(&mut self, child: SurfaceSubdivisionTree<D>) {
        assert!(self.lchild.is_none());
        self.lchild = Some(Arc::new(RWLock::new(child)));
    }

    /// Returns `true` if `child` is the right child of this node.
    #[inline]
    pub fn is_right_child(&self, child: &Arc<RWLock<SurfaceSubdivisionTree<D>>>) -> bool {
        match self.rchild {
            None         => false,
            Some(ref rc) => child.deref() as *RWLock<SurfaceSubdivisionTree<D>> as uint ==
                            rc.deref()    as *RWLock<SurfaceSubdivisionTree<D>> as uint
        }
    }

    /// Returns `true` if `child` is the left child of this node.
    #[inline]
    pub fn is_left_child(&self, child: &Arc<RWLock<SurfaceSubdivisionTree<D>>>) -> bool {
        match self.lchild {
            None         => false,
            Some(ref rc) => child.deref() as *RWLock<SurfaceSubdivisionTree<D>> as uint ==
                            rc.deref()    as *RWLock<SurfaceSubdivisionTree<D>> as uint
        }
    }

    /// Removes the right child of this node.
    #[inline]
    pub fn remove_right_child(&mut self) {
        self.rchild = None;
    }

    /// Removes the left child of this node.
    #[inline]
    pub fn remove_left_child(&mut self) {
        self.lchild = None;
    }
}
