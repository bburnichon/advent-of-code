use glam::IVec2;
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{line_ending, one_of},
    combinator::{all_consuming, map, opt},
    multi::{many0, many1, separated_list0},
    sequence::{separated_pair, terminated},
    IResult,
};
use nom_locate::{position, LocatedSpan};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet, VecDeque},
    fmt::{Display, Formatter, Write as _},
};

type Span<'a> = LocatedSpan<&'a str>;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (mut grid, directions)) =
        parse(Span::new(input)).map_err(|err| {
            miette::miette!("Parse error: {}", err)
        })?;

    let mut bot_position = grid
        .objects
        .iter()
        .find_map(|(object, position)| match object {
            Object::Bot => Some(*position),
            _ => None,
        })
        .ok_or_else(|| {
            miette::miette!("Could not find bot")
        })?;

    println!("Initial:\n{}\n", grid);

    for direction in directions {
        //println!("Direction: {:?}\n", direction);
        let new_bot_position =
            grid.try_move(&bot_position, &direction);
        bot_position =
            new_bot_position.ok_or_else(|| {
                miette::miette!(
                    "The bot was lost in the process"
                )
            })?;
        //println!("{}", grid);
    }

    let result = grid
        .objects
        .iter()
        .filter_map(|(object, position)| {
            if let Object::Box = object {
                Some(100 * position.y + position.x)
            } else {
                None
            }
        })
        .sum::<i32>();

    Ok(result.to_string())
}

enum Object {
    Wall,
    Box,
    Bot,
}

impl Display for Object {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Object::Wall => f.write_str("**"),
            Object::Box => f.write_str("[]"),
            Object::Bot => f.write_str("@"),
        }
    }
}

impl Object {
    fn occupies(&self, position: &IVec2) -> Vec<IVec2> {
        let mut occupied = vec![];
        match self {
            Object::Wall | Object::Box => {
                occupied.push(*position);
                occupied.push(*position + IVec2::X);
            }
            Object::Bot => {
                occupied.push(*position);
            }
        }
        occupied
    }

