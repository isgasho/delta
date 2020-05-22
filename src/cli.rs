use std::process;
use std::str::FromStr;
use std::string::ToString;

use console::Term;
use structopt::StructOpt;

use crate::bat::assets::HighlightingAssets;
use crate::bat::output::PagingMode;
use crate::config;
use crate::env;
use crate::style;

#[derive(StructOpt, Clone, Debug)]
#[structopt(
    name = "delta",
    about = "A syntax-highlighter for git and diff output",
    after_help = "\
Colors
------

All delta color options work the same way. There are three ways to specify a color:

1. RGB hex code

   An example of using an RGB hex code is:
   --file-color=\"#0e7c0e\"

2. ANSI color name

   There are 8 ANSI color names:
   black, red, green, yellow, blue, magenta, cyan, white.

   In addition, all of them have a bright form:
   bright-black, bright-red, bright-green, bright-yellow, bright-blue, bright-magenta, bright-cyan, bright-white

   An example of using an ANSI color name is:
   --file-color=\"green\"

   Unlike RGB hex codes, ANSI color names are just names: you can choose the exact color that each
   name corresponds to in the settings of your terminal application (the application you use to
   enter commands at a shell prompt). This means that if you use ANSI color names, and you change
   the color theme used by your terminal, then delta's colors will respond automatically, without
   needing to change the delta command line.

   \"purple\" is accepted as a synonym for \"magenta\". Color names and codes are case-insensitive.

3. ANSI color number

   An example of using an ANSI color number is:
   --file-color=28

   There are 256 ANSI color numbers: 0-255. The first 16 are the same as the colors described in
   the \"ANSI color name\" section above. See https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit.
   Specifying colors like this is useful if your terminal only supports 256 colors (i.e. doesn\'t
   support 24-bit color).
"
)]
pub struct Opt {
    /// Use default colors appropriate for a light terminal background. For more control, see the other
    /// color options.
    #[structopt(long = "light")]
    pub light: bool,

    /// Use default colors appropriate for a dark terminal background. For more control, see the
    /// other color options.
    #[structopt(long = "dark")]
    pub dark: bool,

    #[structopt(long = "minus-style", default_value = "normal auto")]
    /// The style for removed lines.
    pub minus_style: String,

    #[structopt(long = "minus-emph-style", default_value = "normal auto")]
    /// The style for emphasized sections of removed lines.
    pub minus_emph_style: String,

    #[structopt(long = "zero-style", default_value = "syntax normal")]
    /// The style for unchanged lines.
    pub zero_style: String,

    #[structopt(long = "plus-style", default_value = "syntax auto")]
    /// The style for removed lines.
    pub plus_style: String,

    #[structopt(long = "plus-emph-style", default_value = "syntax auto")]
    /// The style for emphasized sections of removed lines.
    pub plus_emph_style: String,

    #[structopt(long = "minus-color")]
    /// The background color for removed lines.
    pub deprecated_minus_background_color: Option<String>,

    #[structopt(long = "minus-emph-color")]
    /// The background color for emphasized sections of removed lines.
    pub deprecated_minus_emph_background_color: Option<String>,

    #[structopt(long = "plus-color")]
    /// The background color for added lines.
    pub deprecated_plus_background_color: Option<String>,

    #[structopt(long = "plus-emph-color")]
    /// The background color for emphasized sections of added lines.
    pub deprecated_plus_emph_background_color: Option<String>,

    #[structopt(long = "theme", env = "BAT_THEME")]
    /// The code syntax highlighting theme to use. Use --theme=none to disable syntax highlighting.
    /// If the theme is not set using this option, it will be taken from the BAT_THEME environment
    /// variable, if that contains a valid theme name. Use --list-themes to view available themes.
    /// Note that the choice of theme only affects code syntax highlighting. See --commit-color,
    /// --file-color, --hunk-color to configure the colors of other parts of the diff output.
    pub theme: Option<String>,

    #[structopt(long = "highlight-removed")]
    /// DEPRECATED: supply 'syntax' as the foreground color in --minus-style.
    pub deprecated_highlight_minus_lines: bool,

    #[structopt(long = "color-only")]
    /// Do not alter the input in any way other than applying colors. Equivalent to
    /// `--keep-plus-minus-markers --width variable --tabs 0 --commit-style plain
    ///  --file-style plain --hunk-style plain`.
    pub color_only: bool,

    #[structopt(long = "keep-plus-minus-markers")]
    /// Prefix added/removed lines with a +/- character, respectively, exactly as git does. The
    /// default behavior is to output a space character in place of these markers.
    pub keep_plus_minus_markers: bool,

