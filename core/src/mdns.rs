// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! The multicast DNS responder's messages (RFC 6762): the clock is
//! `<hostname>.local` on a network whose router has no name for it.
//!
//! Only the host's own records: an A with its address, and an NSEC
//! that says A is the one type the name has, so a lookup of AAAA is
//! told "none" at once instead of waiting out its time. No services,
//! no IPv6, no reverse lookup.
//!
//! The name is not probed for and not defended, on purpose
//! (`agents/behaviour.md`, "The name on the network"): two clocks of
//! one name both answer, and the cure is a hostname a clock.
//!
//! `respond` and `announce` are pure; the socket is the firmware's
//! `mdns` task.

/// The port, for the query and the answer alike.
pub const PORT: u16 = 5353;

/// The group every responder listens on.
pub const GROUP: [u8; 4] = [224, 0, 0, 251];

/// How long an asker may keep the address: what the RFC gives a
/// host's records.
pub const TTL_SECS: u32 = 120;

/// The same for an asker that is no mDNS resolver, which would not
/// see the record change: the RFC's cap.
pub const LEGACY_TTL_SECS: u32 = 10;

/// The longest message `respond` or `announce` writes: the header, a
/// question, the name in full once, and the two records.
pub const RESPONSE_MAX: usize = HEADER + NAME_MAX + 4 + (2 + 10 + 4) + (2 + 10 + NSEC_RDATA.len());

const HEADER: usize = 12;
/// A hostname's label, `local` and the root, with their length bytes.
const NAME_MAX: usize = 1 + crate::settings::HOSTNAME_MAX + 1 + LOCAL.len() + 1;
const LOCAL: &[u8] = b"local";

const TYPE_A: u16 = 1;
const TYPE_NSEC: u16 = 47;
const TYPE_ANY: u16 = 255;
const CLASS_IN: u16 = 1;
const CLASS_ANY: u16 = 255;
/// In a question's class: answer me alone. In a record's: this is
/// all there is, forget what you had.
const TOP_BIT: u16 = 0x8000;

/// A pointer to the name, which is always the first thing after the
/// header.
const NAME_POINTER: [u8; 2] = [0xC0, HEADER as u8];

/// The NSEC's data: the next name (the name itself), then window 0
/// with one byte of types, of which bit 1 is A.
const NSEC_RDATA: [u8; 5] = [NAME_POINTER[0], NAME_POINTER[1], 0, 1, 0x40];

/// The longest name on the wire, length bytes and all (RFC 1035).
const WIRE_NAME_MAX: usize = 255;

/// The least time between two answers to the group with the same
/// record (RFC 6762, section 6).
pub const GROUP_GAP_MS: u64 = 1000;

/// Where an answer goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum To {
    /// To the group, port 5353.
    Group,
    /// To the address and port the query came from.
    Asker,
}

/// An answer `respond` wrote.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reply {
    /// Its length in the buffer.
    pub len: usize,
    pub to: To,
    /// It has the address; without, it is the NSEC alone.
    pub address: bool,
}

/// Holds answers to the group to one of a kind in `GROUP_GAP_MS`, so
/// a flood of queries is not a flood of multicast. An answer to the
/// asker alone is not counted: it is one datagram for one, to an
/// address on the subnet.
#[derive(Default)]
pub struct Limit {
    /// When the address last went to the group, and the NSEC alone.
    last: [Option<u64>; 2],
}

impl Limit {
    pub const fn new() -> Self {
        Limit { last: [None; 2] }
    }

    /// Whether `reply` may be sent at `now_ms`, which is then its
    /// record's time if it goes to the group.
    pub fn allow(&mut self, reply: &Reply, now_ms: u64) -> bool {
        if reply.to != To::Group {
            return true;
        }
        let last = &mut self.last[reply.address as usize];
        if last.is_some_and(|t| now_ms.saturating_sub(t) < GROUP_GAP_MS) {
            return false;
        }
        *last = Some(now_ms);
        true
    }

