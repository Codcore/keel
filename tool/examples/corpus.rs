// A throwaway binary: run the real stripper over a real corpus and
// count declarations that vanish.
use std::path::Path;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = Path::new(&args[1]);
    let ext = &args[2];
    let tongue = &args[3];
    let mut files = 0usize;
    let mut changed = 0usize;
    let mut lost_files = 0usize;
    let mut lost_total = 0usize;
    let mut worst: Vec<(usize, String)> = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|x| x.to_str()) != Some(ext.as_str()) {
                continue;
            }
            let Ok(src) = std::fs::read_to_string(&p) else {
                continue;
            };
            files += 1;
            let bare = keel::holding::strip_for_test(&src, tongue);
            if bare != src {
                changed += 1;
            }
            let decl = |text: &str| -> usize {
                text.lines()
                    .filter(|l| {
                        let t = l.trim();
                        match tongue.as_str() {
                            "ruby" => t.starts_with("def ") || t.starts_with("def\t"),
                            "elixir" => {
                                t.starts_with("def ")
                                    || t.starts_with("defp ")
                                    || t.starts_with("defmodule ")
                            }
                            _ => {
                                t.starts_with("pub fn ")
                                    || t.starts_with("fn ")
                                    || t.starts_with("pub struct ")
                                    || t.starts_with("pub enum ")
                                    || t.starts_with("pub trait ")
                                    || t.starts_with("pub const ")
                            }
                        }
                    })
                    .count()
            };
            let before = decl(&src);
            let after = decl(&bare);
            if after < before {
                lost_files += 1;
                lost_total += before - after;
                worst.push((before - after, p.display().to_string()));
            }
        }
    }
    worst.sort_by(|a, b| b.0.cmp(&a.0));
    println!("files scanned: {files}");
    println!("files whose text changed: {changed}");
    println!("files LOSING a declaration: {lost_files} (total lost: {lost_total})");
    for (n, f) in worst.iter().take(12) {
        println!("   -{n}  {f}");
    }
}