    #[structopt(long = "commit-style", default_value = "plain")]
    /// Formatting style for the commit section of git output. Options
    /// are: plain, box.
    pub commit_style: SectionStyle,

    #[structopt(long = "commit-color", default_value = "yellow")]
    /// Color for the commit section of git output.
    pub commit_color: String,

    #[structopt(long = "file-style", default_value = "underline")]
    /// Formatting style for the file section of git output. Options
    /// are: plain, box, underline.
    pub file_style: SectionStyle,

    #[structopt(long = "file-color", default_value = "blue")]
    /// Color for the file section of git output.
    pub file_color: String,

    #[structopt(long = "hunk-style", default_value = "box")]
    /// Formatting style for the hunk-marker section of git output. Options
    /// are: plain, box.
    pub hunk_style: SectionStyle,

    #[structopt(long = "hunk-color", default_value = "blue")]
    /// Color for the hunk-marker section of git output.
    pub hunk_color: String,

    /// Use --width=variable to extend background colors to the end of each line only. Otherwise
    /// background colors extend to the full terminal width.
    #[structopt(short = "w", long = "width")]
    pub width: Option<String>,

    /// The number of spaces to replace tab characters with. Use --tabs=0 to pass tab characters
    /// through directly, but note that in that case delta will calculate line widths assuming tabs
    /// occupy one character's width on the screen: if your terminal renders tabs as more than than
    /// one character wide then delta's output will look incorrect.
    #[structopt(long = "tabs", default_value = "4")]
    pub tab_width: usize,

    /// Show the command-line arguments (RGB hex codes) for the background colors that are in
    /// effect. The hex codes are displayed with their associated background color. This option can
    /// be combined with --light and --dark to view the background colors for those modes. It can
    /// also be used to experiment with different RGB hex codes by combining this option with
    /// --minus-color, --minus-emph-color, --plus-color, --plus-emph-color.
    #[structopt(long = "show-background-colors")]
    pub show_background_colors: bool,

    /// List supported languages and associated file extensions.
    #[structopt(long = "list-languages")]
    pub list_languages: bool,

    /// List available syntax-highlighting color themes.
    #[structopt(long = "list-theme-names")]
    pub list_theme_names: bool,

    /// List available syntax highlighting themes, each with an example of highlighted diff output.
    /// If diff output is supplied on standard input then this will be used for the demo. For
    /// example: `git show --color=always | delta --list-themes`.
    #[structopt(long = "list-themes")]
    pub list_themes: bool,

    /// The maximum distance between two lines for them to be inferred to be homologous. Homologous
    /// line pairs are highlighted according to the deletion and insertion operations transforming
    /// one into the other.
    #[structopt(long = "max-line-distance", default_value = "0.3")]
    pub max_line_distance: f64,

    /// Whether to emit 24-bit ("true color") RGB color codes. Options are auto, always, and never.
    /// "auto" means that delta will emit 24-bit color codes iff the environment variable COLORTERM
    /// has the value "truecolor" or "24bit". If your terminal application (the application you use
    /// to enter commands at a shell prompt) supports 24 bit colors, then it probably already sets
    /// this environment variable, in which case you don't need to do anything.
    #[structopt(long = "24-bit-color", default_value = "auto")]
    pub true_color: String,

