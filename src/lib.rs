use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CargoToml {
    pub dependencies: Option<BTreeMap<String, Dependency>>,
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
    pub version: String,
}

impl CargoToml {
    pub fn get_dependency(&self, name: &str) -> Option<&Dependency> {
        self.dependencies.as_ref().and_then(|deps| deps.get(name))
    }
}

pub fn parse_cargo_toml(toml: &str) -> Result<CargoToml, String> {
    #[derive(Deserialize)]
    struct CargoTomlVisitor {
        dependencies: Option<BTreeMap<String, toml::Value>>,
    }

    let parsed = toml::from_str::<CargoTomlVisitor>(toml).map_err(|e| e.to_string())?;

    let dependencies = parsed
        .dependencies
        .map(parse_dependencies)
        .map_or(Ok(None), |v| v.map(Some))?;

    Ok(CargoToml { dependencies })
}

fn parse_dependencies(
    deps: BTreeMap<String, toml::Value>,
) -> Result<BTreeMap<String, Dependency>, String> {
    let mut dependencies = BTreeMap::new();
    for (name, value) in deps {
        if value.is_str() {
            let version = value.as_str().unwrap();
            dependencies.insert(
                name,
                Dependency {
                    version: version.to_string(),
                },
            );
        } else if value.is_table() {
            if let Some(version) = value.get("version") {
                dependencies.insert(
                    name,
                    Dependency {
                        version: version
                            .as_str()
                            .ok_or("can't parse version as str".to_string())?
                            .to_string(),
                    },
                );
            } else if let Some(tag) = value.get("tag") {
                dependencies.insert(
                    name,
                    Dependency {
                        version: tag
                            .as_str()
                            .ok_or("can't parse tag as str".to_string())?
                            .to_string(),
                    },
                );
            } else {
                return Err("can't find version or path".to_string());
            }
        }
    }
    Ok(dependencies)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_cargo_toml() {
        let toml = r#"
            [package]
            name = "hello"
            version = "0.1.0"
            [dependencies]
            serde = "1.0"
            toml = { version = "0.5" }
            hyle = { git = "https://github.com/Hyle-org/hyle", tag = "0.12" }
            "#;
        let cargo_toml = parse_cargo_toml(toml).unwrap();
        assert_eq!(cargo_toml.dependencies.as_ref().unwrap().len(), 3);

        assert_eq!(cargo_toml.get_dependency("serde").unwrap().version, "1.0");
        assert_eq!(cargo_toml.get_dependency("toml").unwrap().version, "0.5");
        assert_eq!(cargo_toml.get_dependency("hyle").unwrap().version, "0.12");
    }
}

