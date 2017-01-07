
extern crate nom;

use std::str;
use nom::{IResult, Needed, is_space, space, is_digit, line_ending, not_line_ending};

// Rose Tree

#[derive(Debug, PartialEq)]
pub struct RoseTree<T> {
    pub depth: usize,
    pub value: T,
    pub sub_forest: Vec<RoseTree<T>>,
}

#[derive(Debug, PartialEq)]
pub struct TotalTime {
    pub time: f32,
    pub ticks: u32,
    pub freq: u16,
    pub procs: u8,
}

#[derive(Debug, PartialEq)]
pub struct TotalAlloc {
    pub bytes: u64,
}

#[derive(Debug, PartialEq)]
pub struct Header<'a> {
    pub title: &'a str,
    pub program: &'a str,
    pub total_time: TotalTime,
    pub total_alloc: TotalAlloc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Summary<'a>(pub Vec<SummaryLine<'a>>);

#[derive(Debug, Clone, PartialEq)]
pub struct SummaryLine<'a> {
    pub cost_centre: &'a str,
    pub module: &'a str,
    pub time_perc: f32,
    pub alloc_perc: f32,
}

#[derive(Debug, PartialEq)]
pub struct ExtendedSummary<'a>(pub Vec<RoseTree<ExtendedSummaryLine<'a>>>);

#[derive(Debug, PartialEq)]
pub struct ExtendedSummaryLine<'a> {
    pub cost_centre: &'a str,
    module: &'a str,
    no: u32,
    entries: u32,
    individual_time_perc: f32,
    individual_alloc_perc: f32,
    inherited_time_perc: f32,
    inherited_alloc_perc: f32,
}

#[derive(Debug, PartialEq)]
pub struct GHCProf<'a> {
    pub header: Header<'a>,
    pub summary: Summary<'a>,
    extended_summary: ExtendedSummary<'a>,
}

fn is_numlike(chr: u8) -> bool {
    is_digit(chr) || chr == '.' as u8 || chr == ',' as u8
}

named!(numlike<&[u8], &str>, map_res!(take_while!(is_numlike) , str::from_utf8));

// TODO: Improve error handling.
pub fn parse_num<T>(input: &[u8]) -> IResult<&[u8], T>
    where T: str::FromStr
{
    match numlike(input) {
        IResult::Done(leftover, n) => {
            match n.replace(",", "").to_string().parse::<T>() {
                Ok(v) => IResult::Done(leftover, v),
                Err(_) => {
                    println!("inside: {:?}", n);
                    println!("original input: {:?}", str::from_utf8(&input[0..50]));
                    IResult::Incomplete(Needed::Size(2))
                }
            }
        }
        e => {
            println!("{:?}", str::from_utf8(input));
            println!("{:?}", e);
            IResult::Incomplete(Needed::Size(1))
        }
    }
}

named!(pub text_line<&[u8],&str>, do_parse!(
    opt!(space) >>
    res: map_res!(not_line_ending, str::from_utf8) >>
    line_ending >>
    (res)
));