    /// The address was announced at `now_ms`.
    pub fn announced(&mut self, now_ms: u64) {
        self.last[1] = Some(now_ms);
    }
}

/// What the questions for our name came to.
#[derive(Default)]
struct Asked {
    /// The address was asked for.
    a: bool,
    /// Another type was: answered with the NSEC.
    other: bool,
    /// Every question for the name had the unicast bit.
    unicast: bool,
    /// The type and class of the first one, for a legacy answer.
    first: Option<(u16, u16)>,
}

/// Answer a query, or `None` when there is nothing to say: not a
/// query, not for `<hostname>.local`, malformed, or the asker already
/// has the address. `from_port` is the query's source port; one that
/// is not 5353 is a plain resolver's and gets a plain answer.
pub fn respond(query: &[u8], hostname: &str, ip: [u8; 4], from_port: u16, out: &mut [u8]) -> Option<Reply> {
    if query.len() < HEADER || hostname.is_empty() {
        return None;
    }
    let flags = u16::from_be_bytes([query[2], query[3]]);
    // QR must be 0 (a query) and the opcode 0 (standard).
    if flags & 0xF800 != 0 {
        return None;
    }
    let questions = u16::from_be_bytes([query[4], query[5]]);
    let answers = u16::from_be_bytes([query[6], query[7]]);

    let mut asked = Asked { unicast: true, ..Asked::default() };
    let mut i = HEADER;
    for _ in 0..questions {
        let ours = is_name(query, i, hostname)?;
        i = skip_name(query, i)?;
        let qtype = be16(query, i)?;
        let qclass = be16(query, i + 2)?;
        i += 4;
        let class = qclass & !TOP_BIT;
        if !ours || (class != CLASS_IN && class != CLASS_ANY) {
            continue;
        }
        if qtype == TYPE_A || qtype == TYPE_ANY {
            asked.a = true;
        } else {
            asked.other = true;
        }
        asked.unicast &= qclass & TOP_BIT != 0;
        asked.first.get_or_insert((qtype, class));
    }
    let first = asked.first?;

    // The known answers: our address with at least half its life left
    // is one the asker need not be told again.
    for _ in 0..answers {
        let ours = is_name(query, i, hostname)?;
        i = skip_name(query, i)?;
        let rtype = be16(query, i)?;
        let class = be16(query, i + 2)? & !TOP_BIT;
        let ttl = u32::from_be_bytes([*query.get(i + 4)?, *query.get(i + 5)?, *query.get(i + 6)?, *query.get(i + 7)?]);
        let len = be16(query, i + 8)? as usize;
        let data = query.get(i + 10..i + 10 + len)?;
        i += 10 + len;
        if ours && rtype == TYPE_A && class == CLASS_IN && data == ip && ttl >= TTL_SECS / 2 {
            asked.a = false;
        }
    }
    if !asked.a && !asked.other {
        return None;
    }

    let legacy = from_port != PORT;
    let n = write(
        out,
        hostname,
        ip,
        &Shape {
            id: if legacy { [query[0], query[1]] } else { [0, 0] },
            question: legacy.then_some(first),
            address: asked.a,
            ttl: if legacy { LEGACY_TTL_SECS } else { TTL_SECS },
            flush: !legacy,
        },
    )?;
    Some(Reply { len: n, to: if legacy || asked.unicast { To::Asker } else { To::Group }, address: asked.a })
}

/// The unsolicited answer for the group, sent when the address is
/// new: what a cache held for the name is replaced.
pub fn announce(hostname: &str, ip: [u8; 4], out: &mut [u8]) -> Option<usize> {
    write(out, hostname, ip, &Shape { id: [0, 0], question: None, address: true, ttl: TTL_SECS, flush: true })
}

/// What `write` puts in a message.
struct Shape {
    id: [u8; 2],
    /// The question to repeat, its type and class: a plain resolver
    /// wants it back.
    question: Option<(u16, u16)>,
    /// The A as the answer and the NSEC beside it; without, the NSEC
    /// is the answer.
    address: bool,
    ttl: u32,
    /// The cache-flush bit on the records.
    flush: bool,
}

