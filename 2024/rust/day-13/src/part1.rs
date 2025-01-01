use glam::UVec2;
use nom::{
    bytes::complete::tag,
    character::{
        complete,
        complete::{line_ending, one_of},
    },
    combinator::{all_consuming, opt},
    multi::{many1, separated_list1},
    sequence::{preceded, separated_pair, terminated},
    IResult,
};
use pathfinding::prelude::*;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, machines) = parse(input).map_err(|err| {
        miette::miette!("Parse error: {}", err)
    })?;

    let result = machines
        .iter()
        .flat_map(Machine::minimum_cost_to_go_to_prize)
        .sum::<u32>();

    Ok(result.to_string())
}

#[derive(Debug)]
struct Machine {
    /// per push movement and associated cost
    buttons: Vec<(UVec2, u32)>,
    /// prize position
    prize: UVec2,
}

impl Machine {
    const MAX_PUSHES: usize = 100;

    fn parse(input: &str) -> IResult<&str, Machine> {
        let (input, (buttons, prize)) = separated_pair(
            separated_list1(
                line_ending,
                preceded(
                    tag("Button "),
                    separated_pair(
                        one_of("AB"),
                        tag(": X+"),
                        separated_pair(
                            complete::u32,
                            tag(", Y+"),
                            complete::u32,
                        ),
                    ),
                ),
            ),
            line_ending,
            preceded(
                tag("Prize: X="),
                separated_pair(
                    complete::u32,
                    tag(", Y="),
                    complete::u32,
                ),
            ),
        )(input)?;

        let uvec2_from_pair = |(x, y)| UVec2::new(x, y);
        let buttons = buttons
            .into_iter()
            .map(|(label, step)| {
                (
                    uvec2_from_pair(step),
                    match label {
                        'A' => 3u32,
                        'B' => 1u32,
                        _ => unreachable!(
                            "only A and B are valid labels"
                        ),
                    },
                )
            })
            .collect::<Vec<_>>();

        let prize = uvec2_from_pair(prize);

        Ok((input, Machine { buttons, prize }))
    }

    fn minimum_cost_to_go_to_prize(&self) -> Option<u32> {
        let pushes = vec![0; self.buttons.len()];

        astar(
            &(UVec2::ZERO, pushes),
            |(pos, pushes)| {
                self.buttons
                    .iter()
                    .enumerate()
                    .flat_map(|(button, (dir, cost))| {
                        if pushes[button]
                            >= Self::MAX_PUSHES
                        {
                            return None;
                        }

                        let next_pos = pos + dir;
                        self.prize
                            .cmpge(next_pos)
                            .all()
                            .then(|| {
                                let mut pushes =
                                    pushes.clone();
                                pushes[button] += 1;
                                ((next_pos, pushes), *cost)
                            })
                    })
                    .collect::<Vec<_>>()
            },
            |(pos, _pushes)| {
                (self.prize.x.abs_diff(pos.x)
                    + self.prize.y.abs_diff(pos.y))
                    / 3
            },
            |(pos, _pushes)| pos.cmpeq(self.prize).all(),
        )
        .map(|(_path, cost)| cost)
    }
}

fn parse(input: &str) -> IResult<&str, Vec<Machine>> {
    all_consuming(terminated(
        separated_list1(many1(line_ending), Machine::parse),
        opt(line_ending),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "Button A: X+94, Y+34
Button B: X+22, Y+67
Prize: X=8400, Y=5400

Button A: X+26, Y+66
Button B: X+67, Y+21
Prize: X=12748, Y=12176

Button A: X+17, Y+86
Button B: X+84, Y+37
Prize: X=7870, Y=6450

Button A: X+69, Y+23
Button B: X+27, Y+71
Prize: X=18641, Y=10279";
        assert_eq!("480", process(input)?);
        Ok(())
    }
}
