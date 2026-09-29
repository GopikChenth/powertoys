fn main() {
    let style = std::env::var("SLINT_STYLE").unwrap_or_else(|_| "material".into());
    slint_build::compile_with_config(
        "ui/run.slint",
        slint_build::CompilerConfiguration::new().with_style(style),
    )
    .expect("Failed to compile run.slint");
}
