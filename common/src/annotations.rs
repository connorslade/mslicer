//! Annotations are ephemeral hints produced by pre- and/or post-processing
//! passes. They come in flavors like log messages, info, warn, error, etc.
//! They shouldn't typically be stored since they can be reproduced at any
//! time by running the passes. But they need to be accessible for the
//! renderer.
use bitflags::bitflags;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
    sync::Arc,
};

/// Vertical layer index (bottom-up).
pub type SliceIdx = usize;
/// Pixel position within slice.
pub type Coord = [i64; 2];

/// Level-wrapped annotations.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum AnnotationLevel {
    Debug(Annotation),
    Info(Annotation),
    Warn(Annotation),
    Error(Annotation),
}

impl Deref for AnnotationLevel {
    type Target = Annotation;

    fn deref(&self) -> &Self::Target {
        use AnnotationLevel::*;
        match self {
            Debug(annotation) => annotation,
            Info(annotation) => annotation,
            Warn(annotation) => annotation,
            Error(annotation) => annotation,
        }
    }
}

impl DerefMut for AnnotationLevel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        use AnnotationLevel::*;
        match self {
            Debug(annotation) => annotation,
            Info(annotation) => annotation,
            Warn(annotation) => annotation,
            Error(annotation) => annotation,
        }
    }
}

impl AnnotationLevel {
    #[inline]
    pub fn flags(&self) -> AnnotationLevelFlags {
        use AnnotationLevel::*;
        match self {
            Debug(_) => AnnotationLevelFlags::DEBUG,
            Info(_) => AnnotationLevelFlags::INFO,
            Warn(_) => AnnotationLevelFlags::WARN,
            Error(_) => AnnotationLevelFlags::ERROR,
        }
    }

    /// Returns byte representation using [AnnotationLevelFlags].
    #[inline]
    pub fn to_byte(&self) -> u8 {
        self.flags().0 | self.kind() as u8
    }
}

/// Bit flags for annotation levels.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct AnnotationLevelFlags(u8);

bitflags! {
    impl AnnotationLevelFlags: u8 {
        const ERROR = 0b1000_0000;
        const WARN  = 0b0100_0000;
        const INFO  = 0b0010_0000;
        const DEBUG = 0b0001_0000;
    }
}

/// Annotation kinds are the semantic type of the annotation.
/// This is used to communicate just the type without the data.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum AnnotationKind {
    Island = 0b0000_0001,
}

/// Currently known annotations with data.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Annotation {
    /// An unsupported pixel on slice `slice_idx` at `coord`.
    Island { slice_idx: SliceIdx, coord: Coord },
}

impl Annotation {
    /// Mark annotation as debug.
    pub fn as_debug(self) -> AnnotationLevel {
        AnnotationLevel::Debug(self)
    }

    /// Mark annotation as info.
    pub fn as_info(self) -> AnnotationLevel {
        AnnotationLevel::Info(self)
    }

    /// Mark annotation as warning.
    pub fn as_warn(self) -> AnnotationLevel {
        AnnotationLevel::Warn(self)
    }

    /// Mark annotation as error.
    pub fn as_error(self) -> AnnotationLevel {
        AnnotationLevel::Error(self)
    }

    #[inline]
    pub fn slice_idx(&self) -> Option<usize> {
        match self {
            Annotation::Island { slice_idx, .. } => Some(*slice_idx),
        }
    }

    #[inline]
    pub fn slice_pos(&self) -> Option<&Coord> {
        match self {
            Annotation::Island { coord, .. } => Some(coord),
        }
    }

    #[inline]
    pub fn kind(&self) -> AnnotationKind {
        match self {
            Annotation::Island { .. } => AnnotationKind::Island,
        }
    }
}

/// The Neighbor trait allows to define arbitrary reachability.
trait Neighbor {
    /// Returns whether self is reachable from other.
    fn is_neighbor_of(&self, other: &Self) -> bool;
}

