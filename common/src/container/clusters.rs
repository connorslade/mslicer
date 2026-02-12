use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

// look into union find data structure for faster cluster merging.

#[derive(Default)]
pub struct Clusters<T: Hash + PartialEq + Eq + Copy> {
    runs: HashMap<T, u32>,              // maps runs to clusters
    clusters: HashMap<u32, HashSet<T>>, // maps clusters to runs
    next_id: u32,
}

impl<T: Hash + PartialEq + Eq + Copy> Clusters<T> {
    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn runs(&self, cluster: u32) -> &HashSet<T> {
        &self.clusters[&cluster]
    }

    pub fn cluster(&self, run: T) -> u32 {
        self.runs[&run]
    }

    pub fn clusters(&self) -> impl Iterator<Item = (&u32, &HashSet<T>)> {
        self.clusters.iter()
    }

    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }

    // Finds the cluster that contains a given run, creating a new one of
    // needed.
    pub fn get_cluster(&mut self, run: T) -> u32 {
        if let Some(&cluster) = self.runs.get(&run) {
            return cluster;
        }

        let cluster = self.next_id();
        self.runs.insert(run, cluster);
        self.clusters.entry(cluster).or_default().insert(run);
        cluster
    }

    pub fn mark_adjacency(&mut self, a: T, b: T) {
        // Find (or make) cluster id's for each run.
        let a = self.get_cluster(a);
        let b = self.get_cluster(b);
        if a == b {
            return;
        }

        // Merge both clusters by moving all the runs in cluster b into cluster a.
        for run in &self.clusters[&b] {
            *self.runs.get_mut(run).unwrap() = a;
        }

        let old = self.clusters.remove(&b).unwrap();
        self.clusters.get_mut(&a).unwrap().extend(old);
    }
}
