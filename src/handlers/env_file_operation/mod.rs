use std::env;

pub fn load_env_file_data() -> std::io::Result<()> {
    let file_data: String = std::fs::read_to_string("./static.env").unwrap();

    for line in file_data.lines() {

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let white_space_removed_line: String = line.chars()
            .filter(|xfs| !char::is_whitespace(*xfs) && *xfs != '"').collect();
        let (key, value): (&str, &str) = white_space_removed_line.split_once('=').unwrap();

        println!("Key: {} | Value: {}\n", key, value);

        unsafe {
            env::set_var(String::from(key), String::from(value));
        }
    }

    Ok(())
}