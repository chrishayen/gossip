use std::net::{IpAddr, Ipv4Addr};

use human_ids::{Options, generate};
use nanoid::nanoid;

use crate::error::GossipError;

const ALPHABET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub fn make_id(prefix: &str) -> String {
    let friendly_id = generate(Some(
        Options::builder().separator("-").add_adverb(false).build(),
    ));
    let random_id = nanoid!(5, &ALPHABET);
    format!("{}-{}-{}", prefix, friendly_id, random_id).to_lowercase()
}

pub fn extract_ipv4(input: &str) -> Result<Ipv4Addr, GossipError> {
    // Split the string on comma and take the first part (IPv4)
    let ip_str = input.split(',').next().ok_or_else(|| {
        GossipError::IpAddressError("no ipv4 address".to_string())
    })?;

    // Trim whitespace and parse as IpAddr
    let ip_addr = ip_str
        .trim()
        .parse::<IpAddr>()
        .map_err(|_| GossipError::IpAddressError(ip_str.to_string()))?;

    // Ensure it's an IPv4 address
    match ip_addr {
        IpAddr::V4(ipv4) => Ok(ipv4),
        IpAddr::V6(_) => Err(GossipError::IpAddressError(
            "not an ipv4 address".to_string(),
        )),
    }
}

pub fn hash_node_name(name: &str) -> u32 {
    name.as_bytes()
        .iter()
        .fold(0u32, |acc, &b| acc.wrapping_add(b as u32))
}
