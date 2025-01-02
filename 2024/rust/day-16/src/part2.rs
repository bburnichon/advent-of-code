use pathfinding::prelude::*;
use std::collections::HashSet;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let map = parse(input).map_err(|err| {
        miette::miette!("Parse error: {}", err)
    })?;

    let result = astar_bag(
        &PosDirection(map.start.clone(), Direction::East),
        |PosDirection(pos, direction)| {
            map.successors(&pos, &direction)
        },
        |PosDirection(p, _)| p.distance(&map.end) / 3,
        |PosDirection(p, _)| *p == map.end,
    )
    .map(|(solution, _cost)| {
        solution.into_iter().fold(
            HashSet::new(),
            |visited, path| {
                path.into_iter().fold(
                    visited,
                    |mut visited, PosDirection(pos, _)| {
                        visited.insert(pos);
                        visited
                    },
                )
            },
        )
    })
    .ok_or_else(|| {
        miette::miette!("Couldn't find a path")
    })?;

    let result = result.len();

    Ok(result.to_string())
}

#[derive(
    Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
struct Pos(i32, i32);

#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
struct PosDirection(Pos, Direction);

impl Pos {
    fn distance(&self, other: &Pos) -> u32 {
        (self.0.abs_diff(other.0)
            + self.1.abs_diff(other.1)) as u32
    }
}

fn parse(input: &str) -> Result<Map, &'static str> {
    let map = input
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars().enumerate().flat_map(
                move |(x, ch)| match ch {
                    '#' | 'S' | 'E' => {
                        Some((Pos(x as i32, y as i32), ch))
                    }
                    _ => None,
                },
            )
        })
        .fold(
            Map {
                walls: Default::default(),
                start: Pos(0, 0),
                end: Pos(0, 0),
            },
            |mut map, (pos, ch)| {
                match ch {
                    '#' => {
                        map.walls.insert(pos);
                    }
                    'S' => map.start = pos,
                    'E' => map.end = pos,
                    _ => unreachable!(),
                }
                map
            },
        );

    Ok(map)
}

struct Map {
    walls: HashSet<Pos>,
    start: Pos,
    end: Pos,
}

impl Map {
    fn successors(
        &self,
        pos: &Pos,
        direction: &Direction,
    ) -> Vec<(PosDirection, u32)> {
        let (x, y) = (pos.0, pos.1);
        let next_pos = match direction {
            Direction::North => Pos(x, y - 1),
            Direction::East => Pos(x + 1, y),
            Direction::South => Pos(x, y + 1),
            Direction::West => Pos(x - 1, y),
        };

        let mut successors = Vec::new();
        if !self.walls.contains(&next_pos) {
            successors.push((
                PosDirection(next_pos, *direction),
                1,
            ))
        }
        for new_direction in match direction {
            Direction::North | Direction::South => {
                [Direction::East, Direction::West]
            }
            Direction::East | Direction::West => {
                [Direction::North, Direction::South]
            }
        } {
            successors.push((
                PosDirection(
                    pos.clone(),
                    new_direction.clone(),
                ),
                1000,
            ))
        }

        successors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        "###############
#.......#....E#
#.#.###.#.###.#
#.....#.#...#.#
#.###.#####.#.#
#.#.#.......#.#
#.#.#####.###.#
#...........#.#
###.#.#####.#.#
#...#.....#.#.#
#.#.#.###.#.#.#
#.....#...#.#.#
#.###.#.#.#.#.#
#S..#.....#...#
###############",
        "45"
    )]
    #[case(
        "#################
#...#...#...#..E#
#.#.#.#.#.#.#.#.#
#.#.#.#...#...#.#
#.#.#.#.###.#.#.#
#...#.#.#.....#.#
#.#.#.#.#.#####.#
#.#...#.#.#.....#
#.#.#####.#.###.#
#.#.#.......#...#
#.#.###.#####.###
#.#.#...#.....#.#
#.#.#.#####.###.#
#.#.#.........#.#
#.#.#.#########.#
#S#.............#
#################",
        "64"
    )]
    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
