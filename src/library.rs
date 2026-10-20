//! The sidebar's tree over the saved-query library: one row per folder and one
//! per query, built from the names `store::saved_queries` lists.

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

/// Every folder the library's queries sit in, nested ones included: what
/// Collapse All closes, and what a remembered closed folder must still be
/// among to be worth remembering.
pub fn folders(names: &[String]) -> HashSet<String> {
    names
        .iter()
        .flat_map(|name| {
            name.match_indices('/')
                .map(|(at, _)| name[..at].to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The library as tree rows: folders ahead of queries at every level, each in
/// name order. Every folder is open unless `collapsed` names it, so a rebuild
/// after a save keeps closed what was closed and shows a new folder open.
pub fn tree(names: &[String], collapsed: &HashSet<String>) -> Vec<TreeItem> {
    let mut root = Folder::default();
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
        let items = tree(&names, &HashSet::new());
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
    fn every_folder_is_listed_once_nested_ones_included() {
        let names = ["zeta", "reports/weekly", "reports/q1/sales"]
            .map(String::from)
            .to_vec();
        assert_eq!(
            folders(&names),
            HashSet::from(["reports".to_string(), "reports/q1".to_string()])
        );
    }

    #[test]
    fn a_closed_folder_stays_closed_across_a_rebuild() {
        let names = ["reports/q1/sales", "reports/weekly"]
            .map(String::from)
            .to_vec();
        let closed = HashSet::from(["reports/q1".to_string()]);
        let items = tree(&names, &closed);
        assert!(items[0].is_expanded());
        assert!(!items[0].children[0].is_expanded());
    }
}
