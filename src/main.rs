use clap::{Arg, Command};
use mdbook::errors::Error;
use mdbook::preprocess::CmdPreprocessor;
use mdbook::BookItem;
use semver::{Version, VersionReq};
use std::io;
use std::process;

const NAME: &str = "private-chapters";

pub fn make_app() -> Command {
    Command::new(NAME)
        .about("A mdbook preprocessor that removes chapters whose files begin with an underscore")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    if let Some(sub_matches) = matches.subcommand_matches("supports") {
        if sub_matches
            .get_one::<String>("renderer")
            .is_some_and(|renderer| supported_renderers().contains(&renderer.as_str()))
        {
            process::exit(0);
        } else {
            process::exit(1);
        }
    }

    if let Err(e) = process() {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn process() -> Result<(), Error> {
    let (ctx, mut book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;
    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            NAME,
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    if !should_export_private(&ctx) {
        book.sections.retain(|book_item| match book_item {
            BookItem::Chapter(chapter) => should_keep_chapter(chapter.source_path.as_deref()),
            _ => true,
        });
    }

    serde_json::to_writer(io::stdout(), &book)?;

    Ok(())
}

fn supported_renderers() -> &'static [&'static str] {
    &["html", "pdf", "epub"]
}

fn should_keep_chapter(source_path: Option<&std::path::Path>) -> bool {
    !source_path.is_some_and(|path| {
        path.file_name().is_some_and(|filename| filename.to_str().is_some_and(|name| name.starts_with("_")))
    })
}

fn should_export_private(ctx: &mdbook::preprocess::PreprocessorContext) -> bool {
    ctx.config
        .get_preprocessor(NAME)
        .is_some_and(|config| config.get("export-private").and_then(|value| value.as_bool()).unwrap_or(false))
        || std::env::var("MDBOOK_EXPORT_PRIVATE")
            .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}
