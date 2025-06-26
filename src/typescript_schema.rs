/// TypeScript type definitions that correspond to our Rust entity types.
/// These are used in LLM prompts to ensure accurate JSON structure generation.
pub const TYPESCRIPT_SCHEMA: &str = r#"
// Supporting Types and Enums
interface Price {
  amount: number;
  currency: string;
  description?: string;
}

interface ContactInfo {
  email?: string;
  phone?: string;
  fax?: string;
  social_media?: Record<string, string>;
}

type EventStatus = "Upcoming" | "Ongoing" | "Completed" | "Cancelled" | "Postponed";

type ProductAvailability = "InStock" | "OutOfStock" | "PreOrder" | "Discontinued" | "LimitedAvailability";

type EmploymentType = "FullTime" | "PartTime" | "Contract" | "Freelance" | "Internship" | "Temporary";

type SalaryPeriod = "Hourly" | "Daily" | "Weekly" | "Monthly" | "Yearly";

interface SalaryRange {
  min?: number;
  max?: number;
  currency: string;
  period: SalaryPeriod;
}

// Core Entity Types
interface Person {
  type: "Person";
  first_name?: string;
  last_name?: string;
  full_name?: string;
  title?: string;
  company?: string;
  email?: string;
  phone?: string;
  bio?: string;
  social_links?: string[];
}

interface Location {
  type: "Location";
  name?: string;
  address?: string;
  city?: string;
  state?: string;
  country?: string;
  postal_code?: string;
  latitude?: number;
  longitude?: number;
  venue_type?: string;
}

interface Event {
  type: "Event";
  title: string;
  description?: string;
  start_date?: string; // YYYY-MM-DD format
  end_date?: string;   // YYYY-MM-DD format
  start_time?: string;
  end_time?: string;
  location?: Location;
  organizer?: Person;
  attendees?: Person[];
  category?: string;
  tags?: string[];
  price?: Price;
  registration_url?: string;
  status: EventStatus;
}

interface Review {
  author?: Person;
  rating: number;
  title?: string;
  content?: string;
  date?: string; // ISO 8601 format
  verified_purchase: boolean;
}

interface Product {
  type: "Product";
  name: string;
  description?: string;
  price?: Price;
  brand?: string;
  category?: string;
  sku?: string;
  availability: ProductAvailability;
  specifications?: Record<string, string>;
  images?: string[];
  reviews?: Review[];
  rating?: number;
}

interface Organization {
  type: "Organization";
  name: string;
  description?: string;
  website?: string;
  industry?: string;
  size?: string;
  founded?: string; // YYYY-MM-DD format
  headquarters?: Location;
  employees?: Person[];
  contact_info: ContactInfo;
}

interface NewsArticle {
  type: "NewsArticle";
  headline: string;
  summary?: string;
  content?: string;
  author?: Person;
  publication_date?: string; // ISO 8601 format
  source?: string;
  category?: string;
  tags?: string[];
  url?: string;
}

interface JobListing {
  type: "JobListing";
  title: string;
  company?: Organization;
  location?: Location;
  description?: string;
  requirements?: string[];
  salary_range?: SalaryRange;
  employment_type: EmploymentType;
  remote_allowed: boolean;
  posted_date?: string; // ISO 8601 format
  application_deadline?: string; // YYYY-MM-DD format
  application_url?: string;
}

// Union type for all entities
type ExtractedEntity = Person | Location | Event | Product | Organization | NewsArticle | JobListing;

// Response structure for entity extraction
interface EntityExtractionResponse {
  entities: ExtractedEntity[];
  raw_analysis: string;
  extraction_confidence: number; // 0.0 to 1.0
}
"#;

