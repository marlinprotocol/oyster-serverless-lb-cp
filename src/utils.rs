use std::net::ToSocketAddrs;

pub fn is_valid_ip_with_port(ip_port_str: &str) -> bool {
    if let Ok(socket_addr) = ip_port_str.to_socket_addrs() {
        for addr in socket_addr {
            if addr.is_ipv4() {
                return true;
            }
        }
    }
    false
}
