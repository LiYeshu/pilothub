use std::collections::BTreeMap;

use serde::Serialize;

use super::skill_store::SkillRecord;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionComponentType {
    Skill,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExtensionComponent {
    pub id: String,
    #[serde(rename = "type")]
    pub component_type: ExtensionComponentType,
    pub name: String,
    pub description: Option<String>,
    pub skill_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExtensionSource {
    pub source_type: String,
    pub source_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub source: ExtensionSource,
    pub components: Vec<ExtensionComponent>,
}

pub fn map_skills_to_extensions(skills: Vec<SkillRecord>) -> Vec<Extension> {
    let mut extensions = BTreeMap::<String, Extension>::new();

    for skill in skills {
        let source_identity = skill.source_ref.clone().unwrap_or_else(|| skill.id.clone());
        let extension_id = format!("{}:{source_identity}", skill.source_type);

        let extension = extensions
            .entry(extension_id.clone())
            .or_insert_with(|| Extension {
                id: extension_id,
                name: extension_name(&skill),
                source: ExtensionSource {
                    source_type: skill.source_type.clone(),
                    source_ref: skill.source_ref.clone(),
                },
                components: Vec::new(),
            });

        extension.components.push(ExtensionComponent {
            id: skill.id.clone(),
            component_type: ExtensionComponentType::Skill,
            name: skill.name,
            description: skill.description,
            skill_id: skill.id,
        });
    }

    extensions
        .into_values()
        .map(|mut extension| {
            extension
                .components
                .sort_by(|left, right| left.name.cmp(&right.name));
            extension
        })
        .collect()
}

fn extension_name(skill: &SkillRecord) -> String {
    skill
        .source_ref
        .as_deref()
        .and_then(|source_ref| {
            source_ref
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .map(|name| name.trim_end_matches(".git"))
                .filter(|name| !name.is_empty())
        })
        .unwrap_or(&skill.name)
        .to_string()
}
