//! Info entity and related metadata types.

/// Core metadata identifying the CLI tool.
///
/// The `Info` object provides essential metadata about the CLI application,
/// including its name, version, description, and contact information.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Info {
    /// The title of the CLI application.
    pub title: String,

    /// A description of the CLI application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The version of the CLI application.
    pub version: String,

    /// Contact information for the CLI application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    /// License information for the CLI application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
}

impl Info {
    /// Creates a new `Info` with the given title and version.
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            version: version.into(),
            contact: None,
            license: None,
        }
    }

    /// Sets the description for the CLI application.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the contact information for the CLI application.
    pub fn contact(mut self, contact: Contact) -> Self {
        self.contact = Some(contact);
        self
    }

    /// Sets the license information for the CLI application.
    pub fn license(mut self, license: License) -> Self {
        self.license = Some(license);
        self
    }
}

/// Contact information for the CLI application.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Contact {
    /// The identifying name of the contact person/organization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The URL pointing to the contact information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// The email address of the contact person/organization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl Contact {
    /// Creates a new empty `Contact`.
    pub fn new() -> Self {
        Self {
            name: None,
            url: None,
            email: None,
        }
    }

    /// Sets the name of the contact.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the URL of the contact.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the email of the contact.
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }
}

impl Default for Contact {
    fn default() -> Self {
        Self::new()
    }
}

/// License information for the CLI application.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct License {
    /// The license name used for the CLI application.
    pub name: String,

    /// A URL to the license used for the CLI application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl License {
    /// Creates a new `License` with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: None,
        }
    }

    /// Sets the URL for the license.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}
