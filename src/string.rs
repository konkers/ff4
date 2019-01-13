use std::collections::HashMap;
use std::error::Error;

lazy_static! {
    static ref SPECIAL_CHARS: HashMap<u8, &'static str> = {
        let mut map = HashMap::new();

        map.insert(0x1c, "<1c>");
        map.insert(0x25, "<mute>");
        map.insert(0x3a, "<whip>");

        map.insert(0x76, "<flat m>");
        map.insert(0x77, "<flat h>");
        map.insert(0x78, "<flat p>");

        map.insert(0x8a, " t");
        map.insert(0x8b, "th");
        map.insert(0x8c, "he");
        map.insert(0x8d, "t ");
        map.insert(0x8e, "ou");

        map.insert(0x90, " a");
        map.insert(0x91, "s ");
        map.insert(0x92, "er");
        map.insert(0x93, "in");
        map.insert(0x94, "re");
        map.insert(0x95, "d ");
        map.insert(0x96, "an");
        map.insert(0x97, " o");
        map.insert(0x98, "on");
        map.insert(0x99, "st");
        map.insert(0x9a, " w");
        map.insert(0x9b, "o ");
        map.insert(0x9c, " m");
        map.insert(0x9d, "ha");
        map.insert(0x9e, "to");
        map.insert(0x9f, "is");

        map.insert(0xa0, "yo");
        map.insert(0xa1, " y");
        map.insert(0xa2, " i");
        map.insert(0xa3, "al");
        map.insert(0xa4, "ar");
        map.insert(0xa5, " h");
        map.insert(0xa6, "r ");
        map.insert(0xa7, " s");
        map.insert(0xa8, "at");
        map.insert(0xa9, "n ");
        map.insert(0xaa, " c");
        map.insert(0xab, "ng");
        map.insert(0xac, "ce");
        map.insert(0xad, "ll");
        map.insert(0xae, "y ");
        map.insert(0xaf, "nd");

        map.insert(0xb0, "en");
        map.insert(0xb1, "ed");
        map.insert(0xb2, "hi");
        map.insert(0xb3, "or");
        map.insert(0xb4, ", ");
        map.insert(0xb5, "I ");
        map.insert(0xb6, "u ");
        map.insert(0xb7, "me");
        map.insert(0xb8, "ta");
        map.insert(0xb9, " b");
        map.insert(0xba, " I");
        map.insert(0xbb, "te");
        map.insert(0xbc, "of");
        map.insert(0xbd, "ea");
        map.insert(0xbe, "ur");
        map.insert(0xbf, "l ");

        map.insert(0xc0, "'");
        map.insert(0xc1, ".");
        map.insert(0xc2, "-");
        map.insert(0xc3, "…");
        map.insert(0xc4, "!");
        map.insert(0xc5, "?");
        map.insert(0xc6, "%");
        map.insert(0xc7, "/");
        map.insert(0xc8, ":");
        map.insert(0xc9, ",");

        map.insert(0xca, " f");
        map.insert(0xcb, " d");
        map.insert(0xcc, "ow");
        map.insert(0xcd, "se");
        map.insert(0xce, "  ");
        map.insert(0xcf, "it");

        map.insert(0xd0, "et");
        map.insert(0xd1, "le");
        map.insert(0xd2, "f ");
        map.insert(0xd3, " g");
        map.insert(0xd4, "es");
        map.insert(0xd5, "ro");
        map.insert(0xd6, "ne");
        map.insert(0xd7, "ry");
        map.insert(0xd8, " l");
        map.insert(0xd9, "us");
        map.insert(0xda, "no");
        map.insert(0xdb, "ut");
        map.insert(0xdc, "ca");
        map.insert(0xdd, "as");
        map.insert(0xde, "Th");
        map.insert(0xdf, "ai");

        map.insert(0xe0, "ot");
        map.insert(0xe1, "be");
        map.insert(0xe2, "el");
        map.insert(0xe3, "om");
        map.insert(0xe4, "'s");
        map.insert(0xe5, "il");
        map.insert(0xe6, "de");
        map.insert(0xe7, "gh");
        map.insert(0xe8, "ay");
        map.insert(0xe9, "nt");
        map.insert(0xea, "Wh");
        map.insert(0xeb, "Yo");
        map.insert(0xec, "wa");
        map.insert(0xed, "oo");
        map.insert(0xee, "We");
        map.insert(0xef, "g ");

        map.insert(0xf0, "ge");
        map.insert(0xf1, " n");
        map.insert(0xf2, "ee");
        map.insert(0xf3, "wi");
        map.insert(0xf4, " M");
        map.insert(0xf5, "ke");
        map.insert(0xf6, "we");
        map.insert(0xf7, " p");
        map.insert(0xf8, "ig");
        map.insert(0xf9, "ys");
        map.insert(0xfa, " B");
        map.insert(0xfb, "am");
        map.insert(0xfc, "ld");
        map.insert(0xfd, " W");
        map.insert(0xfe, "la");
        map.insert(0xff, " ");

        map
    };
}

pub fn decode(data: &[u8]) -> Result<String, Box<Error>> {
    let mut s = String::from("");

    for &b in data {
        if 0x42 <= b && b <= 0x5b {
            s.push((b - 0x42 + ('A' as u8)).into());
        } else if 0x5c <= b && b <= 0x75 {
            s.push((b - 0x5c + ('a' as u8)).into());
        } else if 0x80 <= b && b <= 0x89 {
            s.push((b - 0x80 + ('0' as u8)).into());
        } else if SPECIAL_CHARS.contains_key(&b) {
            s.push_str(SPECIAL_CHARS[&b]);
        } else {
            bail!("unknown char {:x}", b);
        }
    }

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_test() {
        assert_eq!(
            "Naga    ",
            decode(&[0x4f, 0x5c, 0x62, 0x5c, 0xff, 0xff, 0xff, 0xff]).unwrap()
        );
    }

}
