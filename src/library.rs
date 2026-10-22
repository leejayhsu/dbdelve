//! The sidebar's tree over the saved-query library: one row per folder and one
//! per query, built from the names `store::saved_queries` lists and the
//! folders `store::query_folders` does.

use std::collections::{BTreeMap, HashSet};

use gpui_component::tree::TreeItem;

const QUERY: &str = "query:";
const FOLDER: &str = "folder:";

/// The query a row opens, or `None` for a folder.
pub fn query_of(id: &str) -> Option<&str> {
    id.strip_prefix(QUERY)
}

/// The folder a row stands for, or `None` for a query.
pub fn folder_of(id: &str) -> Option<&str> {
    id.strip_prefix(FOLDER)
}

/// What the sidebar's name field is naming. A rename carries the name it is
/// leaving, a query's or a folder's path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Naming {
    NewQuery,
    NewFolder,
    RenameQuery(String),
    RenameFolder(String),
}

impl Naming {
    /// Above the field, since a prefilled field shows no placeholder to say
    /// what it is for.
    pub fn caption(&self) -> String {
        match self {
            Naming::NewQuery => "New query".to_string(),
            Naming::NewFolder => "New folder".to_string(),
            Naming::RenameQuery(name) => format!("Rename query {name}"),
            Naming::RenameFolder(path) => format!("Rename folder {path}"),
        }
    }
}

/// `name` with every folder trimmed as well as the whole, so `reports / daily`
/// is the folder people meant rather than one whose name ends in a space.
pub fn typed_name(value: &str) -> String {
    value
        .split('/')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("/")
}

/// Where a query or folder inside the folder `from` lands once `from` is
/// renamed to `to`, or `None` when it is not inside. The folder itself is not
/// inside, and neither is a query of the same name, `reports.sql` beside
/// `reports/`.
pub fn moved(path: &str, from: &str, to: &str) -> Option<String> {
    path.strip_prefix(from)?
        .strip_prefix('/')
        .map(|rest| format!("{to}/{rest}"))
}

/// The library as tree rows: folders ahead of queries at every level, each in
/// name order, an empty folder included. Every folder is open unless
/// `collapsed` names it, so a rebuild after a save keeps closed what was
/// closed and shows a new folder open.
pub fn tree(names: &[String], folders: &[String], collapsed: &HashSet<String>) -> Vec<TreeItem> {
    let mut root = Folder::default();
    for path in folders {
        let mut folder = &mut root;
        for part in path.split('/') {
            folder = folder.folders.entry(part.to_string()).or_default();
        }
    }
    for name in names {
        let mut folder = &mut root;
        let mut parts = name.split('/').peekable();
        while let Some(part) = parts.next() {
            match parts.peek() {
                Some(_) => folder = folder.folders.entry(part.to_string()).or_default(),
                None => folder.queries.push((part.to_string(), name.clone())),
            }
        }
    }
    root.items("", collapsed)
}

#[derive(Default)]
struct Folder {
    folders: BTreeMap<String, Folder>,
    /// Label and full name.
    queries: Vec<(String, String)>,
}

impl Folder {
    fn items(self, prefix: &str, collapsed: &HashSet<String>) -> Vec<TreeItem> {
        let mut items = Vec::new();
        for (label, folder) in self.folders {
            let path = format!("{prefix}{label}");
            let children = folder.items(&format!("{path}/"), collapsed);
            items.push(
                TreeItem::new(format!("{FOLDER}{path}"), label)
                    .expanded(!collapsed.contains(&path))
                    .children(children),
            );
        }
        let mut queries = self.queries;
        queries.sort();
        items.extend(
            queries
                .into_iter()
                .map(|(label, name)| TreeItem::new(format!("{QUERY}{name}"), label)),
        );
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outline(items: &[TreeItem], depth: usize, lines: &mut Vec<String>) {
        for item in items {
            lines.push(format!("{}{}", "  ".repeat(depth), item.id));
            outline(&item.children, depth + 1, lines);
        }
    }

    #[test]
    fn folders_come_first_and_each_query_opens_by_its_whole_name() {
        let names = ["zeta", "reports/weekly", "reports/q1/sales", "alpha"]
            .map(String::from)
            .to_vec();
        let items = tree(&names, &[], &HashSet::new());
        let mut lines = Vec::new();
        outline(&items, 0, &mut lines);
        assert_eq!(
            lines,
            [
                "folder:reports",
                "  folder:reports/q1",
                "    query:reports/q1/sales",
                "  query:reports/weekly",
                "query:alpha",
                "query:zeta",
            ]
        );
        assert_eq!(items[0].label.as_ref(), "reports");
        assert_eq!(items[0].children[1].label.as_ref(), "weekly");
        assert_eq!(query_of("query:reports/weekly"), Some("reports/weekly"));
        assert_eq!(query_of("folder:reports"), None);
        assert_eq!(folder_of("folder:reports/q1"), Some("reports/q1"));
        assert_eq!(folder_of("query:reports/weekly"), None);
    }

    #[test]
    fn an_empty_folder_is_a_row_of_its_own() {
        let names = ["reports/weekly".to_string()];
        let folders = ["archive", "reports", "reports/empty"].map(String::from);
        let mut lines = Vec::new();
        outline(&tree(&names, &folders, &HashSet::new()), 0, &mut lines);
        assert_eq!(
            lines,
            [
                "folder:archive",
                "folder:reports",
                "  folder:reports/empty",
                "  query:reports/weekly",
            ]
        );
    }

    #[test]
    fn a_rename_moves_what_is_inside_and_nothing_beside() {
        assert_eq!(moved("reports", "reports", "old"), None);
        assert_eq!(
            moved("reports/q1/sales", "reports", "old/reports"),
            Some("old/reports/q1/sales".into())
        );
        assert_eq!(moved("reports-2024/sales", "reports", "old"), None);
        assert_eq!(typed_name(" reports / daily "), "reports/daily");
    }

    #[test]
    fn a_closed_folder_stays_closed_across_a_rebuild() {
        let names = ["reports/q1/sales", "reports/weekly"]
            .map(String::from)
            .to_vec();
        let closed = HashSet::from(["reports/q1".to_string()]);
        let items = tree(&names, &[], &closed);
        assert!(items[0].is_expanded());
        assert!(!items[0].children[0].is_expanded());
    }
}
