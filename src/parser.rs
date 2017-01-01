
extern crate nom;

use std::str;
use nom::{Err, IResult, Needed, newline, is_space, digit, space, multispace, is_digit, line_ending, not_line_ending};

#[derive(Debug, PartialEq)]
struct TotalTime {
    time: f32,
    ticks: u32,
    freq: u16,
    procs: u8,
}

#[derive(Debug, PartialEq)]
struct TotalAlloc {
    bytes: u64,
}

#[derive(Debug, PartialEq)]
struct Header<'a> {
    title: &'a str,
    program: &'a str,
    total_time: TotalTime,
    total_alloc: TotalAlloc,
}

#[derive(Debug, PartialEq)]
struct Summary<'a>(Vec<SummaryLine<'a>>);

#[derive(Debug, PartialEq)]
struct SummaryLine<'a> {
    cost_centre: &'a str,
    module: &'a str,
    time_perc: f32,
    alloc_perc: f32,
}

fn is_numlike(chr: u8) -> bool {
    is_digit(chr) || chr == '.' as u8 || chr == ',' as u8
}

named!(numlike<&[u8], &str>, map_res!(take_while!(is_numlike) , str::from_utf8));

// TODO: Improve error handling.
fn parse_num<T>(input: &[u8]) -> IResult<&[u8], T>
    where T: str::FromStr
{
    match numlike(input) {
        IResult::Done(leftover, n) => {
            match n.replace(",", "").to_string().parse::<T>() {
                Ok(v) => IResult::Done(leftover, v),
                Err(_) => {
                    println!("inside: {:?}", n);
                    IResult::Incomplete(Needed::Size(2))
                },
            }
        }
        e => {
            println!("{:?}", str::from_utf8(input));
            println!("{:?}", e);
            IResult::Incomplete(Needed::Size(1))
        },
    }
}

named!(text_line<&[u8],&str>, do_parse!(
    opt!(space) >>
    res: map_res!(not_line_ending, str::from_utf8) >>
    line_ending >>
    (res)
));

named!(total_time<&[u8],TotalTime>, do_parse!(
    space >> take_till!(is_digit) >>
        time: parse_num >>
        take_until_and_consume!("(") >>
        ticks: parse_num >>
        take_till!(is_digit) >>
        freq: parse_num >>
        take_till!(is_digit) >>
        procs: parse_num >>
        not_line_ending >> line_ending >>
        (TotalTime{
            time: time,
            ticks: ticks,
            freq: freq,
            procs: procs
        })
));

named!(total_alloc<&[u8],TotalAlloc>, do_parse!(
    space >> take_till!(is_digit) >>
        bytes: parse_num >>
        not_line_ending >> line_ending >>
        (TotalAlloc { bytes: bytes})
));

named!(parse_header<&[u8], Header>, do_parse!(
    title: text_line >>
    line_ending >>
    program: text_line >>
    line_ending >>
    total_time:  total_time  >>
    total_alloc: total_alloc >>
    (Header {
        title: title,
        program: program,
        total_time: total_time,
        total_alloc: total_alloc,
    })
));

named!(parse_summary_line<&[u8], SummaryLine>, do_parse!(
    cost_centre: map_res!(take_till!(is_space), str::from_utf8) >>
        space >>
        module: map_res!(take_till!(is_space), str::from_utf8) >>
        take_while!(is_space) >>
        time_perc: parse_num >>
        take_while!(is_space) >>
        alloc_perc: parse_num >>
        multispace >>
        (SummaryLine{
            cost_centre: cost_centre,
            module: module,
            time_perc: time_perc,
            alloc_perc: alloc_perc,
        })
));

named!(parse_summary<&[u8], Summary>, do_parse!(
    lines: many1!(parse_summary_line) >>
    (Summary(lines))
));

#[test]
fn can_parse_report_title() {
    match text_line("  Thu Dec 29 13:55 2016 Time and Allocation Profiling Report  (Final)
rest"
        .as_bytes()) {
        IResult::Done(leftover, x) => {
            assert_eq!(x, "Thu Dec 29 13:55 2016 Time and Allocation Profiling Report  (Final)");
            assert_eq!(leftover, "rest".as_bytes())
        }
        e => panic!("{:?}", e),
    }
}

