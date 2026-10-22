//! Markdown style configuration derived from a [`Theme`], mirroring `glamour`.
//!
//! The types match `ansi.StyleConfig` of `charm.land/glamour/v2`: same fields,
//! same order and the same JSON names. Serializing [`StyleConfig`] with
//! `serde_json` therefore gives the same text as `json.Marshal` in Go, which
//! is how the parity tests compare the two.
//!
//! This crate does not render markdown. The configuration is plain data that
//! any markdown renderer can consume.

use charm_core::Rgb;
use serde::{Deserialize, Serialize};

use crate::theme::Theme;

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(value: &u32) -> bool {
    *value == 0
}

/// Style of one text fragment, mirroring `ansi.StylePrimitive`.
///
/// Optional values are `None` when unset, so a renderer can tell "not set"
/// apart from "set to false".
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct StylePrimitive {
    /// Text printed before a block.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub block_prefix: String,
    /// Text printed after a block.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub block_suffix: String,
    /// Text printed before the fragment.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub prefix: String,
    /// Text printed after the fragment.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub suffix: String,
    /// Foreground color as `#rrggbb`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Background color as `#rrggbb`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    /// Underlined text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
    /// Bold text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    /// Uppercase text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper: Option<bool>,
    /// Lowercase text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower: Option<bool>,
    /// Title-cased text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<bool>,
    /// Italic text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    /// Struck-through text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossed_out: Option<bool>,
    /// Faint text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faint: Option<bool>,
    /// Concealed text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conceal: Option<bool>,
    /// Inverted colors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inverse: Option<bool>,
    /// Blinking text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blink: Option<bool>,
    /// Template applied to the text, such as `"Image: {{.text}} "`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub format: String,
}

/// Style of a task list item, mirroring `ansi.StyleTask`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct StyleTask {
    /// Style shared with the other primitives.
    #[serde(flatten)]
    pub primitive: StylePrimitive,
    /// Marker for a completed task.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ticked: String,
    /// Marker for a pending task.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub unticked: String,
}

/// Style of a block element, mirroring `ansi.StyleBlock`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct StyleBlock {
    /// Style shared with the other primitives.
    #[serde(flatten)]
    pub primitive: StylePrimitive,
    /// Indentation width in columns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent: Option<u32>,
    /// Text repeated to build the indentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_token: Option<String>,
    /// Margin width in columns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<u32>,
}

/// Style of a code block, mirroring `ansi.StyleCodeBlock`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct StyleCodeBlock {
    /// Block style of the code block.
    #[serde(flatten)]
    pub block: StyleBlock,
    /// Name of the syntax highlighting theme.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub theme: String,
    /// Syntax highlighting colors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroma: Option<Chroma>,
}

/// Style of a list, mirroring `ansi.StyleList`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct StyleList {
    /// Block style of the list.
    #[serde(flatten)]
    pub block: StyleBlock,
    /// Indentation added for each nesting level.
    #[serde(skip_serializing_if = "is_zero")]
    pub level_indent: u32,
}

/// Style of a table, mirroring `ansi.StyleTable`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct StyleTable {
    /// Block style of the table.
    #[serde(flatten)]
    pub block: StyleBlock,
    /// Character used at the crossings of the grid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center_separator: Option<String>,
    /// Character used for vertical lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_separator: Option<String>,
    /// Character used for horizontal lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_separator: Option<String>,
}

/// Syntax highlighting styles of a code block, mirroring `ansi.Chroma`.
///
/// Each field is the style of one kind of token.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Chroma {
    /// Plain text.
    pub text: StylePrimitive,
    /// Error tokens.
    pub error: StylePrimitive,
    /// Comments.
    pub comment: StylePrimitive,
    /// Preprocessor comments.
    pub comment_preproc: StylePrimitive,
    /// Keywords.
    pub keyword: StylePrimitive,
    /// Reserved keywords.
    pub keyword_reserved: StylePrimitive,
    /// Namespace keywords.
    pub keyword_namespace: StylePrimitive,
    /// Type keywords.
    pub keyword_type: StylePrimitive,
    /// Operators.
    pub operator: StylePrimitive,
    /// Punctuation.
    pub punctuation: StylePrimitive,
    /// Names.
    pub name: StylePrimitive,
    /// Built-in names.
    pub name_builtin: StylePrimitive,
    /// Tag names.
    pub name_tag: StylePrimitive,
    /// Attribute names.
    pub name_attribute: StylePrimitive,
    /// Class names.
    pub name_class: StylePrimitive,
    /// Constant names.
    pub name_constant: StylePrimitive,
    /// Decorator names.
    pub name_decorator: StylePrimitive,
    /// Exception names.
    pub name_exception: StylePrimitive,
    /// Function names.
    pub name_function: StylePrimitive,
    /// Other names.
    pub name_other: StylePrimitive,
    /// Literals.
    pub literal: StylePrimitive,
    /// Number literals.
    pub literal_number: StylePrimitive,
    /// Date literals.
    pub literal_date: StylePrimitive,
    /// String literals.
    pub literal_string: StylePrimitive,
    /// Escape sequences inside string literals.
    pub literal_string_escape: StylePrimitive,
    /// Deleted lines in a diff.
    pub generic_deleted: StylePrimitive,
    /// Emphasized generic text.
    pub generic_emph: StylePrimitive,
    /// Inserted lines in a diff.
    pub generic_inserted: StylePrimitive,
    /// Strong generic text.
    pub generic_strong: StylePrimitive,
    /// Subheadings.
    pub generic_subheading: StylePrimitive,
    /// Background of the whole block.
    pub background: StylePrimitive,
}

