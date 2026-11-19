use nt_hook::diagnostics::append_line;

const WRITER_THREADS: usize = 8;
const LINES_PER_THREAD: usize = 400;
const LINE_PADDING_CHARS: usize = 120;

#[test]
fn lines_written_by_many_threads_at_once_stay_whole() {
    let path = std::env::temp_dir().join(format!("nt_hook_log_file_{}.log", std::process::id()));
    let path_text = path.to_str().expect("temp path is text").to_owned();
    let _ = std::fs::remove_file(&path);

    std::thread::scope(|scope| {
        for thread in 0..WRITER_THREADS {
            let path_text = &path_text;
            scope.spawn(move || {
                for index in 0..LINES_PER_THREAD {
                    let padding = "x".repeat(LINE_PADDING_CHARS);
                    append_line(path_text, &format!("T{thread}:{index}:{padding}:END"));
                }
            });
        }
    });

    let text = std::fs::read_to_string(&path).expect("log file is readable");
    let _ = std::fs::remove_file(&path);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), WRITER_THREADS * LINES_PER_THREAD);
    for line in lines {
        let parts: Vec<&str> = line.split(':').collect();
        assert_eq!(parts.len(), 4, "torn line: {line}");
        assert_eq!(parts[2].len(), LINE_PADDING_CHARS, "torn line: {line}");
        assert_eq!(parts[3], "END", "torn line: {line}");
    }
}
