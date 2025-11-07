extern crate tauri_winres as winres;

fn main() {
    let time = vergen::BuildBuilder::default()
        .build_timestamp(true)
        .use_local(true)
        .build()
        .unwrap();

    vergen::Emitter::default()
        .add_instructions(&time)
        .unwrap()
        .emit()
        .unwrap();

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set("FileDescription", "Starter for ScreenCapture");
        res.set(
            "LegalCopyright",
            "Copyright (C) Mikachu2333 2025. MIT License.",
        );

        if let Err(e) = res.compile() {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
