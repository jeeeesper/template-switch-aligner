use std::{fmt::Display, io::Write};

use nom::{AsChar, IResult};
use num_traits::PrimInt;

use super::CostFunction;

use crate::{
    costs::cost::Cost,
    error::Result,
    io::{skip_any_whitespace, skip_whitespace},
};

impl<SourceType: PrimInt + Display> CostFunction<SourceType> {
    pub fn write_plain(&self, mut writer: impl Write) -> Result<()> {
        let column_widths: Vec<_> = self
            .function
            .iter()
            .map(|(index, cost)| {
                if index == &SourceType::max_value() {
                    3
                } else if index == &SourceType::min_value() {
                    4
                } else {
                    format!("{index}").len()
                }
                .max(if cost == &Cost::MAX {
                    3
                } else {
                    format!("{cost}").len()
                })
            })
            .collect();

        let mut once = false;
        for (&column_width, (index, _)) in column_widths.iter().zip(self.function.iter()) {
            if once {
                write!(writer, " ")?;
            } else {
                once = true;
            }

            if index == &SourceType::max_value() {
                for _ in 3..column_width {
                    write!(writer, " ")?;
                }
                write!(writer, "inf")?;
            } else if index == &SourceType::min_value() {
                for _ in 4..column_width {
                    write!(writer, " ")?;
                }
                write!(writer, "-inf")?;
            } else {
                write!(writer, "{index: >column_width$}")?;
            }
        }
        writeln!(writer)?;

        let mut once = false;
        for (&column_width, (_, cost)) in column_widths.iter().zip(self.function.iter()) {
            if once {
                write!(writer, " ")?;
            } else {
                once = true;
            }

            if cost == &Cost::MAX {
                for _ in 3..column_width {
                    write!(writer, " ")?;
                }
                write!(writer, "inf")?;
            } else {
                write!(writer, "{cost: >column_width$}")?;
            }
        }

        Ok(())
    }
}

impl<SourceType: PrimInt> CostFunction<SourceType> {
    pub(crate) fn parse_plain(input: &str) -> IResult<&str, Self> {
        let mut input = skip_any_whitespace(input)?;

        let mut indexes = Vec::new();
        while !input.starts_with(['\n', '\r']) {
            let index: SourceType;
            (input, index) = parse_inf_integer(input)?;
            indexes.push(index);
            input = skip_whitespace(input)?;
        }

        input = skip_any_whitespace(input)?;

        let mut costs = Vec::new();
        while !input.starts_with(['\n', '\r']) && !input.is_empty() {
            let cost: u64;
            (input, cost) = parse_inf_integer(input)?;
            costs.push(cost);
            input = skip_whitespace(input)?;
        }

        if indexes.len() != costs.len()
            || indexes[0] != SourceType::min_value()
            || indexes.windows(2).any(|window| window[0] >= window[1])
        {
            return Err(nom::Err::Failure(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Verify,
            }));
        }

        Ok((
            input,
            Self {
                function: indexes
                    .into_iter()
                    .zip(costs.into_iter().map(Into::into))
                    .collect(),
            },
        ))
    }
}

fn parse_inf_integer<Output: PrimInt>(input: &str) -> IResult<&str, Output> {
    let mut length = 0;

    let negative = match input
        .chars()
        .next()
        .ok_or(nom::Err::Failure(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Verify,
        }))? {
        '-' => {
            length += 1;
            true
        }
        '+' => {
            length += 1;
            false
        }
        _ => false,
    };

    if negative && Output::min_value() == Output::zero() {
        return Err(nom::Err::Failure(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Verify,
        }));
    }

    if input[length..].starts_with("inf") {
        length += 3;

        if negative {
            Ok((&input[length..], Output::min_value()))
        } else {
            Ok((&input[length..], Output::max_value()))
        }
    } else {
        let mut result = Output::zero();

        for character in input.chars().skip(length) {
            if character == '_' {
                length += 1;
            } else if character.is_dec_digit() {
                length += 1;

                let character = Output::from(match character {
                    '0' => 0,
                    '1' => 1,
                    '2' => 2,
                    '3' => 3,
                    '4' => 4,
                    '5' => 5,
                    '6' => 6,
                    '7' => 7,
                    '8' => 8,
                    '9' => 9,
                    other => unreachable!("{other}"),
                })
                .unwrap();

                result =
                    result
                        .checked_mul(&Output::from(10).unwrap())
                        .ok_or(nom::Err::Failure(nom::error::Error {
                            input,
                            code: nom::error::ErrorKind::Verify,
                        }))?;
                if negative {
                    result = result.checked_sub(&character).ok_or(nom::Err::Failure(
                        nom::error::Error {
                            input,
                            code: nom::error::ErrorKind::Verify,
                        },
                    ))?;
                } else {
                    result = result.checked_add(&character).ok_or(nom::Err::Failure(
                        nom::error::Error {
                            input,
                            code: nom::error::ErrorKind::Verify,
                        },
                    ))?;
                }
            } else {
                break;
            }
        }

        Ok((&input[length..], result))
    }
}

#[cfg(test)]
mod tests {
    use crate::costs::cost_function::CostFunction;

    #[test]
    fn simple_example() {
        let input = "-inf -12345 -4 -1 0 1 +2 123456 inf\n   1      2  3  4 5 6  7      8   9";
        let expected_output =
            "-inf -12345 -4 -1 0 1 2 123456 inf\n   1      2  3  4 5 6 7      8   9";
        let expected_parsing_result = CostFunction::<isize> {
            function: vec![
                (isize::MIN, 1.into()),
                (-12345, 2.into()),
                (-4, 3.into()),
                (-1, 4.into()),
                (0, 5.into()),
                (1, 6.into()),
                (2, 7.into()),
                (123456, 8.into()),
                (isize::MAX, 9.into()),
            ],
        };

        let (remaining_input, actual_parsing_result) =
            CostFunction::<isize>::parse_plain(input).unwrap();
        assert!(remaining_input.is_empty());

        let mut writer = Vec::new();
        actual_parsing_result.write_plain(&mut writer).unwrap();
        let output = String::from_utf8(writer).unwrap();

        assert_eq!(expected_parsing_result, actual_parsing_result);
        assert_eq!(expected_output, output);
    }
}
