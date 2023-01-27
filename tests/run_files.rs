use ::myton::run_to_string;
use snailquote::escape;
use std::env::args;
use walkdir::{self, WalkDir};

#[test]
fn test_files() {
    // finds recursively all files in the tests directory
    // ending with .my and executes them
    // then compares the output with the content of the .out file

    let files = WalkDir::new("tests")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().unwrap_or_default() == "my")
        .collect::<Vec<_>>();

    for file in files {
        let path = file.path().to_str().unwrap();

        let content = std::fs::read_to_string(path).unwrap();

        let output = run_to_string(content);

        let out_path = path.replace(".my", ".out");

        if args().any(|x| x == "--update") {
            std::fs::write(out_path, output).unwrap();
        } else {
            if let Ok(expected) = std::fs::read_to_string(out_path) {
                let message = format!(
                    "\nfile: {}\nexpected:\n{}\ngot:\n{}",
                    path, &expected, &output
                );
                assert_eq!(output, expected, "{}", message);
            } else {
                //panic!("No .out file found for {}", path);
            }
        }
    }
}
