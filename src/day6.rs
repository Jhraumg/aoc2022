fn detect_marker_end_pos(signal: &[u8], marker_len: usize) -> usize {
    let len = signal.len();
    if len < marker_len {
        panic!("cannot detect a {marker_len} header in a {len} signal !");
    }
    'main: for i in 0..(len - marker_len) {
        let chars = &signal[i..i + marker_len];
        for j in 0..(marker_len - 1) {
            let r = chars[j];
            for c in &chars[j + 1..marker_len] {
                if r == *c {
                    continue 'main;
                }
            }
        }
        return i + marker_len;
    }
    0
}

fn detect_packet_marker(signal: &str) -> usize {
    detect_marker_end_pos(signal.as_bytes(), 4)
}
fn detect_message_marker(signal: &str) -> usize {
    detect_marker_end_pos(signal.as_bytes(), 14)
}

pub fn detect_signal() {
    let signal = include_str!("../resources/day6_signal.txt");

    let packet_pos = detect_packet_marker(signal);
    println!("packet detected at {packet_pos}");

    let message_pos = detect_message_marker(signal);
    println!("message detected at {message_pos}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aoc_examples_works() {
        let test_cases = vec![
            ("mjqjpqmgbljsphdztnvjfqwrcgsmlb", 7, 19),
            ("bvwbjplbgvbhsrlpgdmjqwftvncz", 5, 23),
            ("nppdvjthqldpwncqszvftbrmjlhg", 6, 23),
            ("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", 10, 29),
            ("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", 11, 26),
        ];
        for (signal, pos_header, pos_message) in test_cases {
            assert_eq!(pos_header, detect_packet_marker(signal));
            assert_eq!(pos_message, detect_message_marker(signal));
        }
    }
}
