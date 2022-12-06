// Based on: https://stackoverflow.com/questions/6737283/weighted-randomness-in-java

use std::{fmt::Debug,collections::{BTreeMap, BTreeSet}};
use ordered_float::OrderedFloat;
use rand::Rng;

pub struct Distribution<T> {
    distro: BTreeMap<OrderedFloat<f64>, T>,
    total_weight: f64,
    originals: BTreeMap<T, f64>
}

impl <T:Clone + PartialEq + Eq + PartialOrd + Ord + Debug> Distribution<T> {
    pub fn new() -> Self {
        Distribution {distro: BTreeMap::new(), total_weight: 0.0, originals: BTreeMap::new()}
    }

    pub fn add(&mut self, value: &T, weight: f64) {
        assert!(weight > 0.0);
        self.distro.insert(OrderedFloat(self.total_weight), value.clone());
        self.total_weight += weight;
        self.originals.insert(value.clone(), weight);
    }

    pub fn random_pick(&self) -> T {
        let mut rng = rand::thread_rng();
        let key_picked = closest_key_below(&self.distro, rng.gen_range(0.0..self.total_weight));
        self.distro.get(&key_picked.unwrap()).unwrap().clone()
    }

    pub fn without(&self, removals: BTreeSet<T>) -> Self {
        let mut result = Distribution::new();
        for (value, weight) in self.originals.iter() {
            if !removals.contains(value) {
                result.add(value, *weight);
            }
        }
        result
    }
}

fn closest_key_below<T>(tree: &BTreeMap<OrderedFloat<f64>, T>, target: f64) -> Option<OrderedFloat<f64>> {
    tree.range(..=OrderedFloat(target)).rev().next().map(|(k,_)| *k)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use ordered_float::OrderedFloat;
    use crate::{closest_key_below, Distribution};
    use hash_histogram::HashHistogram;

    fn input_data() -> BTreeMap<OrderedFloat<f64>, String> {
        [(1.0, "a"), (0.5, "b"), (3.5, "c"), (4.8, "d")]
            .iter()
            .map(|(k, v)| (OrderedFloat(*k), v.to_string()))
            .collect()
    }

    fn example_dist() -> Distribution<String> {
        let data = input_data();
        let mut dist = Distribution::new();
        for (weight, value) in data.iter() {
            dist.add(value, weight.into_inner());
        }
        dist
    }
 
    #[test]
    fn test_closest_key_below() {
        let t = input_data();
        assert_eq!(closest_key_below(&t, 0.6).unwrap(), 0.5);
        assert_eq!(closest_key_below(&t, 1.0).unwrap(), 1.0);
        assert_eq!(closest_key_below(&t, 1.00001).unwrap(), 1.0);
        assert_eq!(closest_key_below(&t, 10.0).unwrap(), 4.8);
        assert_eq!(closest_key_below(&t, 4.1).unwrap(), 3.5);
        assert_eq!(closest_key_below(&t, 3.4).unwrap(), 1.0);
    }

    #[test]
    fn general_weight_test() {
        let dist = example_dist();
        let matched = num_match_target(&dist, 20, 200, vec!["d".to_owned(), "c".to_owned(), "a".to_owned(), "b".to_owned()]);
        assert!(matched >= 16);
    }

    fn num_match_target(dist: &Distribution<String>, num_exprs: usize, num_picks: usize, expected: Vec<String>) -> usize {
        let mut matched = 0;
        for _ in 0..num_exprs {
            let mut counts = HashHistogram::new();
            for _ in 0..num_picks {
                counts.bump(&dist.random_pick());
            }
            let ordered = counts.ranking();
            if ordered == expected {
                matched += 1;
            }
        }
        matched
    }

    #[test]
    fn test_without() {
        let dist = example_dist();
        let dist = dist.without(["a".to_owned(), "c".to_owned()].iter().cloned().collect());
        let matched = num_match_target(&dist, 20, 200, vec!["d".to_owned(), "b".to_owned()]);
        assert_eq!(matched, 20);
    }
}