fn write(out: &mut [u8], hostname: &str, ip: [u8; 4], shape: &Shape) -> Option<usize> {
    let host = hostname.as_bytes();
    if host.is_empty() || host.len() > crate::settings::HOSTNAME_MAX {
        return None;
    }
    let mut w = Writer { out, n: 0 };
    w.put(&shape.id)?;
    // A response, authoritative, no error.
    w.put(&0x8400u16.to_be_bytes())?;
    w.put(&(shape.question.is_some() as u16).to_be_bytes())?;
    // One answer; with the address, the NSEC is an additional record.
    w.put(&1u16.to_be_bytes())?;
    w.put(&0u16.to_be_bytes())?;
    w.put(&(shape.address as u16).to_be_bytes())?;

    // The name once, in full, at `NAME_POINTER`: in the question if
    // there is one, else in the first record.
    w.put(&[host.len() as u8])?;
    w.put(host)?;
    w.put(&[LOCAL.len() as u8])?;
    w.put(LOCAL)?;
    w.put(&[0])?;
    if let Some((qtype, qclass)) = shape.question {
        w.put(&qtype.to_be_bytes())?;
        w.put(&qclass.to_be_bytes())?;
        w.put(&NAME_POINTER)?;
    }
    let class = CLASS_IN | if shape.flush { TOP_BIT } else { 0 };
    if shape.address {
        w.record(TYPE_A, class, shape.ttl, &ip)?;
        w.put(&NAME_POINTER)?;
    }
    w.record(TYPE_NSEC, class, shape.ttl, &NSEC_RDATA)?;
    Some(w.n)
}

struct Writer<'a> {
    out: &'a mut [u8],
    n: usize,
}

impl Writer<'_> {
    fn put(&mut self, bytes: &[u8]) -> Option<()> {
        self.out.get_mut(self.n..self.n + bytes.len())?.copy_from_slice(bytes);
        self.n += bytes.len();
        Some(())
    }

    /// A record after its name.
    fn record(&mut self, rtype: u16, class: u16, ttl: u32, data: &[u8]) -> Option<()> {
        self.put(&rtype.to_be_bytes())?;
        self.put(&class.to_be_bytes())?;
        self.put(&ttl.to_be_bytes())?;
        self.put(&(data.len() as u16).to_be_bytes())?;
        self.put(data)
    }
}

fn be16(msg: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*msg.get(at)?, *msg.get(at + 1)?]))
}

/// The offset after the name at `at`: past its root, or past its
/// first pointer.
fn skip_name(msg: &[u8], mut at: usize) -> Option<usize> {
    loop {
        let len = *msg.get(at)? as usize;
        match len {
            0 => return Some(at + 1),
            1..=63 => at += 1 + len,
            _ if len >= 0xC0 => return Some(at + 2),
            _ => return None,
        }
    }
}

