use glam::U64Vec2;
use nom::{
    bytes::complete::tag,
    character::{
        complete,
        complete::{line_ending, one_of},
    },
    combinator::{all_consuming, opt},
    multi::{count, many1, separated_list1},
    sequence::{
        delimited, pair, preceded, separated_pair,
        terminated,
    },
    IResult,
};
use std::cmp::Ordering;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, mut machines) =
        parse(input).map_err(|err| {
            miette::miette!("Parse error: {}", err)
        })?;

    for machine in machines.iter_mut() {
        machine.prize += U64Vec2::splat(10000000000000);
    }

    do_process(&machines)
}

fn do_process(
    machines: &[Machine],
) -> miette::Result<String> {
    let result = machines
        .iter()
        .flat_map(Machine::minimum_cost_to_go_to_prize)
        .sum::<u64>();

    Ok(result.to_string())
}

#[derive(Debug)]
struct Machine {
    /// per push movement and associated cost
    buttons: [(U64Vec2, u64); 2],
    /// prize position
    prize: U64Vec2,
}

impl Machine {
    fn parse(input: &str) -> IResult<&str, Machine> {
        let (input, (buttons, prize)) = pair(
            count(
                delimited(
                    tag("Button "),
                    separated_pair(
                        one_of("AB"),
                        tag(": X+"),
                        separated_pair(
                            complete::u64,
                            tag(", Y+"),
                            complete::u64,
                        ),
                    ),
                    line_ending,
                ),
                2,
            ),
            preceded(
                tag("Prize: X="),
                separated_pair(
                    complete::u64,
                    tag(", Y="),
                    complete::u64,
                ),
            ),
        )(input)?;

        let uvec2_from_pair = |(x, y)| U64Vec2::new(x, y);
        let buttons = buttons
            .into_iter()
            .map(|(label, step)| {
                (
                    uvec2_from_pair(step),
                    match label {
                        'A' => 3,
                        'B' => 1,
                        _ => unreachable!(
                            "only A and B are valid labels"
                        ),
                    },
                )
            })
            .collect::<Vec<_>>();

        let prize = uvec2_from_pair(prize);
        let buttons =
            buttons.try_into().expect("2 buttons parsed");

        Ok((input, Machine { buttons, prize }))
    }

    fn minimum_cost_to_go_to_prize(&self) -> Option<u64> {
        // each button adds parts to an equation set:
        // (1) a.x * a + b.x * b = p.x
        // (2) a.y * a + b.y * b = p.y
        //
        // From (1):
        // (3) b = (p.x - a.x * a) / b.x
        //
        // substitute b from (3) in (2) * b.x
        // (4) a.y * b.x * a + b.y * (p.x - a.x * a) = p.y
        // * b.x
        //
        // (5) (a.y * b.x - a.x * b.y) * a = b.x * p.y -
        // b.y * p.x
        //
        // (6) a = (b.x * p.y - b.y * p.x) / (a.y * b.x -
        // a.x * b.y)
        //
        // (7) a = det PB / det AB
        //
        // Equations are exactly the same so:
        //
        // (8) b = det AP / det AB
        //
        // both a and b should be integers for a solution,
        // otherwise lines. When determinant is 0, this
        // means both equations are proportional to each
        // other. And has either no solutions or many. Can
        // be determined by numerator.
        //
        // Cost equation:
        // a.c * a + b.c * b = p.c which has exactly the
        // same form as the other equations

        let det_ab = Self::determinant(
            &self.buttons[0].0,
            &self.buttons[1].0,
        );
        let det_pb = Self::determinant(
            &self.prize,
            &self.buttons[1].0,
        );
        let det_ap = Self::determinant(
            &self.buttons[0].0,
            &self.prize,
        );

        let a = (det_pb.1 == det_ab.1).then_some(det_pb.0);
        let b = (det_ap.1 == det_ab.1).then_some(det_ap.0);

        match (det_ab.0, a, b) {
            (_, None, _) => None,
            (_, _, None) => None,
            (0, Some(0), Some(0)) => None, /* should not */
            // happen
            (0, _, _) => panic!(
                "both button have only a different size"
            ),
            (d, Some(a), Some(b))
                if a % d == 0 && b % d == 0 =>
            {
                Some(
                    a / d * self.buttons[0].1
                        + b / d * self.buttons[1].1,
                )
            }
            _ => None,
        }
    }

    fn determinant(
        a: &U64Vec2,
        b: &U64Vec2,
    ) -> (u64, bool) {
        let neg_diagonal = a.x * b.y;
        let pos_diagonal = a.y * b.x;

        match neg_diagonal.cmp(&pos_diagonal) {
            Ordering::Less => {
                (pos_diagonal - neg_diagonal, true)
            }
            Ordering::Equal => (0, false),
            Ordering::Greater => {
                (neg_diagonal - pos_diagonal, false)
            }
        }
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
        let (_, machines) =
            parse(input).expect("input should parse");
        assert_eq!("480", do_process(machines.as_slice())?);
        Ok(())
    }
}
