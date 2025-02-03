use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
    dob: DateTime<Utc>,
    skills: Vec<String>,
}

fn main() -> Result<()> {
    let user = User {
        name: "John Doe".to_string(),
        age: 30,
        dob: Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
    };
    let json = serde_json::to_string(&user)?;
    println!("{}", json);

    let user1: User = serde_json::from_str(&json)?;
    println!("{:?}", user1);

    assert_eq!(user, user1);
    Ok(())
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("User", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("age", &self.age)?;
        state.serialize_field("dob", &self.dob)?;
        state.serialize_field("skills", &self.skills)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UserVisitor;

        impl<'de> Visitor<'de> for UserVisitor {
            type Value = User;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct User")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut age = None;
                let mut dob = None;
                let mut skills = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "name" => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        "age" => {
                            if age.is_some() {
                                return Err(serde::de::Error::duplicate_field("age"));
                            }
                            age = Some(map.next_value()?);
                        }
                        "dob" => {
                            if dob.is_some() {
                                return Err(serde::de::Error::duplicate_field("dob"));
                            }
                            dob = Some(map.next_value()?);
                        }
                        "skills" => {
                            if skills.is_some() {
                                return Err(serde::de::Error::duplicate_field("skills"));
                            }
                            skills = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let name = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;
                let age = age.ok_or_else(|| serde::de::Error::missing_field("age"))?;
                let dob = dob.ok_or_else(|| serde::de::Error::missing_field("dob"))?;
                let skills = skills.ok_or_else(|| serde::de::Error::missing_field("skills"))?;

                Ok(User {
                    name,
                    age,
                    dob,
                    skills,
                })
            }
        }

        deserializer.deserialize_struct("User", &["name", "age", "dob", "skills"], UserVisitor)
    }
}