/// Whether the name at `at` is `<hostname>.local`, in any case;
/// `None` for a name that runs off the message, is longer than a name
/// may be, or has a pointer that does not point back. A pointer may
/// only go to before where the name's last piece began, so every one
/// is to a lower offset than the one before and the walk must end.
fn is_name(msg: &[u8], mut at: usize, hostname: &str) -> Option<bool> {
    let wanted = [hostname.as_bytes(), LOCAL];
    let mut label = 0;
    let mut same = true;
    let mut piece = at;
    let mut wire = 1;
    loop {
        let len = *msg.get(at)? as usize;
        match len {
            0 => return Some(same && label == wanted.len()),
            1..=63 => {
                let text = msg.get(at + 1..at + 1 + len)?;
                wire += 1 + len;
                if wire > WIRE_NAME_MAX {
                    return None;
                }
                same = same && wanted.get(label).is_some_and(|w| w.eq_ignore_ascii_case(text));
                label += 1;
                at += 1 + len;
            }
            _ if len >= 0xC0 => {
                let to = (len & 0x3F) << 8 | *msg.get(at + 1)? as usize;
                if to >= piece {
                    return None;
                }
                at = to;
                piece = to;
            }
            _ => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IP: [u8; 4] = [10, 1, 1, 30];
    const HOST: &str = "white-clock";

    fn name(labels: &[&str]) -> Vec<u8> {
        let mut v = Vec::new();
        for l in labels {
            v.push(l.len() as u8);
            v.extend_from_slice(l.as_bytes());
        }
        v.push(0);
        v
    }

    fn header(id: u16, flags: u16, questions: u16, answers: u16) -> Vec<u8> {
        let mut v = Vec::new();
        for w in [id, flags, questions, answers, 0, 0] {
            v.extend_from_slice(&w.to_be_bytes());
        }
        v
    }

    fn question(labels: &[&str], qtype: u16, qclass: u16) -> Vec<u8> {
        let mut v = name(labels);
        v.extend_from_slice(&qtype.to_be_bytes());
        v.extend_from_slice(&qclass.to_be_bytes());
        v
    }

    fn query(labels: &[&str], qtype: u16, qclass: u16) -> Vec<u8> {
        let mut v = header(0, 0, 1, 0);
        v.extend(question(labels, qtype, qclass));
        v
    }

    fn respond_to(q: &[u8], port: u16) -> Option<(Vec<u8>, To)> {
        let mut out = [0u8; RESPONSE_MAX];
        respond(q, HOST, IP, port, &mut out).map(|r| (out[..r.len].to_vec(), r.to))
    }

    /// A record of a response: its name read, then the fields.
    struct Record {
        rtype: u16,
        class: u16,
        ttl: u32,
        data: Vec<u8>,
    }

    /// The records of a response after its questions, each checked to
    /// be named `<hostname>.local`.
    fn records(msg: &[u8]) -> Vec<Record> {
        let questions = be16(msg, 4).unwrap();
        let count = be16(msg, 6).unwrap() + be16(msg, 8).unwrap() + be16(msg, 10).unwrap();
        let mut i = HEADER;
        for _ in 0..questions {
            assert_eq!(is_name(msg, i, HOST), Some(true));
            i = skip_name(msg, i).unwrap() + 4;
        }
        let mut v = Vec::new();
        for _ in 0..count {
            assert_eq!(is_name(msg, i, HOST), Some(true), "a record's name");
            i = skip_name(msg, i).unwrap();
            let len = be16(msg, i + 8).unwrap() as usize;
            v.push(Record {
                rtype: be16(msg, i).unwrap(),
                class: be16(msg, i + 2).unwrap(),
                ttl: u32::from_be_bytes(msg[i + 4..i + 8].try_into().unwrap()),
                data: msg[i + 10..i + 10 + len].to_vec(),
            });
            i += 10 + len;
        }
        assert_eq!(i, msg.len(), "nothing after the last record");
        v
    }

    #[test]
    fn an_a_question_is_answered_to_the_group() {
        let (msg, to) = respond_to(&query(&[HOST, "local"], TYPE_A, CLASS_IN), PORT).unwrap();
        assert_eq!(to, To::Group);
        assert_eq!(&msg[..4], &[0, 0, 0x84, 0], "no id; a response, authoritative");
        assert_eq!(&msg[4..12], &[0, 0, 0, 1, 0, 0, 0, 1], "no question, the A, the NSEC beside it");
        let r = records(&msg);
        assert_eq!((r[0].rtype, r[0].class, r[0].ttl), (TYPE_A, CLASS_IN | TOP_BIT, TTL_SECS));
        assert_eq!(r[0].data, IP);
        assert_eq!((r[1].rtype, r[1].class, r[1].ttl), (TYPE_NSEC, CLASS_IN | TOP_BIT, TTL_SECS));
        assert_eq!(r[1].data, NSEC_RDATA);
    }

    #[test]
    fn the_name_is_matched_in_any_case_and_whole() {
        assert!(respond_to(&query(&["White-CLOCK", "LOCAL"], TYPE_A, CLASS_IN), PORT).is_some());
        for labels in [
            &["white-clock", "lan"][..],
            &["white-clock"],
            &["white-clock", "local", "local"],
            &["white-cloc", "local"],
            &["white-clock2", "local"],
            &["other", "local"],
            &["_http", "_tcp", "local"],
            &["local"],
            &[],
        ] {
            assert!(respond_to(&query(labels, TYPE_A, CLASS_IN), PORT).is_none(), "{labels:?}");
        }
    }

    #[test]
    fn another_type_gets_the_nsec_alone() {
        const TYPE_AAAA: u16 = 28;
        let (msg, to) = respond_to(&query(&[HOST, "local"], TYPE_AAAA, CLASS_IN), PORT).unwrap();
        assert_eq!(to, To::Group);
        assert_eq!(&msg[4..12], &[0, 0, 0, 1, 0, 0, 0, 0]);
        let r = records(&msg);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].rtype, TYPE_NSEC);
    }

    #[test]
    fn any_is_answered_with_the_address() {
        let (msg, _) = respond_to(&query(&[HOST, "local"], TYPE_ANY, CLASS_ANY), PORT).unwrap();
        assert_eq!(records(&msg)[0].rtype, TYPE_A);
    }

    #[test]
    fn another_class_is_not_answered() {
        assert!(respond_to(&query(&[HOST, "local"], TYPE_A, 3), PORT).is_none());
    }

    #[test]
    fn the_unicast_bit_is_answered_to_the_asker() {
        let (msg, to) = respond_to(&query(&[HOST, "local"], TYPE_A, CLASS_IN | TOP_BIT), PORT).unwrap();
        assert_eq!(to, To::Asker);
        assert_eq!(records(&msg)[0].class, CLASS_IN | TOP_BIT, "still an mDNS answer");
        // One question without the bit and the answer is everybody's.
        let mut q = header(0, 0, 2, 0);
        q.extend(question(&[HOST, "local"], TYPE_A, CLASS_IN | TOP_BIT));
        q.extend(question(&[HOST, "local"], 28, CLASS_IN));
        assert_eq!(respond_to(&q, PORT).unwrap().1, To::Group);
    }

    #[test]
    fn a_plain_resolver_gets_a_plain_answer() {
        let mut q = query(&[HOST, "local"], TYPE_A, CLASS_IN);
        q[..2].copy_from_slice(&[0x12, 0x34]);
        let (msg, to) = respond_to(&q, 40000).unwrap();
        assert_eq!(to, To::Asker);
        assert_eq!(&msg[..2], &[0x12, 0x34], "its id");
        assert_eq!(&msg[4..12], &[0, 1, 0, 1, 0, 0, 0, 1], "its question back");
        let question_end = HEADER + name(&[HOST, "local"]).len() + 4;
        assert_eq!(&msg[question_end - 4..question_end], &[0, 1, 0, 1]);
        let r = records(&msg);
        assert_eq!((r[0].rtype, r[0].class, r[0].ttl), (TYPE_A, CLASS_IN, LEGACY_TTL_SECS));
        assert_eq!(r[1].class, CLASS_IN, "no cache-flush bit for a cache that does not know it");
    }

    #[test]
    fn a_second_question_and_a_compressed_name_are_read() {
        // The first question is somebody else's; ours follows, its
        // `local` a pointer into the first.
        let mut q = header(0, 0, 2, 0);
        q.extend(question(&["printer", "local"], TYPE_A, CLASS_IN));
        let local_at = HEADER + 1 + "printer".len();
        q.push(HOST.len() as u8);
        q.extend_from_slice(HOST.as_bytes());
        q.extend_from_slice(&[0xC0, local_at as u8]);
        q.extend_from_slice(&[0, 1, 0, 1]);
        let (msg, _) = respond_to(&q, PORT).unwrap();
        assert_eq!(records(&msg)[0].data, IP);
    }

    fn with_known_answer(ip: [u8; 4], ttl: u32) -> Vec<u8> {
        let mut q = header(0, 0, 1, 1);
        q.extend(question(&[HOST, "local"], TYPE_A, CLASS_IN));
        q.extend_from_slice(&NAME_POINTER);
        q.extend_from_slice(&[0, 1, 0, 1]);
        q.extend_from_slice(&ttl.to_be_bytes());
        q.extend_from_slice(&[0, 4]);
        q.extend_from_slice(&ip);
        q
    }

    #[test]
    fn a_known_answer_is_not_repeated() {
        assert!(respond_to(&with_known_answer(IP, TTL_SECS), PORT).is_none());
        assert!(respond_to(&with_known_answer(IP, TTL_SECS / 2), PORT).is_none());
        // An old one or a wrong one is.
        assert!(respond_to(&with_known_answer(IP, TTL_SECS / 2 - 1), PORT).is_some());
        assert!(respond_to(&with_known_answer([10, 1, 1, 31], TTL_SECS), PORT).is_some());
    }

    #[test]
    fn what_is_no_query_is_passed_by() {
        let q = query(&[HOST, "local"], TYPE_A, CLASS_IN);
        // A response, as every other responder's answers are.
        let mut r = q.clone();
        r[2] = 0x84;
        assert!(respond_to(&r, PORT).is_none());
        // Another opcode.
        let mut r = q.clone();
        r[2] = 0x28;
        assert!(respond_to(&r, PORT).is_none());
        // No question.
        assert!(respond_to(&header(0, 0, 0, 0), PORT).is_none());
        // Cut anywhere.
        for n in 0..q.len() {
            assert!(respond_to(&q[..n], PORT).is_none(), "cut at {n}");
        }
        // A known answer that runs off the end.
        let k = with_known_answer(IP, TTL_SECS);
        for n in q.len()..k.len() {
            assert!(respond_to(&k[..n], PORT).is_none(), "cut at {n}");
        }
    }

    #[test]
    fn a_name_that_points_at_itself_ends() {
        let mut q = header(0, 0, 1, 0);
        q.extend_from_slice(&[0xC0, HEADER as u8, 0, 1, 0, 1]);
        assert!(respond_to(&q, PORT).is_none());
        // A length of the reserved kinds.
        let mut q = header(0, 0, 1, 0);
        q.extend_from_slice(&[0x80, 0, 0, 1, 0, 1]);
        assert!(respond_to(&q, PORT).is_none());
    }

    #[test]
    fn a_pointer_goes_back_only() {
        // Forward, to a name that is ours: not followed.
        let mut q = header(0, 0, 1, 0);
        q.extend_from_slice(&[0xC0, (HEADER + 6) as u8, 0, 1, 0, 1]);
        q.extend(name(&[HOST, "local"]));
        assert!(respond_to(&q, PORT).is_none());
        // Two that point at each other.
        let mut q = header(0, 0, 2, 0);
        q.extend_from_slice(&[0xC0, (HEADER + 6) as u8, 0, 1, 0, 1]);
        q.extend_from_slice(&[0xC0, HEADER as u8, 0, 1, 0, 1]);
        assert!(respond_to(&q, PORT).is_none());
        // A label, then back to itself: a name with no end.
        let mut q = header(0, 0, 1, 0);
        q.extend_from_slice(&[1, b'a', 0xC0, HEADER as u8, 0, 1, 0, 1]);
        assert!(respond_to(&q, PORT).is_none());
    }

    #[test]
    fn a_name_longer_than_a_name_is_refused() {
        let label = "a".repeat(63);
        let mut q = header(0, 0, 2, 0);
        q.extend(question(&[&label, &label, &label, &label], TYPE_A, CLASS_IN));
        q.extend(question(&[HOST, "local"], TYPE_A, CLASS_IN));
        assert!(respond_to(&q, PORT).is_none(), "257 bytes of name");
        let mut q = header(0, 0, 2, 0);
        q.extend(question(&[&label, &label, &label, &label[..61]], TYPE_A, CLASS_IN));
        q.extend(question(&[HOST, "local"], TYPE_A, CLASS_IN));
        assert!(respond_to(&q, PORT).is_some(), "255 is a name, and the next question ours");
    }

    /// No input may panic or be answered past the buffer: random
    /// bytes, and real queries with bytes changed, the same every run.
    #[test]
    fn no_packet_panics() {
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut good = with_known_answer(IP, TTL_SECS / 4);
        good[4..6].copy_from_slice(&2u16.to_be_bytes());
        good.splice(HEADER..HEADER, question(&["printer", "local"], 28, CLASS_IN));
        let mut out = [0u8; RESPONSE_MAX];
        let mut answered = 0;
        for round in 0..200_000 {
            let mut q = if round % 2 == 0 {
                let mut q = vec![0u8; (next() % 96) as usize];
                q.iter_mut().for_each(|b| *b = next() as u8);
                // A query's flags, so the walk is reached.
                if q.len() > 3 && round % 4 == 0 {
                    q[2] &= 0x07;
                }
                q
            } else {
                good.clone()
            };
            if round % 2 == 1 {
                for _ in 0..1 + next() % 4 {
                    let at = (next() % q.len() as u64) as usize;
                    q[at] = match next() % 4 {
                        0 => 0xC0,
                        1 => 0,
                        _ => next() as u8,
                    };
                }
                q.truncate(q.len() - (next() % 3) as usize);
            }
            let port = if next() % 2 == 0 { PORT } else { 40000 };
            if let Some(r) = respond(&q, HOST, IP, port, &mut out) {
                assert!(r.len <= RESPONSE_MAX);
                assert_eq!(&out[2..4], &[0x84, 0]);
                answered += 1;
            }
        }
        assert!(answered > 1000, "the changed queries are still answered often: {answered}");
    }

    #[test]
    fn the_group_hears_a_record_once_a_second() {
        let a = Reply { len: 0, to: To::Group, address: true };
        let nsec = Reply { address: false, ..a };
        let mut limit = Limit::new();
        assert!(limit.allow(&nsec, 5000));
        assert!(limit.allow(&a, 5000), "the address is another record: AAAA is asked first, then A");
        assert!(!limit.allow(&a, 5999));
        assert!(!limit.allow(&nsec, 5001));
        assert!(limit.allow(&a, 6000));
        // An answer to the asker is neither held nor counted.
        let one = Reply { to: To::Asker, ..a };
        assert!(limit.allow(&one, 6001));
        assert!(limit.allow(&one, 6001));
        // An announcement counts as the address.
        limit.announced(9000);
        assert!(!limit.allow(&a, 9500));
        assert!(limit.allow(&a, 10000));
    }

    #[test]
    fn the_announcement_is_the_answer_unasked() {
        let mut out = [0u8; RESPONSE_MAX];
        let n = announce(HOST, IP, &mut out).unwrap();
        let (msg, _) = respond_to(&query(&[HOST, "local"], TYPE_A, CLASS_IN), PORT).unwrap();
        assert_eq!(&out[..n], &msg[..]);
    }

    #[test]
    fn the_longest_message_fits() {
        let host = "h".repeat(crate::settings::HOSTNAME_MAX);
        let mut q = query(&[&host, "local"], TYPE_A, CLASS_IN);
        q[..2].copy_from_slice(&[1, 2]);
        let mut out = [0u8; RESPONSE_MAX];
        let n = respond(&q, &host, IP, 40000, &mut out).unwrap().len;
        assert_eq!(n, RESPONSE_MAX, "a plain resolver's answer is the longest");
        // A buffer too short is no answer, not a panic.
        assert!(respond(&q, &host, IP, 40000, &mut out[..n - 1]).is_none());
        assert!(announce(&host, IP, &mut out[..20]).is_none());
        assert!(announce("", IP, &mut out).is_none());
    }
}
