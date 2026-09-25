use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    #[default]
    En,
    Ru,
    Es,
}

impl Language {
    pub const ALL: [Language; 3] = [Language::En, Language::Ru, Language::Es];
    pub const PROBLEM: &'static str = "must be one of en, ru, es";
    const ANY: &'static str = "*";

    pub fn code(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Ru => "ru",
            Language::Es => "es",
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Language::En => "English",
            Language::Ru => "Русский",
            Language::Es => "Español",
        }
    }

    pub fn parse(text: &str) -> Option<Language> {
        let text = text.trim();
        Self::ALL
            .into_iter()
            .find(|language| language.code().eq_ignore_ascii_case(text))
    }

    pub fn negotiate(accept_language: Option<&str>, fallback: Language) -> Language {
        let mut ranked = accept_language
            .unwrap_or_default()
            .split(',')
            .enumerate()
            .filter_map(|(position, item)| {
                let mut parts = item.split(';');
                let tag = parts.next()?.trim();
                let quality = parts
                    .filter_map(|parameter| parameter.trim().strip_prefix("q="))
                    .find_map(|value| value.trim().parse::<f32>().ok())
                    .unwrap_or(1.0);
                let language = if tag == Self::ANY {
                    Some(fallback)
                } else {
                    Self::parse(tag.split('-').next().unwrap_or_default())
                };
                language
                    .filter(|_| quality > 0.0)
                    .map(|language| (quality, position, language))
            })
            .collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            right
                .0
                .total_cmp(&left.0)
                .then_with(|| left.1.cmp(&right.1))
        });
        ranked
            .first()
            .map_or(fallback, |(_, _, language)| *language)
    }
}

impl fmt::Display for Language {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}
