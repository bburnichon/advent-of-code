use glam::IVec2;
use itertools::repeat_n;
use nom::{
    bytes::complete::tag,
    character::{complete, complete::line_ending},
    combinator::{all_consuming, opt},
    multi::separated_list1,
    sequence::{preceded, separated_pair, terminated},
    IResult,
};

#[cfg(test)]
const MAP_SIZE: IVec2 = IVec2::new(11, 7);
#[cfg(not(test))]
const MAP_SIZE: IVec2 = IVec2::new(101, 103);

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, mut robots) = parse(input).map_err(|err| {
        miette::miette!("Parse error: {}", err)
    })?;

    println!("Initial state:");
    show_bots_on_grid(&robots);

    move_robots(&mut robots, 100);

    let west_range = 0..(MAP_SIZE.x - 1) / 2;
    let east_range = ((MAP_SIZE.x - 1) / 2 + 1)..MAP_SIZE.x;
    let north_range = 0..(MAP_SIZE.y - 1) / 2;
    let south_range = (MAP_SIZE.y - 1) / 2 + 1..MAP_SIZE.y;

    let result = robots
        .iter()
        .fold([0; 4], |mut acc, robot| {
            let west =
                west_range.contains(&robot.position.x);
            let east =
                east_range.contains(&robot.position.x);
            let north =
                north_range.contains(&robot.position.y);
            let south =
                south_range.contains(&robot.position.y);

            match (west, east, north, south) {
                (true, false, true, false) => acc[0] += 1,
                (false, true, true, false) => acc[1] += 1,
                (true, false, false, true) => acc[2] += 1,
                (false, true, false, true) => acc[3] += 1,
                (_, _, _, _) => {}
            }

            acc
        })
        .iter()
        .product::<isize>();

    Ok(result.to_string())
}

#[derive(Debug)]
struct Robot {
    position: IVec2,
    velocity: IVec2,
}

impl Robot {
    fn move_step(&mut self, steps: usize) {
        self.position.x = (self.position.x
            + (steps as i32) * self.velocity.x)
            .checked_rem_euclid(MAP_SIZE.x)
            .expect("Too many steps");
        self.position.y = (self.position.y
            + (steps as i32) * self.velocity.y)
            .checked_rem_euclid(MAP_SIZE.y)
            .expect("Too many steps");
    }
}

fn show_bots_on_grid(robots: &[Robot]) {
    let mut grid: Vec<Vec<char>> = repeat_n(
        repeat_n('.', MAP_SIZE.x as usize).collect(),
        MAP_SIZE.y as usize,
    )
    .collect();

    for robot in robots {
        grid[robot.position.y as usize]
            [robot.position.x as usize] = '#';
    }

    for line in grid.iter() {
        println!("{}", line.iter().collect::<String>());
    }
}

fn move_robots(robots: &mut Vec<Robot>, steps: usize) {
    for robot in robots.iter_mut() {
        robot.move_step(steps);
    }
}

fn parse(input: &str) -> IResult<&str, Vec<Robot>> {
    let (input, robots) = all_consuming(terminated(
        separated_list1(
            line_ending,
            separated_pair(
                preceded(
                    tag("p="),
                    separated_pair(
                        complete::i32,
                        tag(","),
                        complete::i32,
                    ),
                ),
                tag(" "),
                preceded(
                    tag("v="),
                    separated_pair(
                        complete::i32,
                        tag(","),
                        complete::i32,
                    ),
                ),
            ),
        ),
        opt(line_ending),
    ))(input)?;

    let robots = robots
        .into_iter()
        .map(|robot| Robot {
            position: IVec2::new(robot.0 .0, robot.0 .1),
            velocity: IVec2::new(robot.1 .0, robot.1 .1),
        })
        .collect();

    Ok((input, robots))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "p=0,4 v=3,-3
p=6,3 v=-1,-3
p=10,3 v=-1,2
p=2,0 v=2,-1
p=0,0 v=1,3
p=3,0 v=-2,-2
p=7,6 v=-1,-3
p=3,0 v=-1,-2
p=9,3 v=2,3
p=7,3 v=-1,2
p=2,4 v=2,-3
p=9,5 v=-3,-3";
        assert_eq!("12", process(input)?);
        Ok(())
    }
}
