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
    s.push_str("     !!!ERROR!!!\n");
    s.push_str("----- Traceback -----\n");
    if let Some(file) = trace.filename {
        s.push_str(&format!("{}:", file));
    } else {
        s.push_str("(unknown):");
    }
    s.push_str(&format!("{}:{}", trace.pos.1, trace.pos.0));
    if let Some(message) = trace.message {
        s.push_str(&format!(": {}", message));
    }
    s.push_str(&format!("\n"));
    
    if let Some(code) = trace.code {
        s.push_str(&format!("{} | {}\n", trace.pos.1, code.trim_end()));
        s.push_str(&format!("{} | {}\n", " ", " ".repeat(trace.pos.0) + "^"));
    }

    set_had_error();

    s
}
