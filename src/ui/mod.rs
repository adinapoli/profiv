extern crate rustbox;
extern crate ghcprof;

use self::rustbox::{RustBox, Style, Color, Key};
use std::cmp::{max};
use self::types::*;
use self::style::*;
use ghcprof::parser::{Header, Summary, ExtendedSummary, ExtendedSummaryLine, RoseTree, SummaryLine, GHCProf};

pub mod types;
pub mod style;

pub struct UI {
    ui: RustBox,
}

impl UI {
    pub fn new() -> Result<UI, UIError> {
        match RustBox::init(Default::default()) {
            Result::Ok(v) => Ok(UI { ui: v }),
            Result::Err(_) => Err(UIError::UiInitialisationFailed),
        }
    }

    pub fn render_loop<'a>(&self, prof: GHCProf<'a>) {
        let ref rustbox = self.ui;
        let ref mut user_cursor = UserCursor::new(0,1);
        let ref mut draw_cursor = Cursor::new(1,1);

        let ctx = TuiContext {
            ui: rustbox,
            user_cursor: user_cursor,
            draw_cursor: draw_cursor,
        };

        loop {
            render(&ctx, &prof);

            let status_bar_position = ctx.ui.height() - 1;
            let viewport = format!("({}, {})", ctx.ui.width(), ctx.ui.height());

            // Render the status-bar and the viewport
            ctx.ui.print(0, status_bar_position, Style::empty(), Color::Black, Color::Green, viewport.as_str());
            for i in viewport.len() .. ctx.ui.width() {
                ctx.ui.print(i, status_bar_position, Style::empty(), Color::Black, Color::Green, " ");
            }

            // Render the current line number.
            ctx.ui.print(ctx.ui.width() - 5
                         , status_bar_position
                         , Style::empty()
                         , Color::Black
                         , Color::Green
                         , format!("{}", ctx.user_cursor.y).as_str()
            );

            rustbox.present();
            match rustbox.poll_event(false) {
                Ok(rustbox::Event::KeyEvent(key)) => {
                    match key {
                        Key::Ctrl('d') => {
                            println!("scroll down");
                        }
                        Key::Ctrl('u') => {
                            println!("scroll up");
                        }
                        Key::Char('q') => {
                            break;
                        }
                        Key::Down => {
                            ctx.user_cursor.y += 1;
                        }
                        Key::Up => {
                            ctx.user_cursor.y = max(1, ctx.user_cursor.y - 1);
                        }
                        _ => {}
                    }
                }
                Err(e) => panic!("{}", e),
                _ => {}
            }
        }
    }
}

fn render<'a>(ctx: &TuiContext<RustBox>, prof: &GHCProf<'a>) {
        render_header(&ctx, &prof.header);
        let cursor = render_summary(&ctx, &prof.summary);
        render_extended_summary(&ctx, cursor, &prof.extended_summary);
}

fn render_header<'a>(ctx: &TuiContext<RustBox>, header: &Header<'a>) {
    normal_line(ctx, 1, 1, header.title);
    normal_line(ctx, header.title.len() / 4, 3, header.program);
    let ref tt = header.total_time;
    let ref ta = header.total_alloc;
    let total_time = format!("total time  =       {} secs   ({} ticks @ {} us, {} processor)",
                             tt.time,
                             tt.ticks,
                             tt.freq,
                             tt.procs);
    let total_alloc = format!("total alloc = {} bytes  (excludes profiling overheads)",
                              ta.bytes);
    normal_line(ctx, 1, 5, &total_time);
    normal_line(ctx, 1, 6, &total_alloc)
}

