use glam::IVec2;
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
use std::collections::HashMap;

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
        .find_map(|(position, object)| match object {
            Object::Bot => Some(position),
            _ => None,
        })
        .copied()
        .ok_or_else(|| {
            miette::miette!("Could not find bot")
        })?;

    for direction in directions {
        let new_bot_position =
            grid.try_move(bot_position, direction);
        bot_position =
            new_bot_position.ok_or_else(|| {
                miette::miette!(
                    "The bot was lost in the process"
                )
            })?;
    }

    let result = grid
        .objects
        .iter()
        .filter_map(|(position, object)| {
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

impl Object {
    fn try_move(
        &self,
        grid: &mut Grid,
        position: IVec2,
        direction: Direction,
    ) -> IVec2 {
        match self {
            Object::Wall => position,
            _ => {
                let try_position =
                    position + IVec2::from(direction);
                let next_position = match grid
                    .try_move(try_position, direction)
                {
                    // An unmovable object was found
                    Some(new_position)
                        if new_position == try_position =>
                    {
                        position
                    }
                    _ => try_position,
                };
                next_position
            }
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

impl From<Direction> for IVec2 {
    fn from(dir: Direction) -> Self {
        use Direction::*;

        match dir {
            North => IVec2::new(0, -1),
            West => IVec2::new(-1, 0),
            South => IVec2::new(0, 1),
            East => IVec2::new(1, 0),
        }
    }
}

struct Grid {
    objects: HashMap<IVec2, Object>,
}

impl Grid {
    /// Try moving object at position in the
    /// direction and returns new position
    fn try_move(
        &mut self,
        position: IVec2,
        direction: Direction,
    ) -> Option<IVec2> {
        let object = self.objects.remove(&position)?;
        let next_position =
            object.try_move(self, position, direction);
        self.objects.insert(next_position, object);

        Some(next_position)
    }
}

impl Grid {
    fn parse(input: Span) -> IResult<Span, Self> {
        let (input, grid) = many1(terminated(
            Self::object_kind_and_location,
            alt((line_ending, take_while(|ch| ch == '.'))),
        ))(input)?;

        let grid = Grid {
            objects: HashMap::from_iter(
                grid.into_iter().flat_map(
                    |(kind, position)| {
                        let object = match kind {
                            '#' => Some(Object::Wall),
                            'O' => Some(Object::Box),
                            '@' => Some(Object::Bot),
                            _ => None,
                        };
                        object.map(|object| {
                            (position, object)
                        })
                    },
                ),
            ),
        };

        Ok((input, grid))
    }

    fn object_kind_and_location(
        input: Span,
    ) -> IResult<Span, (char, IVec2)> {
        let (input, position) = position(input)?;
        let (input, kind) = one_of("#O@")(input)?;

        let x = position.get_column() as i32 - 1;
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
    #[case(
        "########
#..O.O.#
##@.O..#
#...O..#
#.#.O..#
#...O..#
#......#
########

<^^>>>vv<v>>v<<",
        "2028"
    )]
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
v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^", "10092")]
    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
