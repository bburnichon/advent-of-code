use std::cmp::min;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let mut max_block = 0;
    let lengths = input
        .trim_end()
        .chars()
        .enumerate()
        .map(|(i, ch)| {
            let len =
                ch.to_digit(10).unwrap_or_else(|| {
                    // clippy does not like expect inside
                    // closure
                    panic!("'{}' is not a digit", ch)
                });
            if i % 2 == 0 {
                max_block += len;
            }
            len
        })
        .collect::<Vec<_>>();
    let max_block: usize = max_block as usize;

    let compacted = lengths
        .iter()
        .enumerate()
        .flat_map(|(i, len)| {
            if i % 2 == 0 {
                Some((i / 2, *len))
            } else {
                None
            }
        })
        .collect::<Vec<(usize, u32)>>();

    let checksum = lengths
        .iter()
        .enumerate()
        .scan(
            (0usize, compacted),
            |(offset, compacted), (index, len)| {
                let mut len = *len as usize;
                if *offset + len >= max_block {
                    len = max_block - *offset
                };

                if *offset >= max_block {
                    return None;
                }

                // non-moved file
                if index % 2 == 0 {
                    *offset += len;

                    // page-id * length * (first + last) / 2
                    let checksum = index / 2
                        * (len * (2 * *offset - len - 1)
                            / 2);

                    return Some(checksum);
                }

                let mut checksum = 0usize;

                while len > 0 {
                    let (file_id, mut block_len) =
                        compacted.pop().expect(
                            "compacted can not be empty",
                        );

                    let consume_len =
                        min(block_len as usize, len);

                    *offset += consume_len;

                    checksum += file_id
                        * (consume_len
                            * (2 * *offset
                                - consume_len
                                - 1))
                        / 2;
                    len -= consume_len;
                    block_len -= consume_len as u32;
                    if block_len > 0 {
                        compacted
                            .push((file_id, block_len));
                    }
                }

                Some(checksum)
            },
        )
        .sum::<usize>();

    Ok(checksum.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "2333133121414131402";
        assert_eq!("1928", process(input)?);
        Ok(())
    }
}
