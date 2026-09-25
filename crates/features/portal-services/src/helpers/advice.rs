use portal_model::Diagnosis;

pub fn advice(diagnosis: Diagnosis) -> &'static str {
    match diagnosis {
        Diagnosis::LocalNetworkDenied => {
            "macOS refused this process access to the local network. Allow it in System Settings > Privacy & Security > Local Network for the app that starts the portal (your terminal, or the portal itself when launchd starts it), then probe again."
        }
        Diagnosis::Refused => {
            "The host answered, but nothing listens on that port. Check the port and that the service is running."
        }
        Diagnosis::Timeout => {
            "No answer within the timeout. The host may be off, a firewall may drop the packets, or the timeout is too short."
        }
        Diagnosis::HostUnreachable => {
            "There is no route to the host. Check that it is switched on and on a network this machine can reach."
        }
        Diagnosis::NameNotResolved => {
            "The host name does not resolve. Check its spelling and the DNS server of this machine."
        }
        Diagnosis::Tls => {
            "The TLS handshake failed. The service may speak plain http, or its certificate is not trusted."
        }
        Diagnosis::NotHttp => {
            "Something answered, but not in HTTP. Check the port, or probe it over tcp instead."
        }
        Diagnosis::HttpStatus => {
            "The service answered with an error status. Check its own logs, or point probe.path at a page that answers 200."
        }
        Diagnosis::IcmpNotPermitted => {
            "This process may not send ICMP echo requests. On Linux allow its group in net.ipv4.ping_group_range, or probe over tcp."
        }
        Diagnosis::Other | Diagnosis::Unknown => "See the error text.",
    }
}
