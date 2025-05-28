use std::net::{IpAddr, Ipv4Addr};

use human_ids::{Options, generate};
use nanoid::nanoid;

use crate::error::GossipError;

const ALPHABET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub fn make_id() -> String {
    let friendly_id =
        generate(Some(Options::builder().separator("-").build()));
    let random_id = nanoid!(5, &ALPHABET);
    format!("{}-{}", friendly_id, random_id)
}

pub fn extract_ipv4(input: &str) -> Result<Ipv4Addr, GossipError> {
    // Split the string on comma and take the first part (IPv4)
    let ip_str = input.split(',').next().ok_or_else(|| {
        GossipError::IpAddressError("no ipv4 address".to_string())
    })?;

    // Trim whitespace and parse as IpAddr
    let ip_addr = ip_str.trim().parse::<IpAddr>().map_err(|_| {
        GossipError::IpAddressError(ip_str.to_string())
    })?;

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

// loop {
//     let conn = ts
//         .accept(listener)
//         .map_err(|e| GossipError::NetworkError(e))?;

//     println!("Accepted connection {:?}", conn);
//     let remote_addr = ts.get_remote_addr(conn, listener);
//     println!("Remote address: {:?}", remote_addr);

//     let socket = unsafe { UdpSocket::from_raw_fd(conn) };
//     println!("Socket: {:?}", socket);

//     let mut buf = [0; 1024];

//     match socket.recv(&mut buf) {
//         // match socket.recv_from(&mut buf) {
//         Ok(len) => {
//             // Convert received bytes to string and print
//             let received =
//                 String::from_utf8_lossy(&buf[..len]);
//             println!("Received: {}", received);
//         }
//         Err(e) => {
//             eprintln!("Error receiving data: {}", e);
//             break;
//         }
//     }
// }

// Ok(())
