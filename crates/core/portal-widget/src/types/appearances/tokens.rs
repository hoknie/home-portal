use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! tokens {
    ($name:ident, $default:literal, [$($variant:ident => $text:literal),+ $(,)?]) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
        #[schemars(extend("x-open" = $default))]
        #[serde(rename_all = "kebab-case")]
        pub enum $name {
            $(
                #[serde(rename = $text)]
                $variant,
            )+
            #[default]
            #[serde(other)]
            #[schemars(skip)]
            Unknown,
        }

        impl $name {
            pub const NAMES: &'static str = concat!($($text, ", "),+);

            pub fn known(self) -> bool {
                self != $name::Unknown
            }
        }
    };
}

tokens!(Surface, "card", [Card => "card", Plain => "plain", Tinted => "tinted", Outline => "outline"]);
tokens!(Accent, "neutral", [
    Neutral => "neutral", Blue => "blue", Green => "green", Amber => "amber",
    Red => "red", Violet => "violet", Pink => "pink", Teal => "teal",
]);
tokens!(TitleVisibility, "shown", [Shown => "shown", Hidden => "hidden"]);
tokens!(Padding, "normal", [Normal => "normal", Compact => "compact", NoPadding => "none"]);
tokens!(Align, "start", [Start => "start", Center => "center", End => "end"]);
tokens!(SectionSurface, "none", [NoSurface => "none", SectionCard => "card"]);

pub fn names_of(names: &str) -> &str {
    names.trim_end_matches([',', ' '])
}