/// Get specific TypeScript schema for a particular entity type
pub fn get_entity_schema(entity_type: &str) -> &'static str {
    match entity_type {
        "Person" => {
            r#"
interface Person {
  type: "Person";
  first_name?: string;
  last_name?: string;
  full_name?: string;
  title?: string;
  company?: string;
  email?: string;
  phone?: string;
  bio?: string;
  social_links?: string[];
}"#
        }
        "Location" => {
            r#"
interface Location {
  type: "Location";
  name?: string;
  address?: string;
  city?: string;
  state?: string;
  country?: string;
  postal_code?: string;
  latitude?: number;
  longitude?: number;
  venue_type?: string;
}"#
        }
        "Event" => {
            r#"
interface Event {
  type: "Event";
  title: string;
  description?: string;
  start_date?: string; // YYYY-MM-DD format
  end_date?: string;   // YYYY-MM-DD format
  start_time?: string;
  end_time?: string;
  location?: Location;
  organizer?: Person;
  attendees?: Person[];
  category?: string;
  tags?: string[];
  price?: Price;
  registration_url?: string;
  status: "Upcoming" | "Ongoing" | "Completed" | "Cancelled" | "Postponed";
}"#
        }
        "Product" => {
            r#"
interface Product {
  type: "Product";
  name: string;
  description?: string;
  price?: Price;
  brand?: string;
  category?: string;
  sku?: string;
  availability: "InStock" | "OutOfStock" | "PreOrder" | "Discontinued" | "LimitedAvailability";
  specifications?: Record<string, string>;
  images?: string[];
  reviews?: Review[];
  rating?: number;
}"#
        }
        "Organization" => {
            r#"
interface Organization {
  type: "Organization";
  name: string;
  description?: string;
  website?: string;
  industry?: string;
  size?: string;
  founded?: string; // YYYY-MM-DD format
  headquarters?: Location;
  employees?: Person[];
  contact_info: ContactInfo;
}"#
        }
        "NewsArticle" => {
            r#"
interface NewsArticle {
  type: "NewsArticle";
  headline: string;
  summary?: string;
  content?: string;
  author?: Person;
  publication_date?: string; // ISO 8601 format
  source?: string;
  category?: string;
  tags?: string[];
  url?: string;
}"#
        }
        "JobListing" => {
            r#"
interface JobListing {
  type: "JobListing";
  title: string;
  company?: Organization;
  location?: Location;
  description?: string;
  requirements?: string[];
  salary_range?: SalaryRange;
  employment_type: "FullTime" | "PartTime" | "Contract" | "Freelance" | "Internship" | "Temporary";
  remote_allowed: boolean;
  posted_date?: string; // ISO 8601 format
  application_deadline?: string; // YYYY-MM-DD format
  application_url?: string;
}"#
        }
        _ => "",
    }
}

/// Get supporting type schemas that are referenced by main entities
pub fn get_supporting_schemas() -> &'static str {
    r#"
interface Price {
  amount: number;
  currency: string;
  description?: string;
}

interface ContactInfo {
  email?: string;
  phone?: string;
  fax?: string;
  social_media?: Record<string, string>;
}

interface SalaryRange {
  min?: number;
  max?: number;
  currency: string;
  period: "Hourly" | "Daily" | "Weekly" | "Monthly" | "Yearly";
}

interface Review {
  author?: Person;
  rating: number;
  title?: string;
  content?: string;
  date?: string; // ISO 8601 format
  verified_purchase: boolean;
}"#
}

/// Generate example JSON for a specific entity type
pub fn get_entity_example(entity_type: &str) -> &'static str {
    match entity_type {
        "Person" => {
            r#"
{
  "type": "Person",
  "first_name": "John",
  "last_name": "Doe",
  "title": "Software Engineer",
  "company": "Tech Corp",
  "email": "john.doe@techcorp.com",
  "phone": "+1-555-0123"
}"#
        }
        "Location" => {
            r#"
{
  "type": "Location", 
  "name": "Tech Conference Center",
  "address": "123 Main Street",
  "city": "San Francisco",
  "state": "CA",
  "country": "USA",
  "postal_code": "94105"
}"#
        }
        "Event" => {
            r#"
{
  "type": "Event",
  "title": "Tech Conference 2024",
  "description": "Annual technology conference",
  "start_date": "2024-03-15",
  "end_date": "2024-03-17",
  "location": {
    "type": "Location",
    "name": "Convention Center",
    "city": "San Francisco"
  },
  "status": "Upcoming"
}"#
        }
        "Product" => {
            r#"
{
  "type": "Product",
  "name": "Wireless Headphones",
  "description": "High-quality wireless headphones",
  "price": {
    "amount": 199.99,
    "currency": "USD"
  },
  "brand": "TechBrand",
  "availability": "InStock"
}"#
        }
        "Organization" => {
            r#"
{
  "type": "Organization",
  "name": "Tech Corp",
  "description": "Leading technology company",
  "website": "https://techcorp.com",
  "industry": "Technology",
  "contact_info": {
    "email": "info@techcorp.com"
  }
}"#
        }
        "NewsArticle" => {
            r#"
{
  "type": "NewsArticle",
  "headline": "Breaking Tech News",
  "summary": "Important technology announcement",
  "author": {
    "type": "Person",
    "full_name": "Jane Reporter"
  },
  "publication_date": "2024-01-15T10:30:00Z",
  "source": "Tech News Daily"
}"#
        }
        "JobListing" => {
            r#"
{
  "type": "JobListing",
  "title": "Senior Software Engineer",
  "company": {
    "type": "Organization",
    "name": "Tech Startup"
  },
  "employment_type": "FullTime",
  "remote_allowed": true,
  "salary_range": {
    "min": 120000,
    "max": 180000,
    "currency": "USD",
    "period": "Yearly"
  }
}"#
        }
        _ => "{}",
    }
}
