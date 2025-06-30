use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LLMResponse {
    pub is_objective_met: bool,
    pub results: Option<Vec<ExtractedEntity>>,
    pub analysis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct Person {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub title: Option<String>,
    pub company: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub bio: Option<String>,
    pub social_links: Vec<String>,
}

impl Default for Person {
    fn default() -> Self {
        Self::new()
    }
}

impl Person {
    pub fn new() -> Self {
        Self {
            first_name: None,
            last_name: None,
            full_name: None,
            title: None,
            company: None,
            email: None,
            phone: None,
            bio: None,
            social_links: Vec::new(),
        }
    }

    pub fn with_name(first_name: Option<String>, last_name: Option<String>) -> Self {
        let mut person = Self::new();
        person.first_name = first_name;
        person.last_name = last_name;
        person
    }

    pub fn display_name(&self) -> String {
        if let Some(full_name) = &self.full_name {
            full_name.clone()
        } else {
            match (&self.first_name, &self.last_name) {
                (Some(first), Some(last)) => format!("{first} {last}"),
                (Some(first), None) => first.clone(),
                (None, Some(last)) => last.clone(),
                (None, None) => "Unknown".to_string(),
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct Location {
    pub name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub venue_type: Option<String>,
}

impl Default for Location {
    fn default() -> Self {
        Self::new()
    }
}

impl Location {
    pub fn new() -> Self {
        Self {
            name: None,
            address: None,
            city: None,
            state: None,
            country: None,
            postal_code: None,
            latitude: None,
            longitude: None,
            venue_type: None,
        }
    }

    pub fn display_address(&self) -> String {
        let mut parts = Vec::new();

        if let Some(address) = &self.address {
            parts.push(address.clone());
        }
        if let Some(city) = &self.city {
            parts.push(city.clone());
        }
        if let Some(state) = &self.state {
            parts.push(state.clone());
        }
        if let Some(country) = &self.country {
            parts.push(country.clone());
        }

        if parts.is_empty() {
            self.name
                .as_deref()
                .unwrap_or("Unknown location")
                .to_string()
        } else {
            parts.join(", ")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct Event {
    pub title: String,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub location: Option<Location>,
    pub organizer: Option<Person>,
    pub attendees: Vec<Person>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub price: Option<Price>,
    pub registration_url: Option<String>,
    pub status: EventStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum EventStatus {
    Upcoming,
    Ongoing,
    Completed,
    Cancelled,
    Postponed,
}

impl Event {
    pub fn new(title: String) -> Self {
        Self {
            title,
            description: None,
            start_date: None,
            end_date: None,
            start_time: None,
            end_time: None,
            location: None,
            organizer: None,
            attendees: Vec::new(),
            category: None,
            tags: Vec::new(),
            price: None,
            registration_url: None,
            status: EventStatus::Upcoming,
        }
    }

    pub fn is_multi_day(&self) -> bool {
        match (&self.start_date, &self.end_date) {
            (Some(start), Some(end)) => start != end,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct Price {
    pub amount: f64,
    pub currency: String,
    pub description: Option<String>,
}

impl Price {
    pub fn new(amount: f64, currency: String) -> Self {
        Self {
            amount,
            currency,
            description: None,
        }
    }

    pub fn display(&self) -> String {
        match &self.description {
            Some(desc) => format!("{} {} ({})", self.amount, self.currency, desc),
            None => format!("{} {}", self.amount, self.currency),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct Product {
    pub name: String,
    pub description: Option<String>,
    pub price: Option<Price>,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub sku: Option<String>,
    pub availability: ProductAvailability,
    pub specifications: HashMap<String, String>,
    pub images: Vec<String>,
    pub reviews: Vec<Review>,
    pub rating: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum ProductAvailability {
    InStock,
    OutOfStock,
    PreOrder,
    Discontinued,
    LimitedAvailability,
}

impl Product {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            price: None,
            brand: None,
            category: None,
            sku: None,
            availability: ProductAvailability::InStock,
            specifications: HashMap::new(),
            images: Vec::new(),
            reviews: Vec::new(),
            rating: None,
        }
    }

    pub fn average_rating(&self) -> Option<f32> {
        if self.reviews.is_empty() {
            self.rating
        } else {
            let sum: f32 = self.reviews.iter().map(|r| r.rating).sum();
            Some(sum / self.reviews.len() as f32)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct Review {
    pub author: Option<Person>,
    pub rating: f32,
    pub title: Option<String>,
    pub content: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub verified_purchase: bool,
}

impl Review {
    pub fn new(rating: f32) -> Self {
        Self {
            author: None,
            rating,
            title: None,
            content: None,
            date: None,
            verified_purchase: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct Organization {
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub size: Option<String>,
    pub founded: Option<NaiveDate>,
    pub headquarters: Option<Location>,
    pub employees: Vec<Person>,
    pub contact_info: ContactInfo,
}

impl Organization {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            website: None,
            industry: None,
            size: None,
            founded: None,
            headquarters: None,
            employees: Vec::new(),
            contact_info: ContactInfo::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct ContactInfo {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub fax: Option<String>,
    pub social_media: HashMap<String, String>,
}

impl Default for ContactInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl ContactInfo {
    pub fn new() -> Self {
        Self {
            email: None,
            phone: None,
            fax: None,
            social_media: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct NewsArticle {
    pub headline: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub author: Option<Person>,
    pub publication_date: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub url: Option<String>,
}

impl NewsArticle {
    pub fn new(headline: String) -> Self {
        Self {
            headline,
            summary: None,
            content: None,
            author: None,
            publication_date: None,
            source: None,
            category: None,
            tags: Vec::new(),
            url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct JobListing {
    pub title: String,
    pub company: Option<Organization>,
    pub location: Option<Location>,
    pub description: Option<String>,
    pub requirements: Vec<String>,
    pub salary_range: Option<SalaryRange>,
    pub employment_type: EmploymentType,
    pub remote_allowed: bool,
    pub posted_date: Option<DateTime<Utc>>,
    pub application_deadline: Option<NaiveDate>,
    pub application_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct SalaryRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub currency: String,
    pub period: SalaryPeriod,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum SalaryPeriod {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Freelance,
    Internship,
    Temporary,
}

impl JobListing {
    pub fn new(title: String) -> Self {
        Self {
            title,
            company: None,
            location: None,
            description: None,
            requirements: Vec::new(),
            salary_range: None,
            employment_type: EmploymentType::FullTime,
            remote_allowed: false,
            posted_date: None,
            application_deadline: None,
            application_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum ExtractedEntity {
    Person(Person),
    Location(Location),
    Event(Event),
    Product(Product),
    Organization(Organization),
    NewsArticle(NewsArticle),
    JobListing(JobListing),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EntityExtractionResult {
    pub url: String,
    pub objective: String,
    pub entities: Vec<ExtractedEntity>,
    pub raw_analysis: String,
    pub extraction_confidence: f32,
}

impl EntityExtractionResult {
    pub fn new(url: String, objective: String) -> Self {
        Self {
            url,
            objective,
            entities: Vec::new(),
            raw_analysis: String::new(),
            extraction_confidence: 0.0,
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn get_persons(&self) -> Vec<&Person> {
        self.entities
            .iter()
            .filter_map(|entity| match entity {
                ExtractedEntity::Person(p) => Some(p),
                _ => None,
            })
            .collect()
    }

    pub fn get_locations(&self) -> Vec<&Location> {
        self.entities
            .iter()
            .filter_map(|entity| match entity {
                ExtractedEntity::Location(l) => Some(l),
                _ => None,
            })
            .collect()
    }

    pub fn get_events(&self) -> Vec<&Event> {
        self.entities
            .iter()
            .filter_map(|entity| match entity {
                ExtractedEntity::Event(e) => Some(e),
                _ => None,
            })
            .collect()
    }

    pub fn get_products(&self) -> Vec<&Product> {
        self.entities
            .iter()
            .filter_map(|entity| match entity {
                ExtractedEntity::Product(p) => Some(p),
                _ => None,
            })
            .collect()
    }

    pub fn get_organizations(&self) -> Vec<&Organization> {
        self.entities
            .iter()
            .filter_map(|entity| match entity {
                ExtractedEntity::Organization(o) => Some(o),
                _ => None,
            })
            .collect()
    }

    pub fn get_news_articles(&self) -> Vec<&NewsArticle> {
        self.entities
            .iter()
            .filter_map(|entity| match entity {
                ExtractedEntity::NewsArticle(n) => Some(n),
                _ => None,
            })
            .collect()
    }

    pub fn get_job_listings(&self) -> Vec<&JobListing> {
        self.entities
            .iter()
            .filter_map(|entity| match entity {
                ExtractedEntity::JobListing(j) => Some(j),
                _ => None,
            })
            .collect()
    }
}
