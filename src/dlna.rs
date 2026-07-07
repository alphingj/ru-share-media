// DLNA/UPnP Discovery Module
use std::net::Ipv4Addr;

pub async fn start_dlna_discovery() {
  let socket = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
    Ok(s) => s,
    Err(_) => return,
  };
  let multiaddr: Ipv4Addr = "239.255.255.250".parse().unwrap();
  let interface: Ipv4Addr = "0.0.0.0".parse().unwrap();
  let _ = socket.join_multicast_v4(multiaddr, interface);
  let msg = "NOTIFY * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nNTS: ssdp:alive\r\nNT: urn:schemas-upnp-org:device:MediaServer:1\r\n";
  let _ = socket.send(msg.as_bytes()).await;
}