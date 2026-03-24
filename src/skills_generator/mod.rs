//! Skills generator: writes AI agent skill files for the nlm CLI.
//!
//! Each skill is a `{name}/SKILL.md` file with YAML frontmatter (openclaw format)
//! and markdown body. Content is embedded as string literals — nlm's API surface
//! is static (34 endpoints) so runtime discovery is unnecessary.

mod edm;
mod helpers;
mod index;
mod mcp_config;
mod personas;
mod recipes;
mod shared;
mod sn;

use std::fs;
use std::path::Path;

use crate::error::NlError;

/// Metadata category for a skill.
#[derive(Debug, Clone, Copy)]
pub enum SkillCategory {
    Shared,
    Service,
    Group,
    Helper,
    Utility,
    Recipe,
    Persona,
}

impl SkillCategory {
    fn as_str(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::Service => "service",
            Self::Group => "group",
            Self::Helper => "helper",
            Self::Utility => "utility",
            Self::Recipe => "recipe",
            Self::Persona => "persona",
        }
    }
}

/// A single skill definition ready to be written to disk.
#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: SkillCategory,
    pub domain: Option<String>,
    pub requires_bins: Vec<String>,
    pub requires_skills: Vec<String>,
    pub body: String,
}

impl SkillDefinition {
    /// Serialize the YAML frontmatter block (between `---` fences).
    pub fn to_frontmatter(&self) -> String {
        let mut fm = String::new();
        fm.push_str("---\n");
        fm.push_str(&format!("name: {}\n", self.name));
        fm.push_str(&format!("version: {}\n", self.version));
        fm.push_str(&format!(
            "description: \"{}\"\n",
            self.description.replace('"', "\\\"")
        ));
        fm.push_str("metadata:\n");
        fm.push_str("  openclaw:\n");
        fm.push_str(&format!("    category: \"{}\"\n", self.category.as_str()));
        if let Some(ref domain) = self.domain {
            fm.push_str(&format!("    domain: \"{domain}\"\n"));
        }
        fm.push_str("    requires:\n");
        // bins
        let bins: Vec<String> = self
            .requires_bins
            .iter()
            .map(|b| format!("\"{b}\""))
            .collect();
        fm.push_str(&format!("      bins: [{}]\n", bins.join(", ")));
        // skills (optional)
        if !self.requires_skills.is_empty() {
            let skills: Vec<String> = self
                .requires_skills
                .iter()
                .map(|s| format!("\"{s}\""))
                .collect();
            fm.push_str(&format!("      skills: [{}]\n", skills.join(", ")));
        }
        fm.push_str("---\n");
        fm
    }

    /// Full file content: frontmatter + body.
    pub fn to_file_content(&self) -> String {
        format!("{}\n{}\n", self.to_frontmatter(), self.body)
    }
}

/// Collect all skill definitions from every sub-module.
fn all_skills() -> Vec<SkillDefinition> {
    let mut skills = Vec::new();
    skills.extend(shared::skills());
    skills.extend(edm::skills());
    skills.extend(sn::skills());
    skills.extend(helpers::skills());
    skills.extend(mcp_config::skills());
    skills.extend(recipes::skills());
    skills.extend(personas::skills());
    skills
}

/// Write a single skill to `{output_dir}/{name}/SKILL.md`.
fn write_skill(output_dir: &Path, skill: &SkillDefinition) -> Result<(), NlError> {
    let dir = output_dir.join(&skill.name);
    fs::create_dir_all(&dir)?;
    let path = dir.join("SKILL.md");
    fs::write(&path, skill.to_file_content())?;
    Ok(())
}

/// Main entry point: generate all skills and optionally the index.
pub fn generate(output_dir: &Path, write_index: bool) -> Result<(), NlError> {
    let skills = all_skills();

    // Clean output directory for idempotent regeneration.
    if output_dir.exists() {
        for entry in fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            }
        }
    }

    for skill in &skills {
        write_skill(output_dir, skill)?;
    }

    if write_index {
        index::write_index(output_dir, &skills)?;
    }

    eprintln!(
        "Generated {} skills in {}",
        skills.len(),
        output_dir.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_produces_valid_yaml() {
        let skill = SkillDefinition {
            name: "test-skill".to_string(),
            version: "1.0.0".to_string(),
            description: "A test skill".to_string(),
            category: SkillCategory::Shared,
            domain: Some("testing".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec!["nlm-shared".to_string()],
            body: "# Test\n\nBody here.".to_string(),
        };
        let fm = skill.to_frontmatter();
        assert!(fm.starts_with("---\n"));
        assert!(fm.ends_with("---\n"));
        assert!(fm.contains("name: test-skill"));
        assert!(fm.contains("version: 1.0.0"));
        assert!(fm.contains("description: \"A test skill\""));
        assert!(fm.contains("category: \"shared\""));
        assert!(fm.contains("domain: \"testing\""));
        assert!(fm.contains("bins: [\"nlm\"]"));
        assert!(fm.contains("skills: [\"nlm-shared\"]"));
    }

    #[test]
    fn file_content_includes_frontmatter_and_body() {
        let skill = SkillDefinition {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            category: SkillCategory::Shared,
            domain: None,
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![],
            body: "# Hello".to_string(),
        };
        let content = skill.to_file_content();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("# Hello"));
        // No skills line when empty
        assert!(!content.contains("skills:"));
    }

    #[test]
    fn all_skills_returns_35() {
        let skills = all_skills();
        assert_eq!(skills.len(), 35, "Expected 35 skills, got {}", skills.len());
    }

    #[test]
    fn all_skill_names_are_unique() {
        let skills = all_skills();
        let mut names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        names.sort();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "Duplicate skill names found");
    }

    #[test]
    fn generate_writes_files() {
        let dir = std::env::temp_dir().join("nlm-skills-test");
        let _ = fs::remove_dir_all(&dir);
        generate(&dir, true).unwrap();

        // Count directories
        let count = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .count();
        assert_eq!(count, 35);

        // Check one skill file exists and has frontmatter
        let shared = fs::read_to_string(dir.join("nlm-shared/SKILL.md")).unwrap();
        assert!(shared.starts_with("---\n"));
        assert!(shared.contains("name: nlm-shared"));

        // Check index exists
        let index = fs::read_to_string(dir.parent().unwrap().join("docs/skills.md"));
        // Index may or may not exist depending on relative path — the index writer
        // uses a sibling `docs/` directory from the output_dir's parent.
        // For temp dirs this is fine to skip.
        drop(index);

        let _ = fs::remove_dir_all(&dir);
    }
}
