use glam::IVec2;
use itertools::{repeat_n, Itertools};
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

    let mut step = 0;
    while !robots
        .iter()
        .map(|robot| robot.position)
        .all_unique()
    {
        move_robots(&mut robots, 1);
        step += 1;
    }

    println!("Final state:");
    show_bots_on_grid(&robots);

    Ok(step.to_string())
}

#[derive(Debug)]
struct Robot {
    position: IVec2,
    velocity: IVec2,
}

impl Robot {
    fn move_step(&mut self, steps: usize) {
        self.position = (self.position
            + (steps as i32) * self.velocity)
            .rem_euclid(MAP_SIZE);
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
        assert_eq!("1", process(input)?);
        Ok(())
    }
}
