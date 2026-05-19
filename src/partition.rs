use crate::patch::PatchId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Partition {
    pub id: usize,
    pub patch_ids: Vec<PatchId>,
}

#[derive(Clone, Debug)]
pub struct PartitionMap {
    partitions: Vec<Partition>,
    owners: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BoundaryMetrics {
    pub total_edges: usize,
    pub local_edges: usize,
    pub boundary_edges: usize,
    pub boundary_fraction: f64,
}

impl PartitionMap {
    pub fn contiguous(total_patches: usize, workers: usize) -> Self {
        let worker_count = workers.max(1).min(total_patches.max(1));
        let mut partitions = Vec::with_capacity(worker_count);
        let mut owners = vec![0; total_patches];
        let base = total_patches / worker_count;
        let remainder = total_patches % worker_count;
        let mut next_patch = 0;

        for worker_id in 0..worker_count {
            let size = base + usize::from(worker_id < remainder);
            let mut patch_ids = Vec::with_capacity(size);
            for patch_index in next_patch..next_patch + size {
                owners[patch_index] = worker_id;
                patch_ids.push(PatchId(patch_index));
            }
            next_patch += size;
            partitions.push(Partition {
                id: worker_id,
                patch_ids,
            });
        }

        Self { partitions, owners }
    }

    pub fn partitions(&self) -> &[Partition] {
        &self.partitions
    }

    pub fn owner(&self, patch_id: PatchId) -> Option<usize> {
        self.owners.get(patch_id.0).copied()
    }

    pub fn worker_count(&self) -> usize {
        self.partitions.len()
    }

    pub fn patch_count(&self) -> usize {
        self.owners.len()
    }

    pub fn boundary_metrics(&self, connectivity: &[Vec<(usize, f64)>]) -> BoundaryMetrics {
        let mut metrics = BoundaryMetrics::default();

        for (source, edges) in connectivity.iter().enumerate() {
            let source_owner = self.owners[source];
            for (destination, _) in edges {
                metrics.total_edges += 1;
                if self.owners[*destination] == source_owner {
                    metrics.local_edges += 1;
                } else {
                    metrics.boundary_edges += 1;
                }
            }
        }

        if metrics.total_edges > 0 {
            metrics.boundary_fraction = metrics.boundary_edges as f64 / metrics.total_edges as f64;
        }

        metrics
    }

    #[cfg(test)]
    pub fn patch_ids(&self) -> impl Iterator<Item = PatchId> + '_ {
        self.partitions
            .iter()
            .flat_map(|partition| partition.patch_ids.iter().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::PartitionMap;
    use crate::patch::PatchId;

    #[test]
    fn contiguous_partitions_cover_each_patch_once() {
        let map = PartitionMap::contiguous(10, 3);
        let patch_ids = map.patch_ids().map(|id| id.0).collect::<Vec<_>>();

        assert_eq!(patch_ids, (0..10).collect::<Vec<_>>());
        assert_eq!(map.owner(PatchId(0)), Some(0));
        assert_eq!(map.owner(PatchId(9)), Some(2));
    }

    #[test]
    fn boundary_metrics_count_cross_partition_edges() {
        let map = PartitionMap::contiguous(4, 2);
        let connectivity = vec![vec![(1, 1.0)], vec![(2, 1.0)], vec![(3, 1.0)], vec![]];
        let metrics = map.boundary_metrics(&connectivity);

        assert_eq!(metrics.total_edges, 3);
        assert_eq!(metrics.boundary_edges, 1);
        assert!((metrics.boundary_fraction - (1.0 / 3.0)).abs() < 1.0e-12);
    }
}
