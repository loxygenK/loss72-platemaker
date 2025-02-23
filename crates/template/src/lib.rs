#![deny(clippy::unwrap_used)]

use std::sync::LazyLock;

use regex::{Captures, Regex};

pub const REGEX_CAPTURE_GROUP: &str = "name";

#[derive(Debug)]
pub struct Placeholder {
    regex: Regex,
}

#[derive(Debug, thiserror::Error)]
pub enum PlaceholderError {
    #[error("Invalid regex ({context}): {regex}")]
    InvalidRegex {
        context: &'static str,
        regex: regex::Error,
    },

    #[error("The regex must have a capture group with name '{REGEX_CAPTURE_GROUP}'.")]
    CaptureGroupNotFound,
}

impl Placeholder {
    pub fn from_strs(
        start: &str,
        end: &str,
        content_regex: impl Into<Option<Regex>>,
    ) -> Result<Self, PlaceholderError> {
        static ESCAPE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"([\[\]{}()^$])").expect("Statically provided regex to be valid")
        });

        let [start, end] =
            [start, end].map(|parenthesis| ESCAPE_REGEX.replace_all(parenthesis, r"\$1"));

        let content_regex = content_regex
            .into()
            .map_or_else(|| Regex::new(&format!(r"[^{end}]*?")), Ok)
            .map_err(|e| PlaceholderError::InvalidRegex {
                context:
                    "Default 'content_regex' could not be built. Specify 'content_regex' manually.",
                regex: e,
            })?;

        let regex = Regex::new(&format!(
            r"{start}\s*(?<{REGEX_CAPTURE_GROUP}>{content_regex})\s*{end}"
        ))
            .map_err(|e| PlaceholderError::InvalidRegex {
                context: "Placeholder regex could not be built. You might have to use `Placeholder::from_regex` instead.",
                regex: e
            })?;

        Self::from_regex(regex)
    }

    pub fn from_regex(regex: Regex) -> Result<Self, PlaceholderError> {
        if !regex
            .capture_names()
            .any(|name| name.is_some_and(|name| name == REGEX_CAPTURE_GROUP))
        {
            return Err(PlaceholderError::CaptureGroupNotFound);
        }

        Ok(Self { regex })
    }

    // These lifetimes will vanish at Edition 2024, its rustc is smart
    pub fn parse_used_placeholders<'iter, 'haystack: 'iter>(
        &'iter self,
        haystack: &'haystack str,
    ) -> impl Iterator<Item = String> + 'iter {
        self.regex.captures_iter(haystack).map(|capture| {
            let placeholder = capture
                .name(REGEX_CAPTURE_GROUP)
                .expect("Regex is validated to include the capture group");
            placeholder.as_str().to_string()
        })
    }

    pub fn fill_placeholders(
        &self,
        haystack: &str,
        mut filler: impl FnMut(&str) -> String,
    ) -> String {
        self.regex
            .replace_all(haystack, |capture: &Captures| -> String {
                let name = capture
                    .name(REGEX_CAPTURE_GROUP)
                    .expect("Regex is validated to include the capture group")
                    .as_str();
                filler(name)
            })
            .to_string()
    }

    pub fn partially_fill_placeholders(
        &self,
        haystack: &str,
        mut filler: impl FnMut(&str) -> Option<String>,
    ) -> Result<String, Vec<String>> {
        let mut failed_replaces: Vec<String> = vec![];

        let replaced = self
            .regex
            .replace_all(haystack, |capture: &Captures| -> String {
                let name = capture
                    .name(REGEX_CAPTURE_GROUP)
                    .expect("Regex is validated to include the capture group")
                    .as_str();

                let Some(replace_by) = filler(name) else {
                    failed_replaces.push(name.to_string());
                    return "".to_string();
                };

                replace_by
            });

        if failed_replaces.is_empty() {
            Ok(replaced.to_string())
        } else {
            Err(failed_replaces)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use regex::Regex;

    use super::Placeholder;

    #[test]
    pub fn shell_style() {
        let placeholder = Placeholder::from_strs("${", "}", None)
            .expect("Placeholder::from_strs not to error for valid arguments.");

        println!("{}", placeholder.regex.as_str());

        assert_eq!(
            &placeholder
                .parse_used_placeholders("${a}, ${b}, ${c}")
                .collect::<Vec<_>>(),
            &["a", "b", "c"]
        );
        assert_eq!(
            &placeholder
                .parse_used_placeholders("${ a }, ${ b }, ${ c }")
                .collect::<Vec<_>>(),
            &["a", "b", "c"]
        );
        assert_eq!(
            &placeholder
                .parse_used_placeholders("${a}, ${b}, ${a}")
                .collect::<Vec<_>>(),
            &["a", "b", "a"]
        );
    }

    #[test]
    pub fn shell_style_with_numeric_only() {
        let placeholder = Placeholder::from_strs(
            "${",
            "}",
            Regex::new("[0-9]+").expect("[0-9]+ to be a valid regex"),
        )
        .expect("Placeholder::from_strs not to error for valid arguments.");
        println!("{}", placeholder.regex.as_str());

        assert_eq!(
            &placeholder
                .parse_used_placeholders("${a}, ${b}, ${1}, ${1234567890} ${a}")
                .collect::<Vec<_>>(),
            &["1", "1234567890"]
        );
    }

    #[test]
    pub fn fill_shell_style_placeholder() {
        let replace_dataset = HashMap::from([("title", "foo"), ("description", "bar")]);

        let placeholder = Placeholder::from_strs("${", "}", None)
            .expect("Placeholder::from_strs not to error for valid arguments.");

        assert_eq!(
            placeholder.partially_fill_placeholders(
                "Title: ${title}, Description: ${description}",
                |name| { replace_dataset.get(name).map(|value| value.to_string()) }
            ),
            Ok("Title: foo, Description: bar".to_string()),
        );
    }
}
