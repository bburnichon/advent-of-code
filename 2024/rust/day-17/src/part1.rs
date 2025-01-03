use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::{
        complete,
        complete::{anychar, line_ending},
    },
    combinator::opt,
    multi::{many_m_n, separated_list1},
    sequence::{delimited, separated_pair},
    IResult,
};

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, computer) = parse(input).map_err(|err| {
        miette::miette!("Parse error: {}", err)
    })?;

    let result = computer.flatten().join(",");

    Ok(result.to_string())
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
enum Opcode {
    /// The adv instruction (opcode 0) performs
    /// division. The numerator is the value in
    /// the A register. The denominator is found
    /// by raising 2 to the power of the
    /// instruction's combo operand. (So, an
    /// operand of 2 would divide A by 4 (2^2); an
    /// operand of 5 would divide A by 2^B.) The
    /// result of the division operation is
    /// truncated to an integer and then written
    /// to the A register.
    Adv,
    /// The bxl instruction (opcode 1) calculates
    /// the bitwise XOR of register B and the
    /// instruction's literal operand, then
    /// stores the result in register B.
    Bxl,

    /// The bst instruction (opcode 2) calculates
    /// the value of its combo operand modulo
    /// 8 (thereby keeping only its lowest 3
    /// bits), then writes that value to the B
    /// register.
    Bst,

    /// The jnz instruction (opcode 3) does
    /// nothing if the A register is 0.
    /// However, if the A register is not
    /// zero, it jumps by setting the instruction
    /// pointer to the value of its literal
    /// operand; if this instruction jumps,
    /// the instruction pointer is not
    /// increased by 2 after this instruction.
    Jnz,

    /// The bxc instruction (opcode 4) calculates
    /// the bitwise XOR of register B and
    /// register C, then stores the result in
    /// register B. (For legacy reasons, this
    /// instruction reads an operand but
    /// ignores it.)
    Bxc,

    /// The out instruction (opcode 5) calculates
    /// the value of its combo operand modulo
    /// 8, then outputs that value. (If a
    /// program outputs multiple values, they
    /// are separated by commas.)
    Out,

    /// The bdv instruction (opcode 6) works
    /// exactly like the adv instruction
    /// except that the result is
    /// stored in the B register. (The numerator
    /// is still read from the A register.)
    Bdv,

    /// The cdv instruction (opcode 7) works
    /// exactly like the adv instruction
    /// except that the result is
    /// stored in the C register. (The numerator
    /// is still read from the A register.)
    Cdv,
}

impl From<u8> for Opcode {
    fn from(code: u8) -> Self {
        use Opcode::*;
        match code {
            0 => Adv,
            1 => Bxl,
            2 => Bst,
            3 => Jnz,
            4 => Bxc,
            5 => Out,
            6 => Bdv,
            7 => Cdv,
            _ => unreachable!(),
        }
    }
}

struct Computer {
    register_a: u32,
    register_b: u32,
    register_c: u32,
    instruction_pointer: usize,

    program: Vec<u8>,
}

impl Iterator for Computer {
    type Item = Option<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.instruction_pointer >= self.program.len() {
            return None;
        }

        let instruction = self
            .program
            .get(self.instruction_pointer)
            .copied()
            .map(Opcode::from)?;

        let operand = self
            .program
            .get(self.instruction_pointer + 1)
            .copied()?;

        let mut out = None;

        // println!("Register A: {}", self.register_a);
        // println!("Register B: {}", self.register_b);
        // println!("Register C: {}", self.register_c);
        // println!(
        //     "Instruction pointer: {}",
        //     self.instruction_pointer
        // );
        //
        // println!("instruction: {}", instruction as
        // u32); println!("Operand: {}", operand);
        // println!("combo: {}", self.combo(operand));

        match &instruction {
            Opcode::Adv => {
                self.adv(operand);
            }
            Opcode::Bxl => {
                self.bxl(operand);
            }
            Opcode::Bst => {
                self.bst(operand);
            }
            Opcode::Jnz => {
                self.jnz(operand);
            }
            Opcode::Bxc => {
                self.bxc(operand);
            }
            Opcode::Out => {
                out = Some(self.out(operand));
            }
            Opcode::Bdv => {
                self.bdv(operand);
            }
            Opcode::Cdv => {
                self.cdv(operand);
            }
        }
        if instruction != Opcode::Jnz {
            self.instruction_pointer += 2;
        }

