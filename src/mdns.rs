// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! The multicast DNS responder's socket: the station answers for
//! `<hostname>.local`. The messages are `core`'s `mdns`, pure and
//! under test; this is the group, the socket and the loop.

use embassy_futures::select::{Either, select};
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{IpAddress, IpEndpoint, Ipv4Address, Stack};
use embassy_time::{Duration, Instant, Timer};
use log::{info, warn};
use onebitdawn_core::mdns::{GROUP, Limit, PORT, RESPONSE_MAX, To, announce, respond};

use crate::settings;

/// The largest datagram a frame of 1500 bytes carries, after the IP
/// and UDP headers. The stack drops one that does not fit; fragments
/// are not put together.
const DATAGRAM_MAX: usize = 1500 - 20 - 8;

/// What the socket holds until the loop has read it: two datagrams of
/// the largest size. The other responders' answers come into the same
/// buffer as the queries and are the long ones (a speaker's and a
/// printer's records, 1200 to 1450 bytes here); room for one alone
/// would let such an answer keep out the query behind it.
const RX_BYTES: usize = 2 * DATAGRAM_MAX;

/// How many datagrams the socket holds until the loop has read them:
/// the group's traffic comes in bursts, the other responders' answers
/// with the queries.
const RX_DATAGRAMS: usize = 8;

/// Announcements at a new address, and the wait between them (the
/// RFC's least: two, a second apart).
const ANNOUNCEMENTS: usize = 2;
const ANNOUNCE_GAP: Duration = Duration::from_secs(1);

/// A response is sent with this hop limit (RFC 6762, section 11).
const HOP_LIMIT: u8 = 255;

/// Answer for the hostname while the station has an address.
#[embassy_executor::task]
pub async fn mdns_task(stack: Stack<'static>) -> ! {
    let mut rx_meta = [PacketMetadata::EMPTY; RX_DATAGRAMS];
    let mut rx_buf = [0u8; RX_BYTES];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_buf = [0u8; RESPONSE_MAX * 2];
    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buf, &mut tx_meta, &mut tx_buf);
    if let Err(e) = socket.bind(PORT) {
        warn!("mdns: bind failed: {e:?}");
        loop {
            Timer::after_secs(3600).await;
        }
    }
    socket.set_hop_limit(Some(HOP_LIMIT));
    let group = Ipv4Address::from(GROUP);
    let everybody = IpEndpoint::new(IpAddress::Ipv4(group), PORT);
    let mut out = [0u8; RESPONSE_MAX];
    loop {
        stack.wait_config_up().await;
        // The address for as long as it is up: a new one comes by way
        // of `wait_config_down`.
        let Some(net) = stack.config_v4().map(|c| c.address) else { continue };
        let ip = net.address().octets();
        // Joining is what lets the group's frames past the stack, and
        // the report it sends what tells a snooping switch to pass them.
        if let Err(e) = stack.join_multicast_group(group) {
            warn!("mdns: join failed: {e:?}");
        }
        let hostname = settings::get().hostname;
        info!("mdns: {}.local is {}.{}.{}.{}", hostname.as_str(), ip[0], ip[1], ip[2], ip[3]);
        let mut limit = Limit::new();
        for i in 0..ANNOUNCEMENTS {
            if i > 0 {
                Timer::after(ANNOUNCE_GAP).await;
            }
            if let Some(n) = announce(hostname.as_str(), ip, &mut out)
                && let Err(e) = socket.send_to(&out[..n], everybody).await
            {
                warn!("mdns: announce failed: {e:?}");
            }
            limit.announced(Instant::now().as_millis());
        }
        loop {
            let query = socket.recv_from_with(|query, meta| {
                // The stack is borrowed in here: nothing of it may be
                // asked (`config_v4` panics). The name is the settings'.
                let IpAddress::Ipv4(from) = meta.endpoint.addr;
                // An asker on another network cannot be on the link
                // (RFC 6762, section 11).
                if !net.contains_addr(&from) {
                    return None;
                }
                let hostname = settings::get().hostname;
                let reply = respond(query, hostname.as_str(), ip, meta.endpoint.port, &mut out)?;
                Some((reply, meta.endpoint))
            });
            match select(query, stack.wait_config_down()).await {
                Either::First(None) => {}
                Either::First(Some((reply, asker))) => {
                    // A flood of queries is not answered with one of
                    // multicast.
                    if !limit.allow(&reply, Instant::now().as_millis()) {
                        continue;
                    }
                    let to = match reply.to {
                        To::Group => everybody,
                        To::Asker => asker,
                    };
                    if let Err(e) = socket.send_to(&out[..reply.len], to).await {
                        warn!("mdns: send failed: {e:?}");
                    }
                }
                Either::Second(()) => break,
            }
        }
        let _ = stack.leave_multicast_group(group);
    }
}
