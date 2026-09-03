use catsquad_log::prelude::*;

pub type Uuid = [u8; 16];

pub const RADIX128: u128 = 62;

pub fn uuid_to_str(uuid: Uuid) -> String {
    u128_to_str(u128::from_be_bytes(uuid))
}

pub fn str_to_uuid(uuid: impl AsRef<str>) -> Uuid {
    str_to_u128(uuid.as_ref()).to_be_bytes()
}

pub fn u128_to_str(uuid: u128) -> String {
    let mut num = uuid;
    let mut buffer = [0; 255];
    let mut index = buffer.len();

    loop {
        if num == 0 {
            break;
        }
        index -= 1;

        let n = num % RADIX128;
        let c = num_to_char(n as u8);
        if c == 0 {
            return String::new();
        }
        num = num / RADIX128;
        buffer[index] = c;
    }
    let slice = &buffer[index..];
    let output = str::from_utf8(slice).unwrap().to_string();
    trace!("uuid_to_str {output}");
    output
}

pub fn str_to_u128(uuid: &str) -> u128 {
    let chars = uuid.chars();
    // chars.count()
    if uuid.len() == 0 {
        return 0;
    }
    if uuid.len() == 1 {
        return uuid
            .chars()
            .next()
            .map(|v| char_to_num(v as u8))
            .unwrap_or_default() as u128;
    }

    let mut buffer = 0_u128;
    for (i, c) in chars.rev().enumerate().map(|(i, c)| (i as u32, c as u8)) {
        let n = char_to_num(c);
        if n == 255 {
            return 0;
        }

        trace!("str_to_uuid {buffer} += {RADIX128} ** {i} * {n}");
        buffer += RADIX128.pow(i) * (n as u128);
    }

    trace!("str_to_uuid {buffer}");

    buffer
    // const RADIX128: u128 = 62;
    // let mut num = u128::from_be_bytes(uuid);
    // let mut buffer = [0; 255];
    // let mut index = buffer.len() - 1;

    // loop {
    //     if num == 0 {
    //         break;
    //     }

    //     let n = num % RADIX128;
    //     let c = num_to_char(n as u8);
    //     num = num / RADIX128;
    //     buffer[index] = c;

    //     index -= 1;
    // }
    // let slice = &buffer[index..];
    // str::from_utf8(slice).unwrap().to_string()
}

pub fn num_to_char(n: u8) -> u8 {
    const RANGE1: u8 = b'9' - b'0';
    const RANGE2: u8 = b'Z' - b'A';
    const RANGE3: u8 = b'z' - b'a';

    let c = match n {
        n if n <= RANGE1 => n + b'0',
        n if n <= RANGE1 + RANGE2 + 1 => n + b'A' - RANGE1 - 1,
        n if n <= RANGE1 + RANGE2 + RANGE3 + 2 => n + b'a' - RANGE1 - RANGE2 - 2,
        _ => 0,
    };
    trace!("num_to_char {c}");
    c
}

pub fn char_to_num(c: u8) -> u8 {
    const RANGE1: u8 = b'9' - b'0';
    const RANGE2: u8 = b'Z' - b'A';

    let n = match c {
        c if c <= b'9' => c - b'0',
        c if c >= b'A' && c <= b'Z' => c - b'A' + RANGE1 + 1,
        c if c >= b'a' && c <= b'z' => c - b'a' + RANGE1 + RANGE2 + 2,
        _ => 255,
    };
    trace!("char_to_num {c}");
    n
}

// pub fn num_to_char()

#[test]
fn test_uuid_to_str() {
    init_log();

    assert_eq!(u128_to_str(10), "A");
    assert_eq!(u128_to_str(61), "z");
    assert_eq!(u128_to_str(62), "10");
    assert_eq!(u128_to_str(63), "11");
    assert_eq!(u128_to_str(u128::MAX), "7n42DGM5Tflk9n8mt7Fhc7");
    // let result = ;
    // trace!("{result}");
}

#[test]
fn test_str_to_uuid() {
    init_log();

    // assert_eq!(str_to_uuid("9"), 9);
    assert_eq!(str_to_u128("10"), 62);
    assert_eq!(str_to_u128("11"), 63);
    assert_eq!(str_to_u128("7n42DGM5Tflk9n8mt7Fhc7"), u128::MAX);

    // trace!("{result}");
}

#[test]
fn test_num_to_char() {
    init_log();
    let chars = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E',
        b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
        b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i',
        b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x',
        b'y', b'z',
    ];
    for (i, c) in chars.into_iter().enumerate() {
        assert_eq!(num_to_char(i as u8), c);
    }
    assert_eq!(num_to_char(61), b'z');
    assert_eq!(num_to_char(62), 0);
}

#[test]
fn test_char_to_num() {
    init_log();
    let chars = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E',
        b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
        b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i',
        b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x',
        b'y', b'z',
    ];
    for (i, c) in chars.into_iter().enumerate() {
        assert_eq!(char_to_num(c), i as u8);
    }
    assert_eq!(char_to_num(b'z'), 61);
    assert_eq!(char_to_num(62), 255);
}
