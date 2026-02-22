#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn map_new_entry() {
        let mut entries = Entries::default();
        assert_eq!(MapResult::NewEntry, entries.map(
            "s/a".to_string(), "d/a".to_string(), false, false
        ));
        assert_eq!(MapResult::NewEntry, entries.map(
            "s/b".to_string(), "d/b".to_string(), false, false
        ));
        assert_eq!(MapResult::NewEntry, entries.map(
            "s/c".to_string(), "d/c".to_string(), false, false
        ));
        assert_eq!(MapResult::Noop, entries.map(
            "s/c".to_string(), "d/c".to_string(), false, false
        ));

        assert_eq!(entries.entries[0], Entry {
            src: "s/a".to_string(),
            dst: "d/a".to_string(),
            version: (0, 1, 0),
            enabled: true,
        });
        assert_eq!(entries.entries[1], Entry {
            src: "s/b".to_string(),
            dst: "d/b".to_string(),
            version: (0, 1, 0),
            enabled: true,
        });
        assert_eq!(entries.entries[2], Entry {
            src: "s/c".to_string(),
            dst: "d/c".to_string(),
            version: (0, 1, 0),
            enabled: true,
        });
    }

    #[test]
    fn map_new_dst() {
        let mut entries = Entries::default();
        assert_eq!(MapResult::NewEntry, entries.map(
            "s/a".to_string(), "d/a".to_string(), false, false
        ));
        assert_eq!(MapResult::NewEntry, entries.map(
            "s/b".to_string(), "d/b".to_string(), false, false
        ));
        assert_eq!(MapResult::NewDstBlocked, entries.map(
            "s/a".to_string(), "d/d".to_string(), false, false
        ));
        assert_eq!(MapResult::NewDst, entries.map(
            "s/a".to_string(), "d/c".to_string(), true, false
        ));

        assert_eq!(entries.entries[0], Entry {
            src: "s/a".to_string(),
            dst: "d/c".to_string(),
            version: (0, 1, 0),
            enabled: true,
        });
        assert_eq!(entries.entries[1], Entry {
            src: "s/b".to_string(),
            dst: "d/b".to_string(),
            version: (0, 1, 0),
            enabled: true,
        });
    }

    #[test]
    fn map_new_src() {
        let mut entries = Entries::default();
        assert_eq!(MapResult::NewEntry, entries.map(
            "s/a".to_string(), "d/a".to_string(), false, false
        ));
        assert_eq!(MapResult::NewEntry, entries.map(
            "s/b".to_string(), "d/b".to_string(), false, false
        ));
        assert_eq!(MapResult::NewSrcBlocked, entries.map(
            "s/d".to_string(), "d/a".to_string(), false, false
        ));
        assert_eq!(MapResult::NewSrc, entries.map(
            "s/c".to_string(), "d/a".to_string(), true, false
        ));

        assert_eq!(entries.entries[0], Entry {
            src: "s/c".to_string(),
            dst: "d/a".to_string(),
            version: (0, 1, 0),
            enabled: true,
        });
        assert_eq!(entries.entries[1], Entry {
            src: "s/b".to_string(),
            dst: "d/b".to_string(),
            version: (0, 1, 0),
            enabled: true,
        });
    }
}
