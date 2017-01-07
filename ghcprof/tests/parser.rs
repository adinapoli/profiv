
extern crate ghcprof;
extern crate nom;

use ghcprof::parser::*;
use std::io::prelude::*;
use std::str;
use std::fs::File;
use nom::{Err, IResult};


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
        IResult::Done(leftover, x) => {
            assert!(leftover.is_empty());
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
        IResult::Done(leftover, x) => {
            assert!(leftover.is_empty());
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
        IResult::Done(leftover, x) => {
            assert!(leftover.is_empty());
            assert_eq!(x.module, "Crypto.RNCryptor.V3.Encrypt")
        },
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
        IResult::Done(leftover, Summary(x)) => {
            assert_eq!("", str::from_utf8(leftover).unwrap());
            assert_eq!(x.len(), 9);
            assert_eq!(x[8].alloc_perc, 16.5)
        },
        IResult::Error(Err::Position(_, bytes)) => {
            panic!("error char -> {:?}", str::from_utf8(bytes))
        }
        e => panic!("{:?}", e),
    }
}

#[test]
fn can_parse_extended_summary() {
    match parse_extended_summary("MAIN                                                           MAIN                                     559           0    0.3    0.0   100.0  100.0
 arbitrary                                                     Tests                                   2302           0    0.0    0.0     5.6    3.5
  arbitrary                                                    Data.ByteString.Arbitrary               2304           0    0.0    0.0     5.6    3.5
   fastRandBs                                                  Data.ByteString.Arbitrary               2307           0    0.4    1.7     5.6    3.5
    slowRandBs                                                 Data.ByteString.Arbitrary               2319           0    0.0    0.0     0.0    0.0
    fastRandBs.preChunks                                       Data.ByteString.Arbitrary               2313         100    0.0    0.0     0.0    0.0
    fastRandBs.hashes                                          Data.ByteString.Arbitrary               2312         100    5.3    1.7     5.3    1.7
"
        .as_bytes()) {
        IResult::Done(leftover, ExtendedSummary(trees)) => {
            assert_eq!(trees[0].depth, 0);
            let ref line = trees[0].value;
            assert_eq!(line.cost_centre, "MAIN");
            assert_eq!("", str::from_utf8(leftover).unwrap());
        },
        IResult::Error(Err::Position(_, bytes)) => {
            panic!("error char -> {:?}", str::from_utf8(bytes))
        }
        e => panic!("{:?}", e),
    }
}

#[test]
fn can_parse_summaries_sep() {
    match parse_summaries_sep("
                                                                                                                          individual     inherited
COST CENTRE                                                    MODULE                                   no.     entries  %time %alloc   %time %alloc

"
        .as_bytes()) {
        IResult::Done(_, ()) => {
        },
        IResult::Error(Err::Position(_, bytes)) => {
            panic!("error char -> {:?}", str::from_utf8(bytes))
        }
        e => panic!("{:?}", e),
    }
}

#[test]
fn can_parse_header_and_summary() {
    match parse_header_and_summary("  Thu Dec 29 13:55 2016 Time and Allocation Profiling Report  (Final)

     rncryptor-tests +RTS -p -RTS

  total time  =       53.62 secs   (53615 ticks @ 1000 us, 1 processor)
  total alloc = 60,261,923,248 bytes  (excludes profiling overheads)

COST CENTRE                                       MODULE                      %time %alloc

makeKey                                           Crypto.RNCryptor.Types       61.0   89.5
encryptBlock                                      Crypto.RNCryptor.V3.Encrypt  12.5    0.0
decryptBlock                                      Crypto.RNCryptor.V3.Decrypt  12.5    0.0
fastRandBs.hashes                                 Data.ByteString.Arbitrary     5.3    1.7
encryptBytes                                      Crypto.RNCryptor.V3.Encrypt   5.2    1.7
decryptBytes                                      Crypto.RNCryptor.V3.Decrypt   1.5    1.7
streamingRoundTrip                                Tests                         0.6    1.7
encryptStreamWithContext.finaliseEncryption.(...) Crypto.RNCryptor.V3.Encrypt   0.6    1.7
fastRandBs                                        Data.ByteString.Arbitrary     0.4    1.7

"
        .as_bytes()) {
        IResult::Done(_, (_,_)) => {
        },
        IResult::Error(Err::Position(_, bytes)) => {
            panic!("error char -> {:?}", str::from_utf8(bytes))
        }
        e => panic!("{:?}", e),
    }
}

#[test]
fn can_parse_ghc_profile() {
    let mut prof_file = File::open("../example_format/rncryptor-tests.prof").unwrap();
    let mut profile   = String::new();
    prof_file.read_to_string(&mut profile).unwrap();
    match parse_prof(profile.as_bytes()) {
        IResult::Done(_, prof) => {
            assert_eq!(prof.header.program, "rncryptor-tests +RTS -p -RTS");
        },
        IResult::Error(Err::Position(_, bytes)) => {
            panic!("error char -> {:?}", str::from_utf8(bytes))
        }
        e => panic!("{:?}", e),
    }
}
