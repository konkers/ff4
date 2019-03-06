use nom::types::CompleteByteSlice;
use simple_error::SimpleError;
use std::error::Error;

use super::super::rom_map;

// Each monster has an attack group ID.  This indexes into the attack group
// table.  The table is a list of entries.  Each entry is Terminated by 0xff.
#[derive(Debug, PartialEq)]
pub struct Group {
    entries: Vec<GroupEntry>,
}

#[derive(Debug, PartialEq)]
pub struct GroupEntry {
    condition_set_index: u8,
    action_index: u8,
}

#[derive(Debug, PartialEq)]
pub struct ConditionSet {
    condition_indexes: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct Condition {
    op: u8,
    args: [u8; 3],
}

#[derive(Debug, PartialEq)]
pub struct Ai {
    condition_sets: Vec<ConditionSet>,
    conditions: Vec<Condition>,
    groups: Vec<Group>,
}

named!(parse_groups<CompleteByteSlice, Vec<Group>>,
     complete!(many_m_n!(0, 0x100, parse_group)));

named!(parse_group_entry<CompleteByteSlice, GroupEntry>,
    do_parse!(
        c: take!(1) >>
        a: take!(1) >>
        (GroupEntry{condition_set_index: c[0], action_index: a[0]})
));

named!(parse_group<CompleteByteSlice, Group>, do_parse!(
    e: many_till!(parse_group_entry, tag!(&[0xff][..])) >>
    (Group{entries: e.0})
));

named!(parse_condition_sets<CompleteByteSlice, Vec<ConditionSet>>,
    many_m_n!(0, 0x62, parse_condition_set));

named!(parse_condition_set<CompleteByteSlice, ConditionSet>,
    do_parse!(
        indexes: take_until!(&[0xff][..]) >>
        tag!(&[0xff][..]) >>
        (ConditionSet{condition_indexes: indexes.0.to_vec()})
    ));

named!(parse_condition<CompleteByteSlice, Condition>,
    do_parse!(
        op: take!(1) >>
        args: take!(3) >>
        (Condition{op: op[0], args: [args[0], args[1], args[2]]})
));

named!(parse_conditions<CompleteByteSlice, Vec<Condition>>,
    many1!(parse_condition)
);

pub fn parse(data: &[u8]) -> Result<Ai, Box<Error>> {
    let groups = parse_groups(CompleteByteSlice(
        &data[rom_map::ATTACK_GROUP_START..=rom_map::ATTACK_GROUP_END],
    ))
    .map_err(|e| SimpleError::new(format!("Can't parse AI: {}", e)))?
    .1;

    let condition_sets = parse_condition_sets(CompleteByteSlice(
        &data[rom_map::AI_CONDITION_SET_TABLE_START..rom_map::AI_CONDITION_SET_TABLE_END],
    ))
    .map_err(|e| SimpleError::new(format!("Can't parse condition sets: {}", e)))?
    .1;

    let conditions = parse_conditions(CompleteByteSlice(
        &data[rom_map::AI_CONDITION_TABLE_START..rom_map::AI_CONDITION_TABLE_END],
    ))
    .map_err(|e| SimpleError::new(format!("Can't parse conditions: {}", e)))?
    .1;

    Ok(Ai {
        groups: groups,
        condition_sets: condition_sets,
        conditions: conditions,
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::test_utils;
    use super::*;

    #[test]
    fn parse_group_entry_test() {
        assert_eq!(
            GroupEntry {
                condition_set_index: 0x55,
                action_index: 0x00,
            },
            parse_group_entry(CompleteByteSlice(&[0x55, 0x00]))
                .unwrap()
                .1
        );
    }

    #[test]
    fn parse_group_test() {
        assert_eq!(
            Group {
                entries: vec!(
                    GroupEntry {
                        condition_set_index: 0x55,
                        action_index: 0x00,
                    },
                    GroupEntry {
                        condition_set_index: 0x01,
                        action_index: 0x02,
                    },
                    GroupEntry {
                        condition_set_index: 0x00,
                        action_index: 0x01,
                    },
                ),
            },
            parse_group(CompleteByteSlice(&[
                0x55, 0x00, 0x01, 0x02, 0x00, 0x01, 0xff
            ]))
            .unwrap()
            .1
        );
    }

    #[test]
    fn parse_groups_test() {
        assert_eq!(
            vec!(
                Group {
                    entries: vec!(
                        GroupEntry {
                            condition_set_index: 0x55,
                            action_index: 0x00,
                        },
                        GroupEntry {
                            condition_set_index: 0x01,
                            action_index: 0x02,
                        },
                        GroupEntry {
                            condition_set_index: 0x00,
                            action_index: 0x01,
                        },
                    ),
                },
                Group {
                    entries: vec!(
                        GroupEntry {
                            condition_set_index: 0x01,
                            action_index: 0x02,
                        },
                        GroupEntry {
                            condition_set_index: 0x00,
                            action_index: 0x03,
                        },
                    ),
                },
            ),
            parse_groups(CompleteByteSlice(&[
                0x55, 0x00, 0x01, 0x02, 0x00, 0x01, 0xff, 0x01, 0x02, 0x00, 0x03, 0xff
            ]))
            .unwrap()
            .1
        );
    }

    #[test]
    fn parse_condition_test() {
        assert_eq!(
            Condition {
                op: 0x55,
                args: [0xaa, 0x00, 0xff],
            },
            parse_condition(CompleteByteSlice(&[0x55, 0xaa, 0x00, 0xff]))
                .unwrap()
                .1
        );
    }

    #[test]
    fn parse_condition_set_test() {
        assert_eq!(
            ConditionSet {
                condition_indexes: vec!(0x55),
            },
            parse_condition_set(CompleteByteSlice(&[0x55, 0xff]))
                .unwrap()
                .1
        );
        assert_eq!(
            ConditionSet {
                condition_indexes: vec!(0x55, 0xaa),
            },
            parse_condition_set(CompleteByteSlice(&[0x55, 0xaa, 0xff]))
                .unwrap()
                .1
        );
        assert_eq!(
            ConditionSet {
                condition_indexes: vec!(0x55, 0xaa, 0xcc),
            },
            parse_condition_set(CompleteByteSlice(&[0x55, 0xaa, 0xcc, 0xff]))
                .unwrap()
                .1
        );
    }

    #[test]
    fn parse_condition_sets_test() {
        assert_eq!(
            vec!(
                ConditionSet {
                    condition_indexes: vec!(0x55),
                },
                ConditionSet {
                    condition_indexes: vec!(0x55, 0xaa),
                },
                ConditionSet {
                    condition_indexes: vec!(0x55, 0xaa, 0xcc),
                },
            ),
            parse_condition_sets(CompleteByteSlice(&[
                0x55, 0xff, 0x55, 0xaa, 0xff, 0x55, 0xaa, 0xcc, 0xff
            ]))
            .unwrap()
            .1
        );
    }

    #[test]
    fn parse_conditions_test() {
        assert_eq!(
            vec!(
                Condition {
                    op: 0x55,
                    args: [0xaa, 0x00, 0xff],
                },
                Condition {
                    op: 0x11,
                    args: [0x22, 0x33, 0x44],
                },
            ),
            parse_conditions(CompleteByteSlice(&[
                0x55, 0xaa, 0x00, 0xff, 0x11, 0x22, 0x33, 0x44
            ]))
            .unwrap()
            .1
        );
    }

    #[cfg_attr(feature = "ci_tests", ignore)]
    #[test]
    fn parse_test() {
        let data = test_utils::load_rom().unwrap();
        let ai = parse(&data).unwrap();

        assert_eq!(256, ai.groups.len());

        assert_eq!(
            Group {
                entries: vec!(GroupEntry {
                    condition_set_index: 0x08,
                    action_index: 0x04,
                },),
            },
            ai.groups[2]
        );
    }
}
