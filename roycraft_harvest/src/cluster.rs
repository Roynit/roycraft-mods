use std::collections::{HashSet, VecDeque};

pub type Pos = [i32; 3];

pub fn neighbors(p: Pos) -> impl Iterator<Item = Pos> {
    [(0, 1), (0, -1), (1, 1), (1, -1), (2, 1), (2, -1)]
        .into_iter()
        .filter_map(move |(axis, step)| {
            let mut next = p;
            next[axis] = next[axis].checked_add(step)?;
            Some(next)
        })
}

pub fn cluster(
    origin: Pos,
    limit: usize,
    radius: u32,
    mut matching: impl FnMut(Pos) -> bool,
) -> Vec<Pos> {
    let mut queue = VecDeque::from([origin]);
    let mut visited = HashSet::from([origin]);
    let mut found = Vec::new();
    while let Some(pos) = queue.pop_front() {
        for next in neighbors(pos) {
            if next.iter().zip(origin).any(|(a, b)| a.abs_diff(b) > radius)
                || !visited.insert(next)
                || !matching(next)
            {
                continue;
            }
            if found.len() == limit {
                return found;
            }
            found.push(next);
            queue.push_back(next);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traverses_connected_blocks_only_and_respects_caps() {
        let blocks = [[1, 0, 0], [2, 0, 0], [5, 0, 0], [0, 1, 1]];
        let run = |limit, radius| cluster([0; 3], limit, radius, |p| blocks.contains(&p));
        assert_eq!(run(32, 16), [[1, 0, 0], [2, 0, 0]]);
        assert_eq!(run(1, 16), [[1, 0, 0]]);
        assert_eq!(run(32, 1), [[1, 0, 0]]);
        assert_eq!(neighbors([i32::MAX, 0, 0]).count(), 5);
    }
}