        Some(out)
    }
}
impl Computer {
    /// There are two types of operands; each
    /// instruction specifies the type of its
    /// operand. The value of a literal operand is
    /// the operand itself. For example, the value
    /// of the literal operand 7 is the number 7.
    /// The value of a combo operand can be found
    /// as follows:
    ///
    /// Combo operands 0 through 3 represent
    /// literal values 0 through 3.
    /// Combo operand 4 represents the value of
    /// register A. Combo operand 5 represents
    /// the value of register B. Combo operand
    /// 6 represents the value of register C.
    /// Combo operand 7 is reserved and will not
    /// appear in valid programs.
    fn combo(&self, operand: u8) -> u32 {
        match operand {
            0..=3 => operand as u32,
            4 => self.register_a,
            5 => self.register_b,
            6 => self.register_c,
            7 => {
                panic!("should not happen in valid program")
            }
            _ => unreachable!(),
        }
    }

    /// The adv instruction (opcode 0) performs
    /// division. The numerator is the value in
    /// the A register. The denominator is found
    /// by raising 2 to the power of the
    /// instruction's combo operand. (So, an
    /// operand of 2 would divide A by 4 (2^2); an
    /// operand of 5 would divide A by 2^B.) The
    /// result of the division operation is
    /// truncated to an integer and then written
    /// to the A register.
    fn adv(&mut self, operand: u8) {
        self.register_a /= 2u32.pow(self.combo(operand));
    }

    /// The bxl instruction (opcode 1) calculates
    /// the bitwise XOR of register B and the
    /// instruction's literal operand, then
    /// stores the result in register B.
    fn bxl(&mut self, operand: u8) {
        self.register_b ^= operand as u32;
    }

    /// The bst instruction (opcode 2) calculates
    /// the value of its combo operand modulo
    /// 8 (thereby keeping only its lowest 3
    /// bits), then writes that value to the B
    /// register.
    fn bst(&mut self, operand: u8) {
        self.register_b = self.combo(operand) % 8;
    }

    /// The jnz instruction (opcode 3) does
    /// nothing if the A register is 0.
    /// However, if the A register is not
    /// zero, it jumps by setting the instruction
    /// pointer to the value of its literal
    /// operand; if this instruction jumps,
    /// the instruction pointer is not
    /// increased by 2 after this instruction.
    fn jnz(&mut self, operand: u8) {
        self.instruction_pointer = if self.register_a == 0 {
            self.instruction_pointer + 2usize
        } else {
            operand as usize
        }
    }

    /// The bxc instruction (opcode 4) calculates
    /// the bitwise XOR of register B and
    /// register C, then stores the result in
    /// register B. (For legacy reasons, this
    /// instruction reads an operand but
    /// ignores it.)
    fn bxc(&mut self, _operand: u8) {
        self.register_b ^= self.register_c;
    }

    /// The out instruction (opcode 5) calculates
    /// the value of its combo operand modulo
    /// 8, then outputs that value. (If a
    /// program outputs multiple values, they
    /// are separated by commas.)
    fn out(&mut self, operand: u8) -> u32 {
        self.combo(operand) % 8
    }

    /// The bdv instruction (opcode 6) works
    /// exactly like the adv instruction
    /// except that the result is
    /// stored in the B register. (The numerator
    /// is still read from the A register.)
    fn bdv(&mut self, operand: u8) {
        self.register_b = self.register_a
            / (2u32.pow(self.combo(operand)));
    }

    /// The cdv instruction (opcode 7) works
    /// exactly like the adv instruction
    /// except that the result is
    /// stored in the C register. (The numerator
    /// is still read from the A register.)
    /// The bdv instruction (opcode 6) works
    /// exactly like the adv instruction
    /// except that the result is
    /// stored in the B register. (The numerator
    /// is still read from the A register.)
    fn cdv(&mut self, operand: u8) {
        self.register_c = self.register_a
            / (2u32.pow(self.combo(operand)));
    }
}

fn parse(input: &str) -> IResult<&str, Computer> {
    let (input, (registers, program)) = separated_pair(
        many_m_n(
            3,
            3,
            delimited(
                tag("Register "),
                separated_pair(
                    anychar,
                    tag(": "),
                    complete::u32,
                ),
                line_ending,
            ),
        ),
        line_ending,
        delimited(
            tag("Program: "),
            separated_list1(tag(","), complete::u8),
            opt(line_ending),
        ),
    )(input)?;

    Ok((
        input,
        Computer {
            register_a: registers[0].1,
            register_b: registers[1].1,
            register_c: registers[2].1,
            instruction_pointer: 0,
            program,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "Register A: 729
Register B: 0
Register C: 0

Program: 0,1,5,4,3,0";
        assert_eq!("4,6,3,5,6,3,5,2,1,0", process(input)?);
        Ok(())
    }
}