named!(pub total_time<&[u8],TotalTime>, do_parse!(
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

named!(pub total_alloc<&[u8],TotalAlloc>, do_parse!(
    space >> take_till!(is_digit) >>
        bytes: parse_num >>
        not_line_ending >> line_ending >>
        (TotalAlloc { bytes: bytes})
));

named!(pub parse_header<&[u8], Header>, do_parse!(
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

named!(pub parse_summary_line<&[u8], SummaryLine>, do_parse!(
    cost_centre: map_res!(take_till!(is_space), str::from_utf8) >>
        space >>
        module: map_res!(take_till!(is_space), str::from_utf8) >>
        take_while!(is_space) >>
        time_perc: parse_num >>
        take_while!(is_space) >>
        alloc_perc: parse_num >>
        line_ending >>
        (SummaryLine{
            cost_centre: cost_centre,
            module: module,
            time_perc: time_perc,
            alloc_perc: alloc_perc,
        })
));

named!(pub parse_summary<&[u8], Summary>, do_parse!(
    lines: many_till!(parse_summary_line, line_ending) >>
    (Summary(lines.0))
));

named!(pub parse_extended_summary_line<&[u8], ExtendedSummaryLine>, do_parse!(
    cost_centre: map_res!(take_till!(is_space), str::from_utf8) >>
    space >>
    module: map_res!(take_till!(is_space), str::from_utf8) >>
    take_while!(is_space) >>
    no: parse_num >>
    take_while!(is_space) >>
    entries: parse_num >>
    take_while!(is_space) >>
    individual_time_perc: parse_num >>
    take_while!(is_space) >>
    individual_alloc_perc: parse_num >>
    take_while!(is_space) >>
    inherited_time_perc: parse_num >>
    take_while!(is_space) >>
    inherited_alloc_perc: parse_num >>
    line_ending >>
    (ExtendedSummaryLine{
        cost_centre: cost_centre,
        module: module,
        no: no,
        entries: entries,
        individual_time_perc:  individual_time_perc,
        individual_alloc_perc: individual_alloc_perc,
        inherited_time_perc:  inherited_time_perc,
        inherited_alloc_perc: inherited_alloc_perc,
    })
));

enum Descendant {
    Parent,
    Sibling,
    Child,
}

// fn peek_next_descendant(input: &[u8], current_depth: usize) -> IResult<&[u8], Descendant> {
//    let (_, next_depth) = try_parse!(input, peek!(node_depth));
//    IResult::Done(input,
//                  match next_depth {
//                      _ if next_depth == current_depth => Descendant::Sibling,
//                      _ if next_depth < current_depth => Descendant::Parent,
//                      _ => Descendant::Child,
//                  })
//

/// Parses a top level node of the RoseTree
pub fn parse_node(input: &[u8],
                  current_depth: usize)
                  -> IResult<&[u8], RoseTree<ExtendedSummaryLine>> {
    let (i1, value) = try_parse!(input, parse_extended_summary_line);
    println!("--------> {:?}", value);
    let (i2, next_depth) = try_parse!(i1, node_depth);
    println!("--next depth-----> {:?}", next_depth);
    match next_depth {
        None => {
            IResult::Done(i2,
                          RoseTree {
                              depth: current_depth,
                              value: value,
                              sub_forest: Vec::new(),
                          })
        }
        Some(depth) => {
            if depth < current_depth {
                return IResult::Done(i2,
                                     RoseTree {
                                         depth: current_depth,
                                         value: value,
                                         sub_forest: Vec::new(),
                                     });
            }

            let (i3, siblings) = try_parse!(i2, many0!(apply!(parse_node, depth)));
            IResult::Done(i3,
                          RoseTree {
                              depth: current_depth,
                              value: value,
                              sub_forest: siblings,
                          })
        }
    }
}

fn node_depth(input: &[u8]) -> IResult<&[u8], Option<usize>> {
    if input.is_empty() {
        return IResult::Done(input, None);
    } else {
        let (l, spc) = try_parse!(input, opt!(space));
        match spc {
            None => IResult::Done(l, Some(0)),
            Some(s) => IResult::Done(l, Some(s.len())),
        }
    }
}

named!(pub parse_extended_summary<&[u8], ExtendedSummary>, do_parse!(
    trees: many1!(apply!(parse_node, 0)) >>
    (ExtendedSummary(trees))
));

named!(pub parse_summaries_sep<&[u8], ()>, do_parse!(
    line_ending >>
    count!(text_line, 2) >>
    line_ending >>
    (())
));

named!(pub parse_header_and_summary<&[u8], (Header,Summary)>, do_parse!(
    h: parse_header >>
        line_ending >>
        text_line >>
        line_ending >>
        summary: parse_summary >>
    ((h, summary))
));

named!(pub parse_prof<&[u8], GHCProf>, do_parse!(
    header_and_summary: parse_header_and_summary >>
    parse_summaries_sep >>
    extended_summary: parse_extended_summary >>
    (GHCProf{
        header: header_and_summary.0,
        summary: header_and_summary.1,
        extended_summary: extended_summary
    })
));

// TODO: Return a Result.
pub fn parse_prof_file<'a>(content: &'a [u8]) -> Option<GHCProf<'a>> {
    match parse_prof(content) {
        IResult::Done(_, prof) => Some(prof),
        _ => None,
    }
}