#[test]
fn can_parse_prog_name() {
    match text_line("     rncryptor-tests +RTS -p -RTS

"
        .as_bytes()) {
        IResult::Done(_, x) => assert_eq!(x, "rncryptor-tests +RTS -p -RTS"),
        e => panic!("{:?}", e),
    }
}

#[test]
fn can_parse_total_time() {
    match total_time("  total time  =       53.62 secs   (53615 ticks @ 1000 us, 1 processor)
"
        .as_bytes()) {
        IResult::Done(_, x) => {
            assert_eq!(x, TotalTime{
            time: 53.62,
            ticks: 53615,
            freq: 1000,
            procs: 1
        })
        }
        IResult::Error(Err::Position(_, bytes)) => {
            panic!("error char -> {:?}", str::from_utf8(bytes))
        }
        e => panic!("{:?}", e),
    }
}

#[test]
fn can_parse_total_alloc() {
    match total_alloc("  total alloc = 60,261,923,248 bytes  (excludes profiling overheads)
"
        .as_bytes()) {
        IResult::Done(_, x) => {
            assert_eq!(x, TotalAlloc{
            bytes: 60_261_923_248,
        })
        }
        IResult::Error(Err::Position(_, bytes)) => {
            panic!("error char -> {:?}", str::from_utf8(bytes))
        }
        e => panic!("{:?}", e),
    }
}

#[test]
fn can_parse_header() {
    match parse_header("  Thu Dec 29 13:55 2016 Time and Allocation Profiling Report  (Final)

     rncryptor-tests +RTS -p -RTS

  total time  =       53.62 secs   (53615 ticks @ 1000 us, 1 processor)
  total alloc = 60,261,923,248 bytes  (excludes profiling overheads)
"
        .as_bytes()) {
        IResult::Done(_, x) => {
            assert_eq!(x.program, "rncryptor-tests +RTS -p -RTS");
            assert_eq!(x.total_alloc, TotalAlloc{bytes: 60_261_923_248,})
        }
        IResult::Error(Err::Position(_, bytes)) => {
            panic!("error char -> {:?}", str::from_utf8(bytes))
        }
        e => panic!("{:?}", e),
    }
}

#[test]
fn can_parse_summary_line() {
    match parse_summary_line("encryptBlock                                      Crypto.RNCryptor.V3.Encrypt  25.4    0.0
"
        .as_bytes()) {
        IResult::Done(_, x) => assert_eq!(x.module, "Crypto.RNCryptor.V3.Encrypt"),
        IResult::Error(Err::Position(_, bytes)) => {
            panic!("error char -> {:?}", str::from_utf8(bytes))
        }
        e => panic!("{:?}", e),
    }
}

#[test]
fn can_parse_summary() {
    match parse_summary(r###"encryptBlock                                      Crypto.RNCryptor.V3.Encrypt  25.4    0.0
decryptBlock                                      Crypto.RNCryptor.V3.Decrypt  25.1    0.0
fastpbkdf2_fn.\.\.\                               Crypto.KDF.PBKDF2            15.2    0.0
encryptBytes                                      Crypto.RNCryptor.V3.Encrypt  12.3   16.5
fastRandBs.hashes                                 Data.ByteString.Arbitrary    10.9   16.6
encryptStreamWithContext.finaliseEncryption.(...) Crypto.RNCryptor.V3.Encrypt   2.7   16.5
streamingRoundTrip                                Tests                         2.7   16.5
fastRandBs                                        Data.ByteString.Arbitrary     2.7   16.5
decryptBytes                                      Crypto.RNCryptor.V3.Decrypt   2.2   16.5
"###
        .as_bytes()) {
        IResult::Done(_, Summary(x)) => assert_eq!(x.len(), 9),
        IResult::Error(Err::Position(_, bytes)) => {
            panic!("error char -> {:?}", str::from_utf8(bytes))
        }
        e => panic!("{:?}", e),
    }
}