/// Defines standard neighborhood of closest 8 pixels.
#[inline]
fn connected(c1: &Coord, c2: &Coord) -> bool {
    (c1[0] - c2[0]).abs() <= 1 && (c1[1] - c2[1]).abs() <= 1
}

impl Neighbor for Annotation {
    fn is_neighbor_of(&self, other: &Self) -> bool {
        match *self {
            Annotation::Island {
                slice_idx: l1,
                coord: c1,
            } => matches!(*other, Annotation::Island {
                    slice_idx: l2,
                    coord: c2,
                } if l1 == l2 && connected(&c1, &c2)),
        }
    }
}

impl From<AnnotationLevel> for Annotation {
    fn from(value: AnnotationLevel) -> Self {
        match value {
            AnnotationLevel::Debug(a) => a,
            AnnotationLevel::Info(a) => a,
            AnnotationLevel::Warn(a) => a,
            AnnotationLevel::Error(a) => a,
        }
    }
}

impl Display for Annotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Annotation::Island { slice_idx, coord } => f.write_str(&format!(
                "island on layer #{} at ({}, {})",
                slice_idx, coord[0], coord[1]
            )),
        }
    }
}

/// Groups islands into clusters of neighboring pixels.
/// References an underlying shared [Annotations] object.
#[derive(Debug, derive_more::Deref, Eq, Hash, PartialEq)]
pub struct ClusterView {
    pub annotations: Arc<Annotations>,
    #[deref]
    pub clusters: Vec<Cluster>,
}

impl ClusterView {
    /// Clusters the given annotations.
    pub fn new(annotations: Arc<Annotations>) -> Self {
        Self {
            annotations: annotations.clone(),
            clusters: cluster_islands(annotations),
        }
    }
}

/// Lightweight collection of annotations grouped by some criterion.
/// References an underlying shared [Annotations] object.
#[derive(Debug, derive_more::Deref, Eq, Hash, PartialEq)]
pub struct Cluster {
    annotations: Arc<Annotations>,
    #[deref]
    annotation_indices: Vec<usize>,
}

impl Cluster {
    pub fn new(annotations: Arc<Annotations>, indices: Vec<usize>) -> Self {
        Self {
            annotations,
            annotation_indices: indices,
        }
    }

    pub fn slice_idx(&self) -> Option<SliceIdx> {
        self.annotations[self.annotation_indices[0]].slice_idx()
    }

    /// Computes the "center" of a cluster by averaging the coordinates.
    pub fn center(&self) -> Option<Coord> {
        let locs = self
            .annotation_indices
            .iter()
            .filter_map(|&idx| self.annotations[idx].slice_pos())
            .collect::<Vec<_>>();
        let num = locs.len() as i64;
        locs.into_iter()
            .cloned()
            .reduce(|[x1, y1], [x2, y2]| [x1 + x2, y1 + y2])
            .map(|[x, y]| [x / num, y / num])
    }
}

/// Computes clusters of the given set of annotations.
///
/// Algorithm:
///   1. Compute the index of each annotation, then drop non-islands.
///   2. Group remaining annotations by their slice index.
///   3. For each slice repeat while the group is not empty:
///      1. Move one annotation from the group into the working set.
///      2. Repeat while the working set is not empty:
///         1. Move one annotation from working set to visited set.
///         2. Move all neighbors of that annotation to working set.
///   4. Create new cluster from visited set and repeat.
///
/// # Parallelism
///
/// Filtering and grouping by slice is done in parallel using rayon.
/// This phase end at the reduce, which yields the slice groups.
/// Then all slices are processed in parallel, but each slice sequentially.
///
fn cluster_islands(annotations: Arc<Annotations>) -> Vec<Cluster> {
    let by_layers = annotations
        .par_iter()
        .enumerate()
        .filter(|(_, a)| matches!(***a, Annotation::Island { .. }))
        .flat_map(|(aidx, a)| a.slice_idx().map(|idx| (idx, aidx)))
        .fold(
            HashMap::new,
            |mut acc: HashMap<_, Vec<usize>>, (key, val)| {
                acc.entry(key).or_default().push(val);
                acc
            },
        )
        .reduce(HashMap::new, |mut acc1, acc2| {
            for (key, mut vals) in acc2 {
                acc1.entry(key).or_default().append(&mut vals);
            }
            acc1
        })
        .par_iter_mut()
        .map(|(_, open)| {
            let mut ret: Vec<Cluster> = vec![];
            while !open.is_empty() {
                let mut working_set = vec![open.remove(0)];
                let mut visited: Vec<usize> = vec![];
                while !working_set.is_empty() {
                    let curr = working_set.remove(0);
                    let (conn, rest): (Vec<usize>, Vec<usize>) = (open.clone())
                        .into_iter()
                        .partition(|&a| annotations[a].is_neighbor_of(&annotations[curr]));
                    *open = rest;
                    working_set.extend(conn);
                    visited.push(curr);
                }
                ret.push(Cluster::new(annotations.clone(), visited));
            }
            ret
        })
        .reduce(Vec::new, |mut acc1, mut acc2| {
            acc1.append(&mut acc2);
            acc1
        });
    by_layers
}

