match   format_file(filename.as_ref()) {
    Err(e) => {println!("{:?}", e); return}
    Ok(_) => {}
}