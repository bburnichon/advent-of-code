use glam::IVec2;
use itertools::{chain, Itertools};
use std::{collections::HashMap, iter::successors};

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let mut grid_size = IVec2::default();

    let antennas = input
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            grid_size.y += 1;
            grid_size.x = line.len() as i32;

            line.chars().enumerate().filter_map(
                move |(x, ch)| {
                    if ch != '.' {
                        Some((
                            ch,
                            IVec2::new(x as i32, y as i32),
                        ))
                    } else {
                        None
                    }
                },
            )
        })
        .fold(
            HashMap::new(),
            |mut acc: HashMap<_, Vec<_>>,
             (ch, position)| {
                acc.entry(ch).or_default().push(position);
                acc
            },
        );

    let in_bound_check = |position: &IVec2| {
        position.cmpge(IVec2::ZERO).all()
            && position.cmplt(grid_size).all()
    };

    let antinodes: usize = antennas
        .iter()
        .flat_map(|(_ch, positions)| {
            positions
                .iter()
                .combinations(2)
                .flat_map(|pos| {
                    let diff = *pos[1] - *pos[0];

                    chain!(
                        successors(
                            Some(*pos[0]),
                            move |pos| {
                                in_bound_check(pos)
                                    .then(|| pos - diff)
                            },
                        ),
                        successors(
                            Some(*pos[1]),
                            move |pos| {
                                in_bound_check(pos)
                                    .then(|| pos + diff)
                            },
                        ),
                    )
                })
                .filter(in_bound_check)
        })
        .unique()
        .count();

    // 353: too high
    Ok(antinodes.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "............
........0...
.....0......
.......0....
....0.......
......A.....
............
............
........A...
.........A..
............
............";
        assert_eq!("34", process(input)?);
        Ok(())
    }
}