/// A collection of [AnnotationLevel].
#[derive(
    Clone,
    Debug,
    Default,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::From,
    Eq,
    Hash,
    PartialEq,
)]
pub struct Annotations {
    annotations: Vec<AnnotationLevel>,
}

impl Annotations {
    /// Iterator over all islands.
    #[inline]
    pub fn islands(&self) -> impl Iterator<Item = usize> + '_ {
        self.annotations
            .iter()
            .enumerate()
            .filter(|(_, &a)| matches!(*a, Annotation::Island { .. }))
            .map(|(idx, _)| idx)
    }
}

impl<'a> IntoIterator for &'a Annotations {
    type Item = &'a AnnotationLevel;
    type IntoIter = std::slice::Iter<'a, AnnotationLevel>;

    fn into_iter(self) -> Self::IntoIter {
        self.annotations.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection::vec as pvec;
    use proptest::prelude::*;
    use std::sync::Arc;

    #[test]
    fn cluster_by_single_cluster() {
        let example: Arc<Annotations> = Arc::new(
            vec![
                Annotation::Island {
                    slice_idx: 0,
                    coord: [1, 1],
                }
                .as_debug(),
                Annotation::Island {
                    slice_idx: 0,
                    coord: [2, 2],
                }
                .as_debug(),
                Annotation::Island {
                    slice_idx: 0,
                    coord: [1, 2],
                }
                .as_debug(),
            ]
            .into(),
        );
        let cluster_view = ClusterView::new(example.clone());
        let clusters = cluster_view.clusters;
        assert_eq!(clusters.len(), 1, "only one cluster should be found");
        println!("clusters: {:?}", clusters);
        assert_eq!(
            clusters.get(0).map_or(0, |e| e.len()),
            example.len(),
            "the first cluster should have the same length as the example annotations"
        );
        let all_are_in_cluster0 = example
            .iter()
            .enumerate()
            .all(|(a, _)| clusters.get(0).map(|c| c.contains(&a)).unwrap_or(false));
        assert!(
            all_are_in_cluster0,
            "all example annotations must be in cluster #0"
        );
    }

    #[test]
    fn cluster_by_disjoint() {
        let example: Arc<Annotations> = Arc::new(
            vec![
                Annotation::Island {
                    slice_idx: 0,
                    coord: [4, 4],
                }
                .as_debug(),
                Annotation::Island {
                    slice_idx: 0,
                    coord: [2, 2],
                }
                .as_debug(),
                Annotation::Island {
                    slice_idx: 0,
                    coord: [0, 0],
                }
                .as_debug(),
            ]
            .into(),
        );
        let cluster_view = ClusterView::new(example.clone());
        let clusters = cluster_view.clusters;
        println!("clusters: {:?}", clusters);
        assert_eq!(clusters.len(), 3, "three clusters should be found");
        assert_eq!(
            clusters.iter().map(|c| c.len()).sum::<usize>(),
            example.len()
        );
    }

    #[test]
    fn cluster_by_single_cluster2() {
        let example: Arc<Annotations> = Arc::new(
            vec![
                Annotation::Island {
                    slice_idx: 0,
                    coord: [1, 1],
                }
                .as_debug(),
                Annotation::Island {
                    slice_idx: 0,
                    coord: [2, 2],
                }
                .as_debug(),
                Annotation::Island {
                    slice_idx: 0,
                    coord: [1, 2],
                }
                .as_debug(),
                Annotation::Island {
                    slice_idx: 0,
                    coord: [2, 3],
                }
                .as_debug(),
                Annotation::Island {
                    slice_idx: 0,
                    coord: [2, 4],
                }
                .as_debug(),
                Annotation::Island {
                    slice_idx: 0,
                    coord: [2, 5],
                }
                .as_debug(),
            ]
            .into(),
        );
        let cluster_view = ClusterView::new(example.clone());
        let clusters = cluster_view.clusters;
        assert_eq!(clusters.len(), 1, "only one cluster should be found");
        println!("clusters: {:?}", clusters);
        assert_eq!(
            clusters.get(0).map_or(0, |e| e.len()),
            example.len(),
            "the first cluster should have the same length as the example annotations"
        );
        let all_are_in_cluster0 = example
            .iter()
            .enumerate()
            .all(|(a, _)| clusters.get(0).map(|c| c.contains(&a)).unwrap_or(false));
        assert!(
            all_are_in_cluster0,
            "all example annotations must be in cluster #0"
        );
    }

    prop_compose! {
        fn arb_island()(args in (any::<usize>(), -100000i64..100000, -100000i64..100000)) -> Annotation {
            let (slice_idx, x, y) = args;
            Annotation::Island {
                slice_idx,
                coord: [x, y],
            }
        }
    }

    prop_compose! {
        fn arb_annotations()(anns in pvec(arb_island(), 0..100)) -> Annotations {
            anns.into_iter().map(|a| a.as_debug()).collect::<Vec<_>>().into()
        }
    }

    proptest! {
        #[test]
        fn clusters_correctly(annotations in arb_annotations()) {
            let aa = Arc::new(annotations);
            let clusters = ClusterView::new(aa.clone()).clusters;
            let total_num = clusters.iter().map(|c| c.len()).sum::<usize>();

            assert_eq!(total_num, aa.len(),
                "expected {} annotations, but sum of clusters is {}",
                aa.len(), total_num);

            let all = clusters
                .iter()
                .map(|cluster| cluster.iter())
                .flatten()
                .collect::<Vec<_>>();
            assert!((0..aa.len()).all(|a| all.contains(&&a)),
                "all annotations are contained in some cluster");

            for cluster in clusters {
                if cluster.len() != 1 {
                    let c2 = cluster.clone();
                    for a1 in cluster.iter() {
                        assert!(c2.iter().any(|a2| *a1 != *a2 && aa[*a1].is_neighbor_of(&aa[*a2])),
                            "{} is not connected to any other annotation in cluster: {:?}", a1, c2);
                    }
                }
            }
        }
    }

    prop_compose! {
        fn arb_isolated(radius: i64)(size in 1..10000) -> Annotations {
            let s = (size as f32).sqrt() as i64;
            (0..s)
                .map(|y| (0..s)
                    .map(move |x| Annotation::Island {
                        slice_idx: 0,
                        coord: [x as i64 * (radius + 1), y as i64 * (radius + 1)]
                    }.as_debug()))
                .flatten()
                .collect::<Vec<_>>()
                .into()
        }
    }

    prop_compose! {
        fn arb_isolated_varying_radius()
            (radius in 1..20_i64)
            (annotations in arb_isolated(radius)) -> Annotations {
                annotations
        }
    }

    proptest! {
        #[test]
        fn isolated_annotations_yield_one_cluster_per_annotation(anns in arb_isolated_varying_radius()) {
            let expected_clusters = anns.len();
            assert_eq!(ClusterView::new(Arc::new(anns)).clusters.len(), expected_clusters);
        }
    }
}
