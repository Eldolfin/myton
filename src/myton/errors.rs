static mut HAD_ERROR: bool = false;

pub fn had_error() -> bool {
    unsafe { HAD_ERROR }
}

pub fn report_error(line: usize, file: &str, message: &str) {
    eprintln!("{}:{}: {}", file, line, message);
    unsafe {
        HAD_ERROR = true;
    }
}

pub fn report_error_at(y: usize, x: usize, file: &str, message: &str, line: &str) {
    eprintln!("{}:{}: {}", file, y, message);
    eprintln!("{} | {}", y, line.strip_suffix("\n").unwrap());
    eprintln!("{} | {}", " ", " ".repeat(x) + "^");
    unsafe {
        HAD_ERROR = true;
    }
}