    fn would_move(
        &self,
        position: &IVec2,
        direction: &Direction,
    ) -> Option<Vec<IVec2>> {
        match self {
            Object::Wall => None,
            Object::Box => {
                let mut try_positions = vec![];
                let offset = *direction.as_ivec2();
                match direction {
                    Direction::North | Direction::South => {
                        try_positions.extend_from_slice(&[
                            *position + offset,
                            *position + offset + IVec2::X,
                        ]);
                    }
                    Direction::West => {
                        try_positions.extend_from_slice(&[
                            *position + offset,
                        ]);
                    }
                    Direction::East => {
                        try_positions.extend_from_slice(&[
                            *position + offset + IVec2::X,
                        ]);
                    }
                };
                Some(try_positions)
            }
            Object::Bot => Some(vec![
                *position + *direction.as_ivec2(),
            ]),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    fn as_ivec2(&self) -> &IVec2 {
        use Direction::*;

        match *self {
            North => &IVec2::NEG_Y,
            West => &IVec2::NEG_X,
            South => &IVec2::Y,
            East => &IVec2::X,
        }
    }
}

struct Grid {
    occupied: HashMap<IVec2, usize>,
    objects: Vec<(Object, IVec2)>,
}

impl Display for Grid {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let objects =
            self.objects.iter().sorted_by(|a, b| {
                match a.1.y.cmp(&b.1.y) {
                    Ordering::Equal => a.1.x.cmp(&b.1.x),
                    y => y,
                }
            });

        let mut y = 0;
        let line = &mut String::new();
        for (object, pos) in objects {
            if pos.y != y {
                writeln!(f, "{}", line)?;
                y = pos.y;
                line.clear();
            }
            line.push_str(
                ".".repeat(pos.x as usize - line.len())
                    .as_str(),
            );
            write!(line, "{}", object)?;
        }

        writeln!(f, "{}", line)
    }
}

impl Grid {
    /// Try moving object at position in the
    /// direction and returns new position
    fn try_move(
        &mut self,
        position: &IVec2,
        direction: &Direction,
    ) -> Option<IVec2> {
        let mut visited = HashSet::new();
        let mut moving_order = VecDeque::new();
        let mut to_move = VecDeque::new();

        to_move.push_back(*position);

        while let Some(position) = to_move.pop_front() {
            let Some(index) =
                self.occupied.get(&position).cloned()
            else {
                // nothing at position, can move
                continue;
            };

            // try to insert position, if already there,
            // check was already done
            if !visited.insert(index) {
                continue;
            }

            moving_order.push_front(index);
            let Some(candidates) =
                self.objects[index].0.would_move(
                    &self.objects[index].1,
                    direction,
                )
            else {
                // given object will not move
                moving_order.clear();
                break;
            };

            to_move.extend(candidates);
        }

        // all objects were checked, now move all objects
        if moving_order.is_empty() {
            return if !visited.is_empty() {
                Some(*position)
            } else {
                None
            };
        }

        let mut last_moved_position = None;
        while let Some(index) = moving_order.pop_front() {
            let (to_move, position) =
                &mut self.objects[index];
            for pos in to_move.occupies(&position) {
                self.occupied.remove(&pos);
            }
            *position += *direction.as_ivec2();
            for pos in to_move.occupies(&position) {
                self.occupied.insert(pos, index);
            }

            last_moved_position = Some(*position);
        }

        last_moved_position
    }
}

impl Grid {
    fn parse(input: Span) -> IResult<Span, Self> {
        let (input, grid) = many1(terminated(
            Self::object_kind_and_location,
            alt((line_ending, take_while(|ch| ch == '.'))),
        ))(input)?;

        let object_map =
            Vec::from_iter(grid.into_iter().flat_map(
                |(kind, position)| {
                    match kind {
                        '#' => Some(Object::Wall),
                        'O' => Some(Object::Box),
                        '@' => Some(Object::Bot),
                        _ => None,
                    }
                    .map(|object| (position, object))
                },
            ));

        let grid = object_map.into_iter().enumerate().fold(
            Grid {
                objects: Vec::new(),
                occupied: HashMap::new(),
            },
            |mut grid, (index, (position, object))| {
                for pos in object.occupies(&position) {
                    grid.occupied.insert(pos, index);
                }
                grid.objects.push((object, position));
                grid
            },
        );

        Ok((input, grid))
    }

    fn object_kind_and_location(
        input: Span,
    ) -> IResult<Span, (char, IVec2)> {
        let (input, position) = position(input)?;
        let (input, kind) = one_of("#O@")(input)?;

        // all x coordinates are twice as large
        let x = (position.get_column() as i32 - 1) * 2;
        let y = position.location_line() as i32 - 1;

        Ok((input, (kind, IVec2::new(x, y))))
    }
}

impl Direction {
    fn parse(input: Span) -> IResult<Span, Self> {
        let (input, dir) = one_of("^<v>")(input)?;

        let dir = match dir {
            '^' => Self::North,
            '<' => Self::West,
            'v' => Self::South,
            '>' => Self::East,
            _ => unreachable!("invalid direction"),
        };

        Ok((input, dir))
    }
}

fn parse(
    input: Span,
) -> IResult<Span, (Grid, Vec<Direction>)> {
    let (input, (grid, directions)) =
        all_consuming(separated_pair(
            Grid::parse,
            line_ending,
            terminated(
                map(
                    separated_list0(
                        line_ending,
                        many0(Direction::parse),
                    ),
                    |directions| {
                        directions
                            .into_iter()
                            .flatten()
                            .collect::<Vec<Direction>>()
                    },
                ),
                opt(line_ending),
            ),
        ))(input)?;

    Ok((input, (grid, directions)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("##########
#..O..O.O#
#......O.#
#.OO..O.O#
#..O@..O.#
#O#..O...#
#O..O..O.#
#.OO.O.OO#
#....O...#
##########

<vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
<<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
>^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
<><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^", "9021")]
    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
