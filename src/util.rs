use human_ids::{Options, generate};
use nanoid::nanoid;

const ALPHABET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub fn make_id() -> String {
    let friendly_id = generate(Some(Options::builder().separator("-").build()));
    let random_id = nanoid!(5, &ALPHABET);
    format!("{}-{}", friendly_id, random_id)
}
