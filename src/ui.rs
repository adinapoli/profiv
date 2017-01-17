extern crate rustbox;
extern crate ghcprof;

use self::rustbox::{Style, RustBox, Color, Key};
use ghcprof::parser::{Header, Summary, ExtendedSummary, ExtendedSummaryLine, RoseTree, SummaryLine, GHCProf};

pub struct UI {
    ui: RustBox,
}

#[derive(Debug)]
pub enum UIError {
    UiInitialisationFailed,
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
        render_header(rustbox, &prof.header);
        let cursor = render_summary(rustbox, &prof.summary);
        render_extended_summary(rustbox, cursor, &prof.extended_summary);
        loop {
            rustbox.present();
            match rustbox.poll_event(false) {
                Ok(rustbox::Event::KeyEvent(key)) => {
                    match key {
                        Key::Char('q') => {
                            break;
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

fn normal_line(rustbox: &RustBox, x: usize, y: usize, str: &str) {
    rustbox.print(x, y, rustbox::RB_NORMAL, Color::White, Color::Default, str);
}

fn render_header<'a>(rustbox: &RustBox, header: &Header<'a>) {
    normal_line(rustbox, 1, 1, header.title);
    normal_line(rustbox, header.title.len() / 4, 3, header.program);
    let ref tt = header.total_time;
    let ref ta = header.total_alloc;
    let total_time = format!("total time  =       {} secs   ({} ticks @ {} us, {} processor)",
                             tt.time,
                             tt.ticks,
                             tt.freq,
                             tt.procs);
    let total_alloc = format!("total alloc = {} bytes  (excludes profiling overheads)",
                              ta.bytes);
    normal_line(rustbox, 1, 5, &total_time);
    normal_line(rustbox, 1, 6, &total_alloc)
}

enum Temperature {
    Hot,
    Warm,
    Cold,
}

impl Temperature {
    fn from(temp_c: f32) -> Temperature {
        match temp_c {
            0.0...10.0 => Temperature::Cold,
            10.1...50.0 => Temperature::Warm,
            _ => Temperature::Hot,
        }
    }

    // Monoidal append.
    fn append(t1: &Temperature, t2: &Temperature) -> Temperature {
        match (t1, t2) {
            (&Temperature::Hot, _) => Temperature::Hot,
            (_, &Temperature::Hot) => Temperature::Hot,
            (&Temperature::Warm, _) => Temperature::Warm,
            (_, &Temperature::Warm) => Temperature::Warm,
            _ => Temperature::Cold,
        }
    }

    fn to_style(&self) -> Style {
        match *self {
            Temperature::Cold => rustbox::RB_NORMAL,
            Temperature::Warm => rustbox::RB_NORMAL,
            Temperature::Hot => rustbox::RB_BOLD,
        }
    }

    fn to_colour(&self) -> Color {
        match *self {
            Temperature::Cold => Color::White,
            Temperature::Warm => Color::Yellow,
            Temperature::Hot => Color::Red,
        }
    }
}

fn heat_line(rustbox: &RustBox, x: usize, y: usize, temp: &Temperature, str: &str) {
    rustbox.print(x, y, temp.to_style(), temp.to_colour(), Color::Default, str);
}

fn styled_line(rustbox: &RustBox, x: usize, y: usize, temp: &Temperature, str: &str) {
    rustbox.print(x, y, temp.to_style(), Color::White, Color::Default, str);
}

fn render_summary<'a>(rustbox: &RustBox, &Summary(ref lines): &Summary<'a>) -> usize {
    normal_line(rustbox, 1, 8, "COST CENTRE");

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
    normal_line(rustbox, longest_cc + 2, 8, "MODULE");
    normal_line(rustbox, longest_cc + longest_mo + 4, 8, "%time");
    normal_line(rustbox,
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

        styled_line(rustbox, 1, idx, &combined_temp, line.cost_centre);
        styled_line(rustbox,
                    cc_len + cc_slack + 2,
                    idx,
                    &combined_temp,
                    line.module);
        heat_line(rustbox,
                  cc_len + cc_slack + mo_slack + mo_len + 4,
                  idx,
                  &time_temp,
                  &tm_str);
        heat_line(rustbox,
                  cc_len + cc_slack + mo_slack + mo_len + tm_slack + tm_len + 6,
                  idx,
                  &memory_temp,
                  &format!("{}", line.alloc_perc));
        idx += 1
    }

    idx
}

fn render_extended_summary<'a>(rustbox: &RustBox, idx: usize, &ExtendedSummary(ref tree): &ExtendedSummary<'a>) {
    // TODO: This needs to scale according to the size of the longest cost centre. Ditto for module etc.
    normal_line(rustbox, 1, idx + 2, "                                                                                                                          individual     inherited");
    normal_line(rustbox, 1, idx + 3, "COST CENTRE                                                    MODULE                                   no.     entries  %time %alloc   %time %alloc
");
    let cursor = idx + 4;
    render_rose_tree(rustbox, &mut (1, cursor), &tree)
}


fn render_rose_tree<'a>(rustbox: &RustBox, cursor: &mut (usize, usize), tree: &RoseTree<ExtendedSummaryLine<'a>>) {
    cursor.1 += 1;
    cursor.0 = tree.depth + 1;
    render_extended_summary_line(rustbox, cursor, &tree.value);
    for t in &tree.sub_forest {
        render_rose_tree(rustbox, cursor, &t)
  }
}

fn render_extended_summary_line<'a>(rustbox: &RustBox, cursor: &mut (usize, usize), line: &ExtendedSummaryLine<'a>) {
    let cost_centre_len = line.cost_centre.len();
    let module_len      = line.module.len();
    let no              = format!("{}", line.no);
    let entries         = format!("{}", line.entries);
    let ind_time        = format!("{}", line.individual_time_perc);
    let ind_alloc       = format!("{}", line.individual_alloc_perc);
    let inh_time        = format!("{}", line.inherited_time_perc);
    let inh_alloc       = format!("{}", line.inherited_alloc_perc);
    normal_line(rustbox, cursor.0, cursor.1, line.cost_centre);
    // TODO: All hardcoded for now
    normal_line(rustbox, 64, cursor.1, line.module);
    normal_line(rustbox, 105, cursor.1, &no);
    normal_line(rustbox, 113, cursor.1, &entries);
    heat_line(rustbox, 122, cursor.1, &Temperature::from(line.individual_time_perc), &ind_time);
    heat_line(rustbox, 128, cursor.1, &Temperature::from(line.individual_alloc_perc), &ind_alloc);
    heat_line(rustbox, 137, cursor.1, &Temperature::from(line.inherited_time_perc), &inh_time);
    heat_line(rustbox, 143, cursor.1, &Temperature::from(line.inherited_alloc_perc), &inh_alloc);
}
