use super::traceback;

static mut HAD_ERROR: bool = false;

pub fn had_error() -> bool {
    unsafe { HAD_ERROR }
}

fn set_had_error() {
    unsafe { HAD_ERROR = true }
}

pub fn report_trace(trace: traceback::Traceback) -> String {
    let mut s = String::new();
    let spaces = " ".repeat(count_digits(trace.pos.1+1));

    s.push_str(&format!("error[{}]: {}\n", trace.tipe, trace.message.unwrap_or("no message".to_string())));
    s.push_str("----- Traceback -----\n");
    s.push_str(&format!("{} ┌─ ", spaces.clone()));
    if let Some(file) = trace.filename {
        s.push_str(&format!("<{}>:", file));
    } else {
        s.push_str("<unknown>:");
    }
    s.push_str(&format!("{}:{}\n", trace.pos.1+1, trace.pos.0));
    if let Some(code) = &trace.code {
        for i in 0..2 {
            let line_nb :i32 = (trace.pos.1 + i) as i32 - 1;
            let line = if line_nb >= 0 { code.lines().nth(line_nb as usize).unwrap_or("").trim_end() } else { "" };
            let prefix = if i == 1 { (trace.pos.1+1).to_string() } else { spaces.clone() };
            s.push_str(&format!("{} | {}\n", prefix, line));
        }
        s.push_str(&format!("{} | {}\n", spaces.clone(), " ".repeat(trace.pos.0) + "^"));
    }

    set_had_error();

    s
}

fn count_digits(n: usize) -> usize {
    let mut n = n;
    let mut count = 0;
    while n > 0 {
        n /= 10;
        count += 1;
    }
    count
}