    /// Whether to use a pager when displaying output. Options are: auto, always, and never. The
    /// default pager is `less`: this can be altered by setting the environment variables BAT_PAGER
    /// or PAGER (BAT_PAGER has priority).
    #[structopt(long = "paging", default_value = "auto")]
    pub paging_mode: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SectionStyle {
    Box,
    Plain,
    Underline,
    Omit,
}

// TODO: clean up enum parsing and error handling

#[derive(Debug)]
pub enum Error {
    SectionStyleParseError,
}

impl FromStr for SectionStyle {
    type Err = Error;
    fn from_str(s: &str) -> Result<SectionStyle, Error> {
        match s.to_lowercase().as_str() {
            "box" => Ok(SectionStyle::Box),
            "plain" => Ok(SectionStyle::Plain),
            "underline" => Ok(SectionStyle::Underline),
            _ => Err(Error::SectionStyleParseError),
        }
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        "".to_string()
    }
}

pub fn process_command_line_arguments<'a>(mut opt: Opt) -> config::Config<'a> {
    let assets = HighlightingAssets::new();

    _check_validity(&opt, &assets);

    // Apply rewrite rules
    _rewrite_style_strings_to_honor_deprecated_options(&mut opt);
    _rewrite_options_to_implement_color_only(&mut opt);
    // We do not use the full width, in case `less --status-column` is in effect. See #41 and #10.
    // TODO: There seems to be some confusion in the accounting: we are actually leaving 2
    // characters unused for less at the right edge of the terminal, despite the subtraction of 1
    // here.
    let available_terminal_width = (Term::stdout().size().1 - 1) as usize;

    let paging_mode = match opt.paging_mode.as_ref() {
        "always" => PagingMode::Always,
        "never" => PagingMode::Never,
        "auto" => PagingMode::QuitIfOneScreen,
        _ => {
            eprintln!(
                "Invalid value for --paging option: {} (valid values are \"always\", \"never\", and \"auto\")",
                opt.paging_mode
            );
            process::exit(1);
        }
    };

    let true_color = match opt.true_color.as_ref() {
        "always" => true,
        "never" => false,
        "auto" => is_truecolor_terminal(),
        _ => {
            eprintln!(
                "Invalid value for --24-bit-color option: {} (valid values are \"always\", \"never\", and \"auto\")",
                opt.true_color
            );
            process::exit(1);
        }
    };

    config::get_config(
        opt,
        assets.syntax_set,
        assets.theme_set,
        true_color,
        available_terminal_width,
        paging_mode,
    )
}

fn _check_validity(opt: &Opt, assets: &HighlightingAssets) {
    if opt.light && opt.dark {
        eprintln!("--light and --dark cannot be used together.");
        process::exit(1);
    }
    if let Some(ref theme) = opt.theme {
        if !style::is_no_syntax_highlighting_theme_name(&theme) {
            if !assets.theme_set.themes.contains_key(theme.as_str()) {
                eprintln!("Invalid theme: '{}'", theme);
                process::exit(1);
            }
            let is_light_theme = style::is_light_theme(&theme);
            if is_light_theme && opt.dark {
                eprintln!(
                    "{} is a light theme, but you supplied --dark. \
                     If you use --theme, you do not need to supply --light or --dark.",
                    theme
                );
                process::exit(1);
            } else if !is_light_theme && opt.light {
                eprintln!(
                    "{} is a dark theme, but you supplied --light. \
                     If you use --theme, you do not need to supply --light or --dark.",
                    theme
                );
                process::exit(1);
            }
        }
    }
}

/// Implement --color-only
fn _rewrite_options_to_implement_color_only(opt: &mut Opt) {
    if opt.color_only {
        opt.keep_plus_minus_markers = true;
        opt.tab_width = 0;
        opt.commit_style = SectionStyle::Plain;
        opt.file_style = SectionStyle::Plain;
        opt.hunk_style = SectionStyle::Plain;
    }
}

/// Honor deprecated arguments by rewriting the canonical --*-style arguments if appropriate.
// TODO: How to avoid repeating the default values for style options here and in
// the structopt definition?
// If --highlight-removed was passed then we should set minus and minus emph foreground to "syntax", if they are still at their default values.
fn _rewrite_style_strings_to_honor_deprecated_options(opt: &mut Opt) {
    let deprecated_minus_foreground_arg = if opt.deprecated_highlight_minus_lines {
        Some("syntax")
    } else {
        None
    };

    if let Some(rewritten) = _rewrite_style_string_maybe(
        &opt.minus_style,
        ("normal", "auto"),
        (
            deprecated_minus_foreground_arg,
            opt.deprecated_minus_background_color.as_deref(),
        ),
        "minus",
    ) {
        opt.minus_style = rewritten.to_string();
    }
    if let Some(rewritten) = _rewrite_style_string_maybe(
        &opt.minus_emph_style,
        ("normal", "auto"),
        (
            deprecated_minus_foreground_arg,
            opt.deprecated_minus_emph_background_color.as_deref(),
        ),
        "minus-emph",
    ) {
        opt.minus_emph_style = rewritten.to_string();
    }
    if let Some(rewritten) = _rewrite_style_string_maybe(
        &opt.plus_style,
        ("syntax", "auto"),
        (None, opt.deprecated_plus_background_color.as_deref()),
        "plus",
    ) {
        opt.plus_style = rewritten.to_string();
    }
    if let Some(rewritten) = _rewrite_style_string_maybe(
        &opt.plus_emph_style,
        ("syntax", "auto"),
        (None, opt.deprecated_plus_emph_background_color.as_deref()),
        "plus-emph",
    ) {
        opt.plus_emph_style = rewritten.to_string();
    }
}

pub fn _rewrite_style_string_maybe(
    style: &str,
    style_default_pair: (&str, &str),
    deprecated_args_style_pair: (Option<&str>, Option<&str>),
    element_name: &str,
) -> Option<String> {
    let format_style = |pair: (&str, &str)| format!("{} {}", pair.0, pair.1);
    match (style, deprecated_args_style_pair) {
        (_, (None, None)) => None, // no rewrite
        (style, deprecated_args_style_pair) if style == format_style(style_default_pair) => {
            // TODO: We allow the deprecated argument values to have effect if
            // the style argument value is equal to its default value. This is
            // non-ideal, because the user may have explicitly supplied the
            // style argument (i.e. it might just happen to equal the default).
            Some(format_style((
                deprecated_args_style_pair.0.unwrap_or(style_default_pair.0),
                deprecated_args_style_pair.1.unwrap_or(style_default_pair.1),
            )))
        }
        (_, (_, Some(_))) => {
            eprintln!(
                "--{name}-color cannot be used with --{name}-style. \
                 Use --{name}-style=\"fg bg attr1 attr2 ...\" to set \
                 foreground color, background color, and style attributes. \
                 --{name}-color can only be used to set the background color. \
                 (It is still available for backwards-compatibility.)",
                name = element_name,
            );
            process::exit(1);
        }
        (_, (Some(_), None)) => {
            eprintln!(
                "Deprecated option --highlight-removed cannot be used with \
                 --{name}-style. Use --{name}-style=\"fg bg attr1 attr2 ...\" \
                 to set foreground color, background color, and style \
                 attributes.",
                name = element_name,
            );
            process::exit(1);
        }
    }
}

fn is_truecolor_terminal() -> bool {
    env::get_env_var("COLORTERM")
        .map(|colorterm| colorterm == "truecolor" || colorterm == "24bit")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::cli;
    use crate::style;
    use crate::tests::integration_test_utils::integration_test_utils;

    #[test]
    fn test_theme_selection() {
        #[derive(PartialEq)]
        enum Mode {
            Light,
            Dark,
        };
        for (
            theme_option,
            bat_theme_env_var,
            mode_option, // (--light, --dark)
            expected_theme,
            expected_mode,
        ) in vec![
            (None, "", None, style::DEFAULT_DARK_THEME, Mode::Dark),
            (Some("GitHub".to_string()), "", None, "GitHub", Mode::Light),
            (
                Some("GitHub".to_string()),
                "1337",
                None,
                "GitHub",
                Mode::Light,
            ),
            (None, "1337", None, "1337", Mode::Dark),
            (
                None,
                "<not set>",
                None,
                style::DEFAULT_DARK_THEME,
                Mode::Dark,
            ),
            (
                None,
                "",
                Some(Mode::Light),
                style::DEFAULT_LIGHT_THEME,
                Mode::Light,
            ),
            (
                None,
                "",
                Some(Mode::Dark),
                style::DEFAULT_DARK_THEME,
                Mode::Dark,
            ),
            (
                None,
                "<@@@@@>",
                Some(Mode::Light),
                style::DEFAULT_LIGHT_THEME,
                Mode::Light,
            ),
            (None, "1337", Some(Mode::Light), "1337", Mode::Light),
            (Some("none".to_string()), "", None, "none", Mode::Dark),
            (
                Some("None".to_string()),
                "",
                Some(Mode::Light),
                "None",
                Mode::Light,
            ),
        ] {
            if bat_theme_env_var == "<not set>" {
                env::remove_var("BAT_THEME")
            } else {
                env::set_var("BAT_THEME", bat_theme_env_var);
            }
            let is_true_color = true;
            let mut options = integration_test_utils::get_command_line_options();
            options.theme = theme_option;
            match mode_option {
                Some(Mode::Light) => {
                    options.light = true;
                    options.dark = false;
                }
                Some(Mode::Dark) => {
                    options.light = false;
                    options.dark = true;
                }
                None => {
                    options.light = false;
                    options.dark = false;
                }
            }
            let config = cli::process_command_line_arguments(options);
            assert_eq!(config.theme_name, expected_theme);
            if style::is_no_syntax_highlighting_theme_name(expected_theme) {
                assert!(config.theme.is_none())
            } else {
                assert_eq!(config.theme.unwrap().name.as_ref().unwrap(), expected_theme);
            }
            assert_eq!(
                config.minus_style.background.unwrap(),
                style::get_minus_background_color_default(
                    expected_mode == Mode::Light,
                    is_true_color
                )
            );
            assert_eq!(
                config.minus_emph_style.background.unwrap(),
                style::get_minus_emph_background_color_default(
                    expected_mode == Mode::Light,
                    is_true_color
                )
            );
            assert_eq!(
                config.plus_style.background.unwrap(),
                style::get_plus_background_color_default(
                    expected_mode == Mode::Light,
                    is_true_color
                )
            );
            assert_eq!(
                config.plus_emph_style.background.unwrap(),
                style::get_plus_emph_background_color_default(
                    expected_mode == Mode::Light,
                    is_true_color
                )
            );
        }
    }
}
