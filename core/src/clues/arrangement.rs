//! A module containing logic for randomizing the order of clues while minimizing
//! overlap of locations.

use {
    crate::clues::{Clue, Clues},
    rand::{rngs::ThreadRng, seq::SliceRandom},
    std::collections::{HashMap, HashSet, VecDeque},
};

pub struct Arrangements {
    inner: Vec<Clues>,
}

impl Arrangements {
    pub fn new(clues: Clues) -> Self {
        let n_clues = clues.0.len();

        let mut clues_by_location: HashMap<String, Vec<Clue>> = HashMap::new();
        for clue in clues.0 {
            let location = clue.location.clone();
            let entry = clues_by_location.entry(location);
            let list = entry.or_default();
            list.push(clue);
        }

        // TODO: should be dynamic
        let n_arrangements = 4;

        // Start with an arrangement that never repeats two locations in a row then
        // create new arrangements from the base arrangement by choosing a clue at a different
        // location from what was chosen in the other arrangements.
        let mut rng = rand::rng();
        let base_arrangement = create_arrangement(&clues_by_location, n_clues, &mut rng);

        let mut used_locations = HashSet::new();

        let mut n_attempts = 0;
        let arrangements = 'outer: loop {
            let mut arrangements: Vec<Vec<Clue>> =
                fill_vec(n_arrangements, || Vec::with_capacity(n_clues));
            // Create a separate queues for each arrangement to draw from
            let mut clue_queues: Vec<VecDeque<Clue>> = Vec::with_capacity(n_arrangements);
            clue_queues.push(base_arrangement.clone().into());
            for _ in 1..n_arrangements {
                let mut q = base_arrangement.clone();
                q.shuffle(&mut rng);
                clue_queues.push(q.into());
            }

            for _ in 0..n_clues {
                used_locations.clear();
                let clue = clue_queues[0].pop_front().unwrap();
                used_locations.insert(clue.location.clone());
                arrangements[0].push(clue);
                for i in 1..n_arrangements {
                    let previous_location = arrangements[i].last().map(|c| c.location.clone());
                    // Find a clue with a new location, if possible
                    let clue = if clue_queues[i].iter().all(|c| {
                        used_locations.contains(&c.location)
                            || Some(&c.location) == previous_location.as_ref()
                    }) {
                        // Try again to pick arrangements that never repeat locations.
                        n_attempts += 1;
                        if n_attempts == 1_000 {
                            panic!(
                                "Failed to generate other arrangements without repeating locations"
                            );
                        }
                        continue 'outer;
                    } else {
                        loop {
                            let clue = clue_queues[i].pop_front().unwrap();
                            if !used_locations.contains(&clue.location)
                                && Some(&clue.location) != previous_location.as_ref()
                            {
                                break clue;
                            }
                            clue_queues[i].push_back(clue);
                        }
                    };
                    used_locations.insert(clue.location.clone());
                    arrangements[i].push(clue);
                }
            }
            break arrangements;
        };

        Self {
            inner: arrangements.into_iter().map(Clues).collect(),
        }
    }

    pub fn iterator(self) -> impl Iterator<Item = Clues> {
        let mut rng = rand::rng();
        let random_clues = {
            let mut base = self.inner.first().unwrap().clone();
            move || {
                base.0.shuffle(&mut rng);
                base.clone()
            }
        };

        // Start with the known arrangements, then give random orders.
        self.inner
            .into_iter()
            .chain(std::iter::repeat_with(random_clues))
    }
}

fn create_arrangement(
    clues_by_location: &HashMap<String, Vec<Clue>>,
    n_clues: usize,
    rng: &mut ThreadRng,
) -> Vec<Clue> {
    let mut n_attempts = 0;
    'outer: loop {
        let mut locations: Vec<String> = clues_by_location.keys().cloned().collect();
        let mut clues_by_location = clues_by_location.clone();

        let mut arrangement: Vec<Clue> = Vec::with_capacity(n_clues);
        while arrangement.len() < n_clues {
            let previous_location = arrangement.last().map(|c| c.location.clone());
            let remaining_locations = clues_by_location
                .iter()
                .filter_map(|(location, clues)| {
                    let location = Some(location);
                    if clues.is_empty() || location == previous_location.as_ref() {
                        None
                    } else {
                        location
                    }
                })
                .count();
            if remaining_locations == 0 {
                n_attempts += 1;
                if n_attempts == 1_000 {
                    panic!("Failed to find arrangement that does not repeat locations!");
                }
                continue 'outer;
            }
            locations.shuffle(rng);
            for location in locations.iter() {
                let list = clues_by_location.get_mut(location).unwrap();
                if let Some(clue) = list.pop() {
                    match arrangement.last().map(|c| &c.location) {
                        Some(ll) if ll == location => list.push(clue),
                        _ => arrangement.push(clue),
                    }
                }
            }
        }
        break arrangement;
    }
}

fn fill_vec<T, F>(size: usize, filler: F) -> Vec<T>
where
    F: Fn() -> T,
{
    let mut result = Vec::with_capacity(size);
    for _ in 0..size {
        result.push(filler());
    }
    result
}

#[test]
fn test_arrangements() {
    fn are_same_clues(x: &Clues, y: &Clues) -> bool {
        let mut a = x.0.clone();
        a.sort_by_key(|c| c.poem.clone());

        let mut b = y.0.clone();
        b.sort_by_key(|c| c.poem.clone());

        a == b
    }

    let clues = Clues(vec![
        Clue::mock(0, "A"),
        Clue::mock(1, "B"),
        Clue::mock(2, "C"),
        Clue::mock(3, "D"),
        Clue::mock(4, "D"),
        Clue::mock(5, "D"),
        Clue::mock(6, "E"),
        Clue::mock(7, "E"),
        Clue::mock(8, "F"),
        Clue::mock(9, "F"),
        Clue::mock(10, "G"),
        Clue::mock(11, "G"),
        Clue::mock(12, "H"),
        Clue::mock(13, "H"),
    ]);

    let arrangements: Vec<Clues> = Arrangements::new(clues.clone())
        .iterator()
        .take(4)
        .collect();
    for arrangement in &arrangements {
        assert!(
            are_same_clues(arrangement, &clues),
            "Arrangements should have same clues as input"
        );
        assert!(
            arrangement
                .0
                .iter()
                .zip(arrangement.0.iter().skip(1))
                .all(|(x, y)| x.location != y.location),
            "Arrangements never repeat the same location twice in a row: {:?}",
            arrangement
                .0
                .iter()
                .map(|c| (&c.location, &c.poem))
                .collect::<Vec<_>>(),
        );
    }
    for i in 0..clues.0.len() {
        let mut used_locations = HashSet::new();
        for arrangement in &arrangements {
            used_locations.insert(&arrangement.0[i].location);
        }
        assert_eq!(
            used_locations.len(),
            arrangements.len(),
            "First few arrangements don't send multiple teams to the same location"
        );
    }
}
