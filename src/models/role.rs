use strum_macros::{Display, EnumString};

#[derive(Clone, Display, Debug, EnumString, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub enum Role {
    #[strum(ascii_case_insensitive)]
    User,
    #[strum(ascii_case_insensitive)]
    Admin,
}
