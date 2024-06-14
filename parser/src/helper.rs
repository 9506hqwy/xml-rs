use nom::error::{ErrorKind, ParseError};
use nom::{Compare, CompareResult, Err, FindSubstring, IResult, InputLength, Parser, Slice};
use std::ops::{RangeFrom, RangeTo};

// -----------------------------------------------------------------------------------------------

pub fn take_except<F, T, Input, Error: ParseError<Input>>(
    mut parser: F,
    except: T,
) -> impl FnMut(Input) -> IResult<Input, Input, Error>
where
    F: Parser<Input, Input, Error>,
    Input: Clone,
    T: Clone + Compare<Input>,
{
    move |input: Input| {
        let i = input.clone();
        let e = except.clone();
        match parser.parse(i) {
            Ok((rest, value)) => match e.compare_no_case(value.clone()) {
                CompareResult::Ok => Err(Err::Error(Error::from_error_kind(
                    input,
                    ErrorKind::TakeUntil,
                ))),
                _ => Ok((rest, value)),
            },
            Err(e) => Err(e),
        }
    }
}

pub fn take_until<F, T, Input, Error: ParseError<Input>>(
    mut parser: F,
    except: T,
) -> impl FnMut(Input) -> IResult<Input, Input, Error>
where
    F: Parser<Input, Input, Error>,
    Input: Clone + FindSubstring<T> + InputLength + Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
    T: Clone,
{
    move |input: Input| {
        let i = input.clone();
        let e = except.clone();
        match parser.parse(i) {
            Ok((rest, value)) => match value.find_substring(e) {
                Some(index) => {
                    if index == 0 {
                        Err(Err::Error(Error::from_error_kind(
                            input,
                            ErrorKind::TakeUntil,
                        )))
                    } else {
                        Ok((input.slice(index..), input.slice(..index)))
                    }
                }
                None => Ok((rest, value)),
            },
            Err(e) => Err(e),
        }
    }
}

// -----------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nom::character::complete::alpha1;

    #[test]
    fn test_take_except() {
        let (rest, ret) = take_except::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("aaa").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaa", ret);

        let (rest, ret) = take_except::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("abc").unwrap();
        assert_eq!("", rest);
        assert_eq!("abc", ret);

        let (rest, ret) = take_except::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("bca").unwrap();
        assert_eq!("", rest);
        assert_eq!("bca", ret);

        let err = take_except::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("1")
            .err()
            .unwrap();
        assert_eq!(Err::Error(("1", ErrorKind::Alpha)), err);

        let err = take_except::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("bc")
            .err()
            .unwrap();
        assert_eq!(Err::Error(("bc", ErrorKind::TakeUntil)), err);
    }

    #[test]
    fn test_take_until() {
        let (rest, ret) = take_until::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("aaa").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaa", ret);

        let (rest, ret) = take_until::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("aabc").unwrap();
        assert_eq!("bc", rest);
        assert_eq!("aa", ret);

        let (rest, ret) = take_until::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("aaba").unwrap();
        assert_eq!("", rest);
        assert_eq!("aaba", ret);

        let (rest, ret) = take_until::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("aa1").unwrap();
        assert_eq!("1", rest);
        assert_eq!("aa", ret);

        let err = take_until::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("1")
            .err()
            .unwrap();
        assert_eq!(Err::Error(("1", ErrorKind::Alpha)), err);

        let err = take_until::<_, _, _, (_, ErrorKind)>(alpha1, "bc")("bc")
            .err()
            .unwrap();
        assert_eq!(Err::Error(("bc", ErrorKind::TakeUntil)), err);
    }
}

// -----------------------------------------------------------------------------------------------
