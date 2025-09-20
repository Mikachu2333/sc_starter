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
}
