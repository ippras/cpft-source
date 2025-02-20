use egui::Context;
use egui_l20n::{ContextExt as _, Localization};

/// Text
pub trait Text {
    fn text(&self) -> &'static str;

    fn hover_text(&self) -> &'static str;
}

/// Extension methods for [`Context`]
pub(crate) trait ContextExt {
    fn set_localizations(&self);
}

impl ContextExt for Context {
    fn set_localizations(&self) {
        self.set_localization(
            locales::EN,
            Localization::new(locales::EN).with_sources(sources::EN),
        );
        self.set_localization(
            locales::RU,
            Localization::new(locales::RU).with_sources(sources::RU),
        );
        self.set_language_identifier(locales::EN)
    }
}

mod locales {
    use egui_l20n::{LanguageIdentifier, langid};

    pub(super) const EN: LanguageIdentifier = langid!("en");
    pub(super) const RU: LanguageIdentifier = langid!("ru");
}

mod sources {
    macro_rules! source {
        ($path:literal) => {
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $path))
        };
    }

    pub(super) const EN: &[&str] = &[
        source!("/ftl/en/main.ftl"),
        source!("/ftl/en/params.ftl"),
        source!("/ftl/und/icons.ftl"),
    ];

    pub(super) const RU: &[&str] = &[
        source!("/ftl/ru/main.ftl"),
        source!("/ftl/ru/params.ftl"),
        source!("/ftl/und/icons.ftl"),
    ];
}
