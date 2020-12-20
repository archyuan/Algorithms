//! # Resources
//!
//! - [**Video**: Agglomerative Clustering: how it works](https://www.youtube.com/watch?v=XJ3194AmH40)
//! - [Understanding the Concept of Hierarchical Clustering](https://towardsdatascience.com/understanding-the-concept-of-hierarchical-clustering-technique-c6e8243758ec)

use crate::algo::graph::WeightedUndirectedAdjacencyMatrixCondensed;
use crate::data_structures::union_find::UnionFind;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use std::cmp::{max, min};

impl WeightedUndirectedAdjacencyMatrixCondensed {
    pub fn hierarchical_cluster_single(&self) -> Vec<(usize, usize, f64)> {
        let n = self.node_count();
        let total_clusters_count = n + (n - 1); // another `n-1` clusters will be generated
        let mut edges = self.edges().collect::<Vec<_>>();
        edges.sort_by_key(|(_f, _t, dist)| OrderedFloat(*dist));
        let mut steps = Vec::new();
        let mut k = n;
        let mut ds = UnionFind::with_ranks([vec![0; n], (1..n).collect()].concat());
        for (i, j, dist) in edges {
            let (_i, _j) = (ds.find(i), ds.find(j));
            if _i != _j {
                ds.union(_i, k);
                ds.union(_j, k);
                steps.push((min(_i, _j), max(_i, _j), dist));

                k += 1;
                if k == total_clusters_count {
                    break;
                }
            }
        }
        steps
    }
    pub fn hierarchical_cluster_complete(&self) -> Vec<(usize, usize, f64)> {
        let n = self.node_count();
        let mut pq = PriorityQueue::with_capacity(n * (n - 1) / 2);
        let total_clusters_count = n + (n - 1); // another `n-1` clusters will be generated
        let mut extended = self.resized(total_clusters_count);

        for (i, j, dist) in self.edges() {
            pq.push((i, j), -OrderedFloat(dist));
        }
        let mut steps = Vec::new();
        let mut k = n;
        let mut ds = UnionFind::with_ranks([vec![0; n], (1..n).collect()].concat());
        while let Some(((i, j), dist)) = pq.pop() {
            let dist = -dist.into_inner();
            let (_i, _j) = (ds.find(i), ds.find(j));
            if i != _i || j != _j {
                continue;
            }
            if i != j {
                ds.union(i, k);
                ds.union(j, k);
                steps.push((i, j, dist));
                for idx in (0..k).filter(|idx| *idx != i && *idx != j) {
                    let dist_to_k = ::partial_min_max::max(extended[(idx, i)], extended[(idx, j)]);
                    extended[(idx, k)] = dist_to_k;

                    pq.push((idx, k), -OrderedFloat(dist_to_k));
                    pq.remove(&(min(idx, i), max(idx, i)));
                    pq.remove(&(min(idx, j), max(idx, j)));
                }

                k += 1;
                if k == total_clusters_count {
                    break;
                }
            }
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algo::geometry::geographical_coordinate::GeographicalCoordinate;
    use lazy_static::lazy_static;
    use rand::{thread_rng, Rng};
    lazy_static! {
        // From kodama's example
        static ref KODAMA_EXAMPLE: Vec<f64> = {
            let coordinates = vec![
                GeographicalCoordinate::new(42.5833333, -71.8027778),
                GeographicalCoordinate::new(42.2791667, -71.4166667),
                GeographicalCoordinate::new(42.3458333, -71.5527778),
                GeographicalCoordinate::new(42.1513889, -71.6500000),
                GeographicalCoordinate::new(42.3055556, -71.5250000),
                GeographicalCoordinate::new(42.2694444, -71.6166667),
            ];
            let mut condensed = vec![];
            for i in 0..coordinates.len() - 1 {
                for j in i + 1..coordinates.len() {
                    condensed.push(coordinates[i].distance(coordinates[j]));
                }
            }
            condensed
        };
        // random
        static ref RANDOM: Vec<f64> =  {
            const N: usize = 10;
            let mut rng = thread_rng();
            let mut condensed = Vec::new();
            condensed.extend((0..(N - 1) * N / 2).map(|_|rng.gen_range(1.0, 10.0)));
            condensed
        };
    }
    fn _generate_expected(
        n: usize,
        v: &Vec<f64>,
        method: ::kodama::Method,
    ) -> Vec<(usize, usize, f64)> {
        let mut condensed_ = v.clone();
        kodama::linkage(&mut condensed_, n, method)
            .steps()
            .into_iter()
            .map(|x| (x.cluster1, x.cluster2, x.dissimilarity))
            .collect::<Vec<_>>()
    }
    #[test]
    fn test_hierarchical_cluster_single() {
        let expected = _generate_expected(6, &KODAMA_EXAMPLE, ::kodama::Method::Single);
        let m = WeightedUndirectedAdjacencyMatrixCondensed::from_slice(&KODAMA_EXAMPLE);
        assert_eq!(m.hierarchical_cluster_single(), expected);

        let expected = _generate_expected(10, &RANDOM, ::kodama::Method::Single);
        let m = WeightedUndirectedAdjacencyMatrixCondensed::from_slice(&RANDOM);
        assert_eq!(m.hierarchical_cluster_single(), expected);
    }

    #[test]
    fn test_hierarchical_cluster_complete() {
        let expected = _generate_expected(6, &KODAMA_EXAMPLE, ::kodama::Method::Complete);
        let m = WeightedUndirectedAdjacencyMatrixCondensed::from_slice(&KODAMA_EXAMPLE);
        assert_eq!(m.hierarchical_cluster_complete(), expected);
        let expected = _generate_expected(10, &RANDOM, ::kodama::Method::Complete);
        let m = WeightedUndirectedAdjacencyMatrixCondensed::from_slice(&RANDOM);
        assert_eq!(m.hierarchical_cluster_complete(), expected);
    }
}