fn render_summary<'a>(ctx: &TuiContext<RustBox>, &Summary(ref lines): &Summary<'a>) -> usize {
    normal_line(ctx, 1, 8, "COST CENTRE");

    // Computes all the slacks to render the summary in a tabulated style.
    let mut lines_mut = lines.clone();
    lines_mut.sort_by(|a, b| b.cost_centre.len().cmp(&a.cost_centre.len()));
    let longest_cc = lines_mut.get(0).map_or(1, |v| v.cost_centre.len());

    lines_mut.sort_by(|a, b| b.module.len().cmp(&a.module.len()));
    let longest_mo = lines_mut.get(0).map_or(1, |v| v.module.len());

    lines_mut.sort_by(|a, b| (format!("{}", b.time_perc).len())
            .cmp(&format!("{}", a.time_perc).len()));
    let longest_tm = lines_mut.get(0).map_or(1, |v| (format!("{}", v.time_perc).len()));

    // Render the rest of the summary header
    normal_line(ctx, longest_cc + 2, 8, "MODULE");
    normal_line(ctx, longest_cc + longest_mo + 4, 8, "%time");
    normal_line(ctx,
                longest_cc + longest_mo + longest_tm + 6,
                8,
                "%alloc");

    let mut idx = 10;

    for line in lines {
        let &SummaryLine { time_perc: time, alloc_perc: memory, .. } = line;
        let tm_str = format!("{}", line.time_perc);
        let cc_len = line.cost_centre.len();
        let cc_slack = longest_cc - cc_len;
        let mo_len = line.module.len();
        let mo_slack = longest_mo - mo_len;
        let tm_len = tm_str.len();
        let tm_slack = longest_tm - tm_len;

        let time_temp = Temperature::from(time);
        let memory_temp = Temperature::from(memory);
        let combined_temp = Temperature::append(&time_temp, &memory_temp);

        styled_line(ctx, 1, idx, &combined_temp, line.cost_centre);
        styled_line(ctx,
                    cc_len + cc_slack + 2,
                    idx,
                    &combined_temp,
                    line.module);
        heat_line(ctx,
                  cc_len + cc_slack + mo_slack + mo_len + 4,
                  idx,
                  &time_temp,
                  &tm_str);
        heat_line(ctx,
                  cc_len + cc_slack + mo_slack + mo_len + tm_slack + tm_len + 6,
                  idx,
                  &memory_temp,
                  &format!("{}", line.alloc_perc));
        idx += 1
    }

    idx
}

fn render_extended_summary<'a>(ctx: &TuiContext<RustBox>, idx: usize, &ExtendedSummary(ref tree): &ExtendedSummary<'a>) {
    // TODO: This needs to scale according to the size of the longest cost centre. Ditto for module etc.
    normal_line(ctx, 1, idx + 2, "                                                                                                                          individual     inherited");
    normal_line(ctx, 1, idx + 3, "COST CENTRE                                                    MODULE                                   no.     entries  %time %alloc   %time %alloc
");
    let cursor = idx + 4;
    render_rose_tree(ctx, &mut Cursor::new(1, cursor), &tree)
}


fn render_rose_tree<'a>(ctx: &TuiContext<RustBox>, cursor: &mut Cursor, tree: &RoseTree<ExtendedSummaryLine<'a>>) {
    cursor.y += 1;
    cursor.x = tree.depth + 1;
    render_extended_summary_line(ctx, cursor, &tree.value);
    for t in &tree.sub_forest {
        render_rose_tree(ctx, cursor, &t)
  }
}

fn render_extended_summary_line<'a>(ctx: &TuiContext<RustBox>, cursor: &Cursor, line: &ExtendedSummaryLine<'a>) {
    let cost_centre_len = line.cost_centre.len();
    let module_len      = line.module.len();
    let no              = format!("{}", line.no);
    let entries         = format!("{}", line.entries);
    let ind_time        = format!("{}", line.individual_time_perc);
    let ind_alloc       = format!("{}", line.individual_alloc_perc);
    let inh_time        = format!("{}", line.inherited_time_perc);
    let inh_alloc       = format!("{}", line.inherited_alloc_perc);
    normal_line(ctx, cursor.x, cursor.y, line.cost_centre);
    // TODO: All hardcoded for now
    normal_line(ctx, 64, cursor.y, line.module);
    normal_line(ctx, 105, cursor.y, &no);
    normal_line(ctx, 113, cursor.y, &entries);
    heat_line(ctx, 122, cursor.y, &Temperature::from(line.individual_time_perc), &ind_time);
    heat_line(ctx, 128, cursor.y, &Temperature::from(line.individual_alloc_perc), &ind_alloc);
    heat_line(ctx, 137, cursor.y, &Temperature::from(line.inherited_time_perc), &inh_time);
    heat_line(ctx, 143, cursor.y, &Temperature::from(line.inherited_alloc_perc), &inh_alloc);
}
