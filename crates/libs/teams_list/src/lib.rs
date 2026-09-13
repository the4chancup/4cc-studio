//! The 4cc team identity space (`TeamName`, `TeamId`) and the `teams_list.txt`
//! file.
//!
//! `TeamName::new` is the one place the `/xx/` fold lives, so export names and
//! the list's `Name` column cannot drift apart. The list itself parses and
//! writes `teams_list.txt` and [`reconcile`] merges an incoming list into a
//! working copy; callers own filesystem I/O.

mod file;
mod id;
mod name;
mod reconcile;

pub use file::{Row, TeamsList, TeamsListError};
pub use id::{TeamId, TeamIdError};
pub use name::{TeamName, TeamNameError};
pub use reconcile::{MergeSummary, reconcile};

#[cfg(test)]
mod tests {
    use super::*;

    fn list(text: &str) -> TeamsList {
        TeamsList::parse(text).unwrap()
    }

    #[test]
    fn upstream_parses_to_220_rows() {
        let list = list(TeamsList::UPSTREAM);
        assert_eq!(list.rows().len(), 220);

        let umajp = TeamName::new("UMAJP").unwrap();
        assert_eq!(list.id_of(&umajp), TeamId::new(827).ok());
        assert_eq!(
            list.name_of(TeamId::new(827).unwrap()).unwrap().as_str(),
            "/umajp/"
        );
        assert_eq!(
            list.id_of(&TeamName::new("/@/").unwrap()),
            TeamId::new(781).ok()
        );

        // 9 Backup + 16 VGL + 70 Invitational rows have names that do not fold.
        let placeholders = list
            .rows()
            .iter()
            .filter(|row| matches!(row, Row::Placeholder { .. }))
            .count();
        assert_eq!(placeholders, 95);
        assert!(matches!(
            &list.rows()[71],
            Row::Placeholder { fields } if fields[0] == "772" && fields[1] == "Backup 1" && fields.len() == 4
        ));
        assert_eq!(list.teams().count(), 220 - placeholders);
    }

    #[test]
    fn unmodified_list_writes_back_identically() {
        let list = list(TeamsList::UPSTREAM);
        assert_eq!(list.write(), TeamsList::UPSTREAM);
    }

    #[test]
    fn bom_and_lf_input_writes_crlf_without_bom() {
        let list = list("\u{feff}ID\tName\n701\t/co/\n\n702\t/a/\r\n");
        assert_eq!(list.write(), "ID\tName\r\n701\t/co/\r\n702\t/a/\r\n");
    }

    #[test]
    fn header_errors_carry_the_right_variant_and_line() {
        assert_eq!(
            TeamsList::parse("Name\tID\n"),
            Err(TeamsListError::MissingHeader)
        );
        assert_eq!(
            TeamsList::parse("ID\tOther\n"),
            Err(TeamsListError::MissingColumn("Name"))
        );
        assert_eq!(TeamsList::parse(""), Err(TeamsListError::MissingHeader));
        assert_eq!(
            TeamsList::parse("ID\tName\n700\t/co/\n"),
            Err(TeamsListError::InvalidId {
                line: 2,
                text: "700".to_owned()
            })
        );
        assert_eq!(
            TeamsList::parse("ID\tName\nabc\t/co/\n"),
            Err(TeamsListError::InvalidId {
                line: 2,
                text: "abc".to_owned()
            })
        );
        assert_eq!(
            TeamsList::parse("ID\tName\n701\t/co/\n701\t/a/\n"),
            Err(TeamsListError::DuplicateId { line: 3, id: 701 })
        );
        assert_eq!(
            TeamsList::parse("ID\tName\n701\t/co/\n702\t/CO/\n"),
            Err(TeamsListError::DuplicateName {
                line: 3,
                name: "/co/".to_owned()
            })
        );
    }

    #[test]
    fn reconcile_merges_incoming_into_working() {
        let working = list(
            "ID\tName\tMin\n\
             701\t/co/\t1\n\
             772\tBackup 1\t9\n\
             703\t/shared/\t3\n",
        );
        let incoming = list(
            "ID\tName\tMin\n\
             701\t/co/\t11\n\
             799\t/shared/\t33\n\
             750\t/newteam/\t55\n",
        );
        let (merged, summary) = reconcile(&working, &incoming);

        assert_eq!(
            summary.added,
            [(TeamId::new(750).unwrap(), TeamName::new("newteam").unwrap())]
        );
        // 701 kept (identical in both), 703 overridden — `kept` counts
        // working team rows that survived unchanged.
        assert_eq!(summary.kept, 1);
        assert_eq!(
            summary.overridden,
            [(
                TeamName::new("shared").unwrap(),
                TeamId::new(703).unwrap(),
                TeamId::new(799).unwrap()
            )]
        );
        assert!(summary.unresolved.is_empty());

        // Placeholder preserved, appended row last, raw name/rest copied.
        assert!(matches!(merged.rows()[1], Row::Placeholder { .. }));
        let last = merged.rows().last().unwrap();
        assert!(matches!(
            last,
            Row::Team { id, raw_name, rest, .. }
            if *id == TeamId::new(750).unwrap() && raw_name == "/newteam/" && rest == &["55".to_owned()]
        ));
        assert!(merged.write().ends_with("750\t/newteam/\t55\r\n"));
        assert_eq!(
            merged.id_of(&TeamName::new("shared").unwrap()),
            TeamId::new(799).ok()
        );
    }

    #[test]
    fn reconcile_reports_unresolved_collisions() {
        let working = list("ID\tName\n701\t/co/\n702\t/a/\n");
        // New name but existing id -> unresolved by id.
        let (merged, summary) = reconcile(&working, &list("ID\tName\n701\t/other/\n"));
        assert_eq!(
            summary.unresolved,
            [(TeamId::new(701).unwrap(), TeamName::new("other").unwrap())]
        );
        assert_eq!(merged.teams().count(), 2);
        assert!(merged.id_of(&TeamName::new("other").unwrap()).is_none());

        // Existing name but the incoming id belongs to a different row ->
        // unresolved by name.
        let (merged, summary) = reconcile(&working, &list("ID\tName\n702\t/co/\n"));
        assert_eq!(
            summary.unresolved,
            [(TeamId::new(702).unwrap(), TeamName::new("co").unwrap())]
        );
        assert_eq!(
            merged.id_of(&TeamName::new("co").unwrap()),
            TeamId::new(701).ok()
        );
    }
}
