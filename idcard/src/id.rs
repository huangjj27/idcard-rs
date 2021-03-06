//! 第二代中华人民共和国身份证公民身份号码

use gb2260::Division;
use std::str::FromStr;

use crate::utils::{Date, Seq};

const IDNUMBER_LENGTH: usize = 18;
const WEIGHTS: [u8; 17] = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
const CHECK_CODE: [char; 11] = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];
const DIV_CODE_LENGTH: usize = 6;
const BIRTHDAY_LENGTH: usize = 8;
const SEQ_LENGTH: usize = 3;
const ID_MODULE: u8 = 11;

/// 第二代中华人民共和国身份证公民身份号码。包括身份证持有人出生时行政区划分代码（6位数字）、
/// 出生日期（8位数字）、 当日出生顺序号（3位数字），以及一位的校验码。
///
/// 结构中不需要存校验码，只有合法的身份号码才能被转换成该结构体。
#[derive(Debug, PartialEq)]
pub struct IdentityNumber {
    // 中华人民共和国国家标准 GB/T 2260 行政区划代码
    div: Division,

    // 出生日期
    birth: Date,

    // 出生顺序号
    seq: Seq,
}

/// 通常违反身份号码校验规则的错误
#[derive(Debug, PartialEq)]
pub enum InvalidId {
    /// 第二代身份号码长度为18位，其他位数的字符串均不可能为身份号码
    LengthNotMatch(usize),

    /// 行政地区代码没有在历史的 GB/T 2260 地区编码中找到
    DivisionNotFound(String),

    /// 正常人类寿命一般不超过 120年，因此不会还有 1900 年之前出生的老者
    /// 另外也不可能超过验证时出生。
    InvalidBirthday(String),

    /// 序列号格式不正确
    InvalidSeq(String),

    /// 校验码为非法字符
    InvalidCheckCode(char),

    /// 字符串格式正确，但是校验码与输入不匹配
    WrongCheckCode(char),
}

impl FromStr for IdentityNumber {
    type Err = InvalidId;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_len = s.chars().count();
        if s_len != IDNUMBER_LENGTH {
            return Err(InvalidId::LengthNotMatch(s_len));
        }

        let (div_code, rest) = s.split_at(DIV_CODE_LENGTH);
        let div = match Division::get(div_code) {
            Some(d) => d,
            None => return Err(InvalidId::DivisionNotFound(div_code.to_owned())),
        };

        let (birthday, rest) = rest.split_at(BIRTHDAY_LENGTH);
        let birth = birthday
            .parse::<Date>()
            .map_err(|_| InvalidId::InvalidBirthday(birthday.to_owned()))?;

        let (seq, chk_code) = rest.split_at(SEQ_LENGTH);
        let seq = seq
            .parse::<Seq>()
            .map_err(|_| InvalidId::InvalidSeq(seq.to_owned()))?;

        let chk_code = match chk_code.chars().next() {
            Some(chr) if CHECK_CODE.contains(&chr) => chr,
            Some(chr) if !CHECK_CODE.contains(&chr) => {
                return Err(InvalidId::InvalidCheckCode(chr))
            }
            Some(_) => unreachable!("chk_code should be always found. This is a bug"),
            None => unreachable!("chk_code should be always found. This is a bug"),
        };

        let chk_idx: usize =
            s.chars()
                .take(IDNUMBER_LENGTH - 1)
                .map(|chr| chr.to_digit(10).unwrap() as u8)
                .zip(WEIGHTS.iter())
                .fold(0u8, |acc, (d, w)| (acc + d * w) % ID_MODULE) as usize;
        if chk_code != CHECK_CODE[chk_idx] {
            return Err(InvalidId::WrongCheckCode(chk_code));
        }

        Ok(IdentityNumber { div, birth, seq })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wrong_length_id() {
        let shorter = "51010819720505213";
        assert_eq!(
            shorter.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::LengthNotMatch(IDNUMBER_LENGTH - 1)
        );

        let longer = "5101081972050521378";
        assert_eq!(
            longer.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::LengthNotMatch(IDNUMBER_LENGTH + 1)
        );
    }

    #[test]
    fn test_invalid_division() {
        let wrong_division = "000000197205052137";
        assert_eq!(
            wrong_division.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::DivisionNotFound("000000".to_string())
        );

        let wrong_format = "51X108197205052137";
        assert_eq!(
            wrong_format.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::DivisionNotFound("51X108".to_string())
        );
    }

    #[test]
    fn test_invalid_birthday() {
        let wrong_format = "5101081972?5052137";
        assert_eq!(
            wrong_format.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::InvalidBirthday("1972?505".to_string())
        );

        let old_date = "510108187205052137";
        assert_eq!(
            old_date.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::InvalidBirthday("18720505".to_string())
        );

        let upcoming = "510108297205052137";
        assert_eq!(
            upcoming.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::InvalidBirthday("29720505".to_string())
        );
    }

    #[test]
    fn test_invalid_seq() {
        let wrong_format = "5101081972050521$7";
        assert_eq!(
            wrong_format.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::InvalidSeq("21$".to_string())
        );
    }

    #[test]
    fn test_invalid_checkcode() {
        let wrong_format = "51010819720505213%";
        assert_eq!(
            wrong_format.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::InvalidCheckCode('%')
        );

        // 小写的 `x` 校验码也被认为是非法校验码
        let wrong_format = "51010819720505213x";
        assert_eq!(
            wrong_format.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::InvalidCheckCode('x')
        );
    }

    #[test]
    fn test_wrong_checkcode() {
        let wrong_chk = "51010819720505213X";
        assert_eq!(
            wrong_chk.parse::<IdentityNumber>().unwrap_err(),
            InvalidId::WrongCheckCode('X')
        );
    }

    #[test]
    fn test_valid_id() {
        let id = IdentityNumber {
            div: Division::get("510108").unwrap(),
            birth: str::parse::<Date>("19720505").unwrap(),
            seq: str::parse::<Seq>("213").unwrap(),
        };

        let valid_str = "510108197205052137";
        assert_eq!(valid_str.parse::<IdentityNumber>().unwrap(), id);
    }
}