/// Complete markdown style configuration, mirroring `ansi.StyleConfig`.
///
/// Build it from a theme with [`Theme::glamour`]. Every field is public, so a
/// caller can adjust a single element before rendering.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// let config = themes::dracula().glamour();
/// let json = serde_json::to_string(&config).expect("serializable");
/// assert!(json.contains("\"block_prefix\":\"\\n\""));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct StyleConfig {
    /// Whole document.
    pub document: StyleBlock,
    /// Block quotes.
    pub block_quote: StyleBlock,
    /// Paragraphs.
    pub paragraph: StyleBlock,
    /// Lists.
    pub list: StyleList,

    /// Headings without a level-specific style.
    pub heading: StyleBlock,
    /// Level 1 headings.
    pub h1: StyleBlock,
    /// Level 2 headings.
    pub h2: StyleBlock,
    /// Level 3 headings.
    pub h3: StyleBlock,
    /// Level 4 headings.
    pub h4: StyleBlock,
    /// Level 5 headings.
    pub h5: StyleBlock,
    /// Level 6 headings.
    pub h6: StyleBlock,

    /// Plain text.
    pub text: StylePrimitive,
    /// Struck-through text.
    pub strikethrough: StylePrimitive,
    /// Emphasized text.
    pub emph: StylePrimitive,
    /// Strong text.
    pub strong: StylePrimitive,
    /// Horizontal rules (`hr` in JSON).
    #[serde(rename = "hr")]
    pub horizontal_rule: StylePrimitive,

    /// Bullet list items.
    pub item: StylePrimitive,
    /// Numbered list items.
    pub enumeration: StylePrimitive,
    /// Task list items.
    pub task: StyleTask,

    /// Link destinations.
    pub link: StylePrimitive,
    /// Link text.
    pub link_text: StylePrimitive,

    /// Image destinations.
    pub image: StylePrimitive,
    /// Image alternative text.
    pub image_text: StylePrimitive,

    /// Inline code.
    pub code: StyleBlock,
    /// Fenced code blocks.
    pub code_block: StyleCodeBlock,

    /// Tables.
    pub table: StyleTable,

    /// Definition lists.
    pub definition_list: StyleBlock,
    /// Definition terms.
    pub definition_term: StylePrimitive,
    /// Definition descriptions.
    pub definition_description: StylePrimitive,

    /// Raw HTML blocks.
    pub html_block: StyleBlock,
    /// Inline raw HTML.
    pub html_span: StyleBlock,
}

fn hex(color: Rgb) -> Option<String> {
    Some(color.to_hex())
}

fn colored(color: Rgb) -> StylePrimitive {
    StylePrimitive {
        color: hex(color),
        ..StylePrimitive::default()
    }
}

fn bold_colored(color: Rgb) -> StylePrimitive {
    StylePrimitive {
        color: hex(color),
        bold: Some(true),
        ..StylePrimitive::default()
    }
}

fn bold() -> StylePrimitive {
    StylePrimitive {
        bold: Some(true),
        ..StylePrimitive::default()
    }
}

