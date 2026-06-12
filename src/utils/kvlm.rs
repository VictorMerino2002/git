use indexmap::IndexMap;

pub type Kvlm = IndexMap<String, Vec<String>>;

pub fn kvlm_parse(raw: &str) -> Kvlm {
    let mut map = Kvlm::new();

    let (header, message) = match raw.find("\n\n") {
        Some(i) => (&raw[..i], &raw[i + 2..]),
        None => (raw, ""),
    };

    map.insert("message".into(), vec![message.into()]);

    let header = header.replace("\n ", " ");
    for line in header.lines() {
        if let Some(sep) = line.find(' ') {
            map.entry(line[..sep].into())
                .or_default()
                .push(line[sep + 1..].into());
        }
    }

    map
}

pub fn kvlm_serialize(map: &Kvlm) -> String {
    let mut ret = String::new();

    for (key, values) in map.iter() {
        if key == "message" {
            continue;
        }
        for value in values {
            ret.push_str(key);
            ret.push(' ');
            ret.push_str(&value.replace('\n', "\n "));
            ret.push('\n');
        }
    }

    ret.push('\n');
    ret.push_str(&map["message"][0]);

    ret
}
