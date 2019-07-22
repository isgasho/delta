use unicode_segmentation::UnicodeSegmentation;
pub fn delta<I>(
    lines: I,
) -> std::io::Result<()>
where
    I: Iterator<Item = String>,
{
    let mut painter = Painter::new(writer, config, assets);
        if line.starts_with("commit ") {
                handle_commit_meta_header_line(&mut painter, &raw_line, config)?;
        } else if line.starts_with("diff --git ") {
        } else if (line.starts_with("--- ") || line.starts_with("rename from "))
            && config.opt.file_style != cli::SectionStyle::Plain
        {
            minus_file = parse::get_file_path_from_file_meta_line(&line);
        } else if (line.starts_with("+++ ") || line.starts_with("rename to "))
            && config.opt.file_style != cli::SectionStyle::Plain
        {
            plus_file = parse::get_file_path_from_file_meta_line(&line);
            handle_file_meta_header_line(&mut painter, &minus_file, &plus_file, config)?;
        } else if line.starts_with("@@ ") {
                handle_hunk_meta_line(&mut painter, &line, config)?;
            state = handle_hunk_line(&mut painter, &line, state, config);
fn handle_commit_meta_header_line(
fn handle_file_meta_header_line(
fn handle_hunk_meta_line(
    painter: &mut Painter,
    line: &str,
    config: &Config,
) -> std::io::Result<()> {
/// Handle a hunk line, i.e. a minus line, a plus line, or an unchanged line.
// In the case of a minus or plus line, we store the line in a
// buffer. When we exit the changed region we process the collected
// minus and plus lines jointly, in order to paint detailed
// highlighting according to inferred edit operations. In the case of
// an unchanged line, we paint it immediately.
fn handle_hunk_line(painter: &mut Painter, line: &str, state: State, config: &Config) -> State {
    // Don't let the line buffers become arbitrarily large -- if we
    // were to allow that, then for a large deleted/added file we
    // would process the entire file before painting anything.
    if painter.minus_lines.len() > config.max_buffered_lines
        || painter.plus_lines.len() > config.max_buffered_lines
    {
        painter.paint_buffered_lines();
    }
    let line_length = line.graphemes(true).count();
        Some(width) if width > line_length => {
            format!("{}{}\n", line, " ".repeat(width - line_length))

#[cfg(test)]
mod tests {
    use super::*;
    use console::strip_ansi_codes;
    use structopt::StructOpt;

    #[test]
    fn test_added_file() {
        let input = "\
commit d28dc1ac57e53432567ec5bf19ad49ff90f0f7a5
Author: Dan Davison <dandavison7@gmail.com>
Date:   Thu Jul 11 10:41:11 2019 -0400

    .

diff --git a/a.py b/a.py
new file mode 100644
index 0000000..8c55b7d
--- /dev/null
+++ b/a.py
@@ -0,0 +1,3 @@
+# hello
+class X:
+    pass";

        let expected_output = "\
commit d28dc1ac57e53432567ec5bf19ad49ff90f0f7a5
Author: Dan Davison <dandavison7@gmail.com>
Date:   Thu Jul 11 10:41:11 2019 -0400

    .

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
added: a.py
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
────────────────────────────────────────────────────────────────────────────────

────────────────────────────────────────────────────────────────────────────────
 # hello
 class X:
     pass
";

        let mut opt = cli::Opt::from_args();
        opt.width = Some("variable".to_string());
        let assets = HighlightingAssets::new();
        let config = cli::process_command_line_arguments(&assets, &opt);
        let mut writer: Vec<u8> = Vec::new();
        delta(
            input.split("\n").map(String::from),
            &config,
            &assets,
            &mut writer,
        )
        .unwrap();
        let output = strip_ansi_codes(&String::from_utf8(writer).unwrap()).to_string();
        assert!(output.contains("\nadded: a.py\n"));
        if false {
            // TODO: hline width
            assert_eq!(output, expected_output);
        }
    }

    #[test]
    fn test_renamed_file() {
        let input = "\
commit 1281650789680f1009dfff2497d5ccfbe7b96526
Author: Dan Davison <dandavison7@gmail.com>
Date:   Wed Jul 17 20:40:23 2019 -0400

    rename

diff --git a/a.py b/b.py
similarity index 100%
rename from a.py
rename to b.py
";

        let mut opt = cli::Opt::from_args();
        opt.width = Some("variable".to_string());
        let assets = HighlightingAssets::new();
        let config = cli::process_command_line_arguments(&assets, &opt);
        let mut writer: Vec<u8> = Vec::new();
        delta(
            input.split("\n").map(String::from),
            &config,
            &assets,
            &mut writer,
        )
        .unwrap();
        let output = strip_ansi_codes(&String::from_utf8(writer).unwrap()).to_string();
        assert!(output.contains("\nrenamed: a.py ⟶   b.py\n"));
    }
}