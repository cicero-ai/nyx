// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use falcon_cli::*;
use rand::Rng;
use rand::rngs::OsRng;
use rand::seq::SliceRandom;

/// Check password from CLI input, auto-generate if needed
pub fn from_cli(allow_blank: bool) -> String {
    // Get password
    let input = cli_get_password("Password ('g' to generate): ", allow_blank);

    // Check for generate
    if input == "g" {
        return generate(24);
    } else if input == "r" {
        return generate_plain(24);
    }

    if input.len() < 4
        && (input.starts_with("g") || input.starts_with("r"))
        && let Ok(length) = &input[1..].parse::<usize>()
    {
        let password = if input.starts_with("g") {
            generate(*length)
        } else {
            generate_plain(*length)
        };
        return password;
    }

    input.to_string()
}

/// Generate a password of a specified length, ensuring it contains at least one number and one special character.
pub fn generate(length: usize) -> String {
    if length < 3 {
        panic!("Password length must be at least 3 to satisfy requirements");
    }

    // Set chars
    let all_chars =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    let numbers = b"0123456789";
    let special_chars = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

    let mut rng = OsRng;
    let mut password = Vec::with_capacity(length);

    // First pass
    for _ in 0..length {
        password.push(*all_chars.choose(&mut rng).unwrap());
    }

    // Ensure a digit
    let num_pos = rng.gen_range(0..length);
    password[num_pos] = *numbers.choose(&mut rng).unwrap();

    // Ensure special character
    let mut special_pos;
    loop {
        special_pos = rng.gen_range(0..length);
        if special_pos != num_pos {
            break;
        }
    }
    password[special_pos] = *special_chars.choose(&mut rng).unwrap();

    String::from_utf8(password).unwrap()
}

/// Generate a password of a specified length, ensuring it contains at least one number and one special character.
pub fn generate_plain(length: usize) -> String {
    let all_chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    let mut rng = OsRng;
    let mut password = Vec::with_capacity(length);

    // First pass
    for _ in 0..length {
        password.push(*all_chars.choose(&mut rng).unwrap());
    }

    String::from_utf8(password).unwrap()
}