impl StyleConfig {
    /// Builds the markdown configuration from the colors of `theme`.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn from_theme(t: &Theme) -> Self {
        Self {
            document: StyleBlock {
                primitive: StylePrimitive {
                    block_prefix: "\n".to_owned(),
                    block_suffix: "\n".to_owned(),
                    color: hex(t.foreground),
                    ..StylePrimitive::default()
                },
                margin: Some(2),
                ..StyleBlock::default()
            },
            block_quote: StyleBlock {
                indent: Some(1),
                indent_token: Some("â”‚ ".to_owned()),
                ..StyleBlock::default()
            },
            paragraph: StyleBlock::default(),
            list: StyleList {
                level_indent: 2,
                ..StyleList::default()
            },
            heading: StyleBlock {
                primitive: StylePrimitive {
                    block_suffix: "\n".to_owned(),
                    color: hex(t.primary),
                    bold: Some(true),
                    ..StylePrimitive::default()
                },
                ..StyleBlock::default()
            },
            h1: StyleBlock {
                primitive: StylePrimitive {
                    prefix: " ".to_owned(),
                    suffix: " ".to_owned(),
                    color: hex(t.primary),
                    background_color: hex(t.background),
                    bold: Some(true),
                    ..StylePrimitive::default()
                },
                ..StyleBlock::default()
            },
            h2: StyleBlock {
                primitive: bold_colored(t.primary),
                ..StyleBlock::default()
            },
            h3: StyleBlock {
                primitive: bold_colored(t.secondary),
                ..StyleBlock::default()
            },
            h4: StyleBlock {
                primitive: bold_colored(t.accent),
                ..StyleBlock::default()
            },
            h5: StyleBlock {
                primitive: bold_colored(t.muted),
                ..StyleBlock::default()
            },
            h6: StyleBlock {
                primitive: colored(t.muted),
                ..StyleBlock::default()
            },
            text: colored(t.foreground),
            strikethrough: StylePrimitive {
                crossed_out: Some(true),
                ..StylePrimitive::default()
            },
            emph: StylePrimitive {
                italic: Some(true),
                ..StylePrimitive::default()
            },
            strong: bold(),
            horizontal_rule: StylePrimitive {
                color: hex(t.muted),
                format: "\n--------\n".to_owned(),
                ..StylePrimitive::default()
            },
            item: StylePrimitive {
                block_prefix: "â€¢ ".to_owned(),
                ..StylePrimitive::default()
            },
            enumeration: StylePrimitive {
                block_prefix: ". ".to_owned(),
                ..StylePrimitive::default()
            },
            task: StyleTask {
                ticked: "[âœ“] ".to_owned(),
                unticked: "[ ] ".to_owned(),
                ..StyleTask::default()
            },
            link: StylePrimitive {
                color: hex(t.info),
                underline: Some(true),
                ..StylePrimitive::default()
            },
            link_text: bold_colored(t.accent),
            image: StylePrimitive {
                color: hex(t.accent),
                underline: Some(true),
                ..StylePrimitive::default()
            },
            image_text: StylePrimitive {
                color: hex(t.muted),
                format: "Image: {{.text}} ".to_owned(),
                ..StylePrimitive::default()
            },
            code: StyleBlock {
                primitive: StylePrimitive {
                    prefix: "`".to_owned(),
                    suffix: "`".to_owned(),
                    color: hex(t.syntax.string),
                    background_color: hex(t.background),
                    ..StylePrimitive::default()
                },
                ..StyleBlock::default()
            },
            code_block: StyleCodeBlock {
                block: StyleBlock {
                    primitive: colored(t.foreground),
                    margin: Some(2),
                    ..StyleBlock::default()
                },
                chroma: Some(chroma(t)),
                ..StyleCodeBlock::default()
            },
            table: StyleTable::default(),
            definition_list: StyleBlock::default(),
            definition_term: bold_colored(t.primary),
            definition_description: StylePrimitive {
                block_prefix: "\nâ†’ ".to_owned(),
                ..StylePrimitive::default()
            },
            html_block: StyleBlock::default(),
            html_span: StyleBlock::default(),
        }
    }
}

fn chroma(t: &Theme) -> Chroma {
    let s = &t.syntax;
    Chroma {
        text: colored(t.foreground),
        error: colored(t.error),
        comment: colored(s.comment),
        comment_preproc: colored(s.operator),
        keyword: colored(s.keyword),
        keyword_reserved: colored(s.keyword),
        keyword_namespace: colored(s.r#type),
        keyword_type: colored(s.r#type),
        operator: colored(s.operator),
        punctuation: colored(s.constant),
        name: colored(t.foreground),
        name_builtin: colored(s.function),
        name_tag: colored(s.r#type),
        name_attribute: colored(s.constant),
        name_class: bold_colored(s.r#type),
        name_constant: colored(s.constant),
        name_decorator: colored(s.function),
        name_exception: colored(s.keyword),
        name_function: colored(s.function),
        name_other: colored(t.foreground),
        literal: colored(s.constant),
        literal_number: colored(s.number),
        literal_date: colored(s.constant),
        literal_string: colored(s.string),
        literal_string_escape: colored(s.operator),
        generic_deleted: colored(t.error),
        generic_emph: StylePrimitive {
            italic: Some(true),
            ..StylePrimitive::default()
        },
        generic_inserted: colored(t.success),
        generic_strong: bold(),
        generic_subheading: bold_colored(t.secondary),
        background: StylePrimitive {
            background_color: hex(t.background),
            ..StylePrimitive::default()
        },
    }
}
