use std::ops::Range;

use iced::advanced::text::highlighter;
use iced::{font, Color, Font};
use once_cell::sync::Lazy;
use syntect::highlighting;
use syntect::parsing;

static SYNTAXES: Lazy<parsing::SyntaxSet> = Lazy::new(|| {
    parsing::SyntaxSet::load_from_folder(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).unwrap()
});

static THEMES: Lazy<highlighting::ThemeSet> = Lazy::new(highlighting::ThemeSet::load_defaults);

const LINES_PER_SNAPSHOT: usize = 50;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub theme: String,
    pub token: String,
    pub errors: Vec<Range<usize>>,
}

#[derive(Debug)]
pub struct Highlight(highlighting::StyleModifier);

impl Highlight {
    pub fn color(&self) -> Option<Color> {
        self.0.foreground.map(|color| {
            Color::from_rgba8(color.r, color.g, color.b, color.a as f32 / 255.0)
        })
    }

    pub fn font(&self) -> Option<Font> {
        self.0.font_style.and_then(|style| {
            let bold = style.contains(highlighting::FontStyle::BOLD);
            let italic = style.contains(highlighting::FontStyle::ITALIC);
            let underline = style.contains(highlighting::FontStyle::UNDERLINE);

            if bold || italic || underline {
                Some(Font {
                    weight: if bold {
                        font::Weight::Bold
                    } else {
                        font::Weight::Normal
                    },
                    style: if italic {
                        font::Style::Italic
                    } else {
                        font::Style::Normal
                    },
                    ..Font::MONOSPACE
                })
            } else {
                None
            }
        })
    }

    pub fn to_format(&self) -> highlighter::Format<Font> {
        highlighter::Format {
            color: self.color(),
            font: self.font(),
        }
    }
}

#[derive(Debug)]
pub struct Highlighter {
    syntax: &'static parsing::SyntaxReference,
    highlighter: highlighting::Highlighter<'static>,

    caches: Vec<(parsing::ParseState, parsing::ScopeStack)>,
    current_line: usize,

    errors: Vec<Range<usize>>,
}

impl iced::advanced::text::Highlighter for Highlighter {
    type Settings = Settings;
    type Highlight = Highlight;

    type Iterator<'a> =
        Box<dyn Iterator<Item = (Range<usize>, Self::Highlight)> + 'a>;

    fn new(settings: &Self::Settings) -> Self {
        let syntax = SYNTAXES
            .find_syntax_by_token(&settings.token)
            .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());

        let highlighter = highlighting::Highlighter::new(
            &THEMES.themes[&settings.theme],
        );

        let parser = parsing::ParseState::new(syntax);
        let stack = parsing::ScopeStack::new();

        Self {
            syntax,
            highlighter,
            caches: vec![(parser, stack)],
            current_line: 0,
            errors: settings.errors.clone(),
        }
    }

    fn update(&mut self, settings: &Self::Settings) {
        self.syntax = SYNTAXES
            .find_syntax_by_token(&settings.token)
            .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());

        self.highlighter = highlighting::Highlighter::new(
            &THEMES.themes[&settings.theme],
        );

        self.errors = settings.errors.clone();
        // Restart the highlighter
        self.change_line(0);
    }

    fn change_line(&mut self, line: usize) {
        let snapshot = line / LINES_PER_SNAPSHOT;

        if snapshot <= self.caches.len() {
            self.caches.truncate(snapshot);
            self.current_line = snapshot * LINES_PER_SNAPSHOT;
        } else {
            self.caches.truncate(1);
            self.current_line = 0;
        }

        let (parser, stack) =
            self.caches.last().cloned().unwrap_or_else(|| {
                (
                    parsing::ParseState::new(self.syntax),
                    parsing::ScopeStack::new(),
                )
            });

        self.caches.push((parser, stack));
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        if self.current_line / LINES_PER_SNAPSHOT >= self.caches.len() {
            let (parser, stack) =
                self.caches.last().expect("Caches must not be empty");

            self.caches.push((parser.clone(), stack.clone()));
        }

        self.current_line += 1;

        let (parser, stack) =
            self.caches.last_mut().expect("Caches must not be empty");

        let ops = parser.parse_line(line, &SYNTAXES).unwrap_or_default();

        let highlighter = &self.highlighter;
        let line_start = self.current_line - 1;
        let line_end = self.current_line;
        let line_length = line.len();

        let mut highlights = Vec::new();

        let syntax_highlights: Vec<_> = ScopeRangeIterator {
            ops,
            line_length,
            index: 0,
            last_str_index: 0,
        }
        .filter_map(|(range, scope)| {
            let _ = stack.apply(&scope);

            if range.is_empty() {
                None
            } else {
                Some((
                    range,
                    Highlight(highlighter.style_mod_for_stack(&stack.scopes)),
                ))
            }
        })
        .collect();

        for error_range in &self.errors {
            if error_range.start >= line_start && error_range.end <= line_end {
                let start = if error_range.start > line_start {
                    error_range.start - line_start
                } else {
                    0
                };
                let end = if error_range.end < line_end {
                    error_range.end - line_start
                } else {
                    line_length
                };

                let error_style = highlighting::StyleModifier {
                    foreground: Some(highlighting::Color {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255,
                    }),
                    font_style: Some(highlighting::FontStyle::UNDERLINE),
                    ..Default::default()
                };

                highlights.push((start..end, Highlight(error_style)));
            }
        }

        highlights.extend(syntax_highlights);

        Box::new(highlights.into_iter())
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

pub struct ScopeRangeIterator {
    ops: Vec<(usize, parsing::ScopeStackOp)>,
    line_length: usize,
    index: usize,
    last_str_index: usize,
}

impl Iterator for ScopeRangeIterator {
    type Item = (Range<usize>, parsing::ScopeStackOp);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.ops.len() {
            return None;
        }

        let next_str_i = if self.index == self.ops.len() {
            self.line_length
        } else {
            self.ops[self.index].0
        };

        let range = self.last_str_index..next_str_i;
        self.last_str_index = next_str_i;

        let op = if self.index == 0 {
            parsing::ScopeStackOp::Noop
        } else {
            self.ops[self.index - 1].1.clone()
        };

        self.index += 1;
        Some((range, op))
    }
}
