//! Monthly summary (docs/summary-format.md): per client, project and
//! category, the total time and the list of descriptions, as plain text.

use serde::Serialize;

use crate::i18n::Catalog;
use crate::id::Id;
use crate::time::format_hhmm;

/// One stored piece, already limited to the month and to billable entries.
/// The caller supplies them in date and time order; descriptions keep that
/// order.
#[derive(Debug, Clone)]
pub struct SummaryEntry {
    pub client_id: Id,
    pub client_name: String,
    pub project_id: Id,
    pub project_name: String,
    pub category_id: Id,
    pub category_name: String,
    pub description: String,
    pub minutes: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CategoryBlock {
    pub name: String,
    pub minutes: i64,
    pub descriptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProjectBlock {
    pub name: String,
    pub minutes: i64,
    pub categories: Vec<CategoryBlock>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ClientBlock {
    pub name: String,
    pub minutes: i64,
    pub projects: Vec<ProjectBlock>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Summary {
    pub minutes: i64,
    pub clients: Vec<ClientBlock>,
}

fn by_name(a: &str, b: &str) -> std::cmp::Ordering {
    a.to_lowercase().cmp(&b.to_lowercase()).then(a.cmp(b))
}

/// Line breaks and runs of spaces become single spaces.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

type ProjectAcc = (Id, ProjectBlock, Vec<(Id, CategoryBlock)>);
type ClientAcc = (Id, ClientBlock, Vec<ProjectAcc>);

pub fn build(entries: &[SummaryEntry]) -> Summary {
    // Group by identity, in first-seen order, then sort by name.
    let mut clients: Vec<ClientAcc> = Vec::new();
    for e in entries {
        let ci = match clients.iter().position(|c| c.0 == e.client_id) {
            Some(i) => i,
            None => {
                clients.push((
                    e.client_id.clone(),
                    ClientBlock {
                        name: e.client_name.clone(),
                        minutes: 0,
                        projects: vec![],
                    },
                    vec![],
                ));
                clients.len() - 1
            }
        };
        let projects = &mut clients[ci].2;
        let pi = match projects.iter().position(|p| p.0 == e.project_id) {
            Some(i) => i,
            None => {
                projects.push((
                    e.project_id.clone(),
                    ProjectBlock {
                        name: e.project_name.clone(),
                        minutes: 0,
                        categories: vec![],
                    },
                    vec![],
                ));
                projects.len() - 1
            }
        };
        let categories = &mut projects[pi].2;
        let ki = match categories.iter().position(|c| c.0 == e.category_id) {
            Some(i) => i,
            None => {
                categories.push((
                    e.category_id.clone(),
                    CategoryBlock {
                        name: e.category_name.clone(),
                        minutes: 0,
                        descriptions: vec![],
                    },
                ));
                categories.len() - 1
            }
        };
        let category = &mut categories[ki].1;
        category.minutes += e.minutes;
        category.descriptions.push(one_line(&e.description));
    }

    let mut total = 0;
    let mut out: Vec<ClientBlock> = Vec::new();
    for (_, mut client, projects) in clients {
        for (_, mut project, categories) in projects {
            let mut cats: Vec<CategoryBlock> = categories.into_iter().map(|c| c.1).collect();
            cats.sort_by(|a, b| by_name(&a.name, &b.name));
            project.minutes = cats.iter().map(|c| c.minutes).sum();
            project.categories = cats;
            client.projects.push(project);
        }
        client.projects.sort_by(|a, b| by_name(&a.name, &b.name));
        client.minutes = client.projects.iter().map(|p| p.minutes).sum();
        total += client.minutes;
        out.push(client);
    }
    out.sort_by(|a, b| by_name(&a.name, &b.name));
    Summary {
        minutes: total,
        clients: out,
    }
}

impl Summary {
    /// The plain text for `month` (1 to 12) of `year`, with labels from the catalog.
    pub fn to_text(&self, catalog: &Catalog, year: i32, month: u32) -> String {
        let fill = |key: &str, pairs: &[(&str, String)]| {
            let mut text = catalog.text(key).to_owned();
            for (name, value) in pairs {
                text = text.replace(&format!("{{{name}}}"), value);
            }
            text
        };
        let month_name = catalog.text(&format!("month.{month}")).to_owned();
        let mut out = fill(
            "summary.title",
            &[("month", month_name), ("year", year.to_string())],
        );
        out.push_str("\n\n");
        if self.clients.is_empty() {
            out.push_str(catalog.text("summary.empty"));
            out.push('\n');
            return out;
        }
        for client in &self.clients {
            let heading = client.name.to_uppercase();
            out.push_str(&format!(
                "{heading}\n{}\n\n",
                "=".repeat(heading.chars().count())
            ));
            for project in &client.projects {
                out.push_str(&format!(
                    "{} — {}\n",
                    project.name,
                    format_hhmm(project.minutes)
                ));
                for category in &project.categories {
                    out.push_str(&format!(
                        "  {} — {}\n",
                        category.name,
                        format_hhmm(category.minutes)
                    ));
                    for description in &category.descriptions {
                        out.push_str(&format!("    - {description}\n"));
                    }
                }
                out.push('\n');
            }
            out.push_str(&fill(
                "summary.client_total",
                &[("total", format_hhmm(client.minutes))],
            ));
            out.push_str("\n\n");
        }
        out.push_str(&fill(
            "summary.grand_total",
            &[("total", format_hhmm(self.minutes))],
        ));
        out.push('\n');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{IdGenerator, Origin};

    fn entry(
        ids: &[Id],
        client: usize,
        project: usize,
        category: usize,
        description: &str,
        minutes: i64,
    ) -> SummaryEntry {
        let names = ["Acme GmbH", "Beta Studio", "acme two"];
        let projects = ["Website relaunch", "Maintenance retainer", "Brand site"];
        let categories = ["Development", "Meetings", "Design"];
        SummaryEntry {
            client_id: ids[client].clone(),
            client_name: names[client].into(),
            project_id: ids[3 + project].clone(),
            project_name: projects[project].into(),
            category_id: ids[6 + category].clone(),
            category_name: categories[category].into(),
            description: description.into(),
            minutes,
        }
    }

    fn ids() -> Vec<Id> {
        let g = IdGenerator::new(Origin::new("t").unwrap());
        (0..9).map(|_| g.next_id()).collect()
    }

    #[test]
    fn sample_matches_the_signed_off_format() {
        let ids = ids();
        let entries = vec![
            entry(&ids, 0, 0, 0, "Built the contact form", 28 * 60),
            entry(&ids, 0, 0, 0, "Fixed Safari\nlayout   bug", 45),
            entry(&ids, 0, 0, 1, "Weekly sync", 9 * 60 + 15),
            entry(&ids, 0, 0, 2, "Revised hero", 3 * 60 + 30),
            entry(&ids, 1, 2, 2, "Logo variations", 8 * 60),
        ];
        let text = build(&entries).to_text(&Catalog::english(), 2026, 11);
        let expected = "TimeTrack summary — November 2026

ACME GMBH
=========

Website relaunch — 41:30
  Design — 3:30
    - Revised hero
  Development — 28:45
    - Built the contact form
    - Fixed Safari layout bug
  Meetings — 9:15
    - Weekly sync

Client total: 41:30

BETA STUDIO
===========

Brand site — 8:00
  Design — 8:00
    - Logo variations

Client total: 8:00

Grand total: 49:30
";
        assert_eq!(text, expected);
    }

    #[test]
    fn empty_month_says_so() {
        let text = build(&[]).to_text(&Catalog::english(), 2026, 12);
        assert_eq!(
            text,
            "TimeTrack summary — December 2026\n\nNo billable time was recorded.\n"
        );
    }

    #[test]
    fn clients_with_the_same_name_stay_separate() {
        let g = IdGenerator::new(Origin::new("t").unwrap());
        let mk = |client: Id| SummaryEntry {
            client_id: client,
            client_name: "Same".into(),
            project_id: g.next_id(),
            project_name: "P".into(),
            category_id: g.next_id(),
            category_name: "C".into(),
            description: "d".into(),
            minutes: 10,
        };
        let s = build(&[mk(g.next_id()), mk(g.next_id())]);
        assert_eq!(s.clients.len(), 2);
    }

    /// Totals come from minutes and add up at every level; every
    /// description appears (acceptance criterion 8).
    #[test]
    fn totals_add_up_and_every_description_appears() {
        let ids = ids();
        let mut seed: u64 = 12345;
        let mut next = |n: u64| {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) % n
        };
        for round in 0..200 {
            let count = 1 + next(40) as usize;
            let entries: Vec<SummaryEntry> = (0..count)
                .map(|i| {
                    entry(
                        &ids,
                        next(3) as usize,
                        next(3) as usize,
                        next(3) as usize,
                        &format!("round {round} entry {i}"),
                        1 + next(600) as i64,
                    )
                })
                .collect();
            let sum: i64 = entries.iter().map(|e| e.minutes).sum();
            let summary = build(&entries);
            assert_eq!(summary.minutes, sum);
            assert_eq!(summary.clients.iter().map(|c| c.minutes).sum::<i64>(), sum);
            let mut lines = 0;
            for c in &summary.clients {
                assert_eq!(c.projects.iter().map(|p| p.minutes).sum::<i64>(), c.minutes);
                for p in &c.projects {
                    assert_eq!(
                        p.categories.iter().map(|k| k.minutes).sum::<i64>(),
                        p.minutes
                    );
                    lines += p
                        .categories
                        .iter()
                        .map(|k| k.descriptions.len())
                        .sum::<usize>();
                }
            }
            assert_eq!(lines, count);
            let text = summary.to_text(&Catalog::english(), 2026, 11);
            for e in &entries {
                assert!(text.contains(&format!("- {}\n", e.description)));
            }
        }
    }
}
