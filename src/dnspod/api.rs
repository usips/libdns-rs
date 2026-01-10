use std::error::Error;

use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client as HttpClient,
};
use serde::{Deserialize, Serialize};

const DNSPOD_API_URL: &str = "https://api.dnspod.com";

#[derive(Debug, Clone)]
pub struct Client {
    http_client: HttpClient,
    login_token: String,
}

impl Client {
    pub fn new(login_token: &str) -> Result<Self, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        // UserAgent is required by DNSPod API
        headers.insert(
            "User-Agent",
            HeaderValue::from_static("libdns-rs/0.1.0 (github.com/lus/libdns-rs)"),
        );

        let http_client = HttpClient::builder().default_headers(headers).build()?;
        Ok(Self {
            http_client,
            login_token: login_token.to_string(),
        })
    }

    fn build_form_params(&self, params: &[(&str, &str)]) -> String {
        let mut form = format!("login_token={}&format=json", self.login_token);
        for (key, value) in params {
            form.push_str(&format!("&{}={}", key, value));
        }
        form
    }

    // Domain APIs

    pub async fn list_domains(
        &self,
        offset: Option<u32>,
        length: Option<u32>,
    ) -> Result<DomainListResponse, DnspodError> {
        let mut params = Vec::new();
        let offset_str;
        let length_str;

        if let Some(o) = offset {
            offset_str = o.to_string();
            params.push(("offset", offset_str.as_str()));
        }
        if let Some(l) = length {
            length_str = l.to_string();
            params.push(("length", length_str.as_str()));
        }

        let response = self
            .http_client
            .post(format!("{}/Domain.List", DNSPOD_API_URL))
            .body(self.build_form_params(&params))
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: DomainListResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }

    pub async fn get_domain(&self, domain_id: &str) -> Result<DomainInfoResponse, DnspodError> {
        let params = [("domain_id", domain_id)];

        let response = self
            .http_client
            .post(format!("{}/Domain.Info", DNSPOD_API_URL))
            .body(self.build_form_params(&params))
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: DomainInfoResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }

    pub async fn get_domain_by_name(
        &self,
        domain: &str,
    ) -> Result<DomainInfoResponse, DnspodError> {
        let params = [("domain", domain)];

        let response = self
            .http_client
            .post(format!("{}/Domain.Info", DNSPOD_API_URL))
            .body(self.build_form_params(&params))
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: DomainInfoResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }

    pub async fn create_domain(&self, domain: &str) -> Result<DomainCreateResponse, DnspodError> {
        let params = [("domain", domain)];

        let response = self
            .http_client
            .post(format!("{}/Domain.Create", DNSPOD_API_URL))
            .body(self.build_form_params(&params))
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: DomainCreateResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }

    pub async fn delete_domain(&self, domain_id: &str) -> Result<StatusResponse, DnspodError> {
        let params = [("domain_id", domain_id)];

        let response = self
            .http_client
            .post(format!("{}/Domain.Remove", DNSPOD_API_URL))
            .body(self.build_form_params(&params))
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: StatusResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }

    // Record APIs

    pub async fn list_records(
        &self,
        domain_id: &str,
        offset: Option<u32>,
        length: Option<u32>,
    ) -> Result<RecordListResponse, DnspodError> {
        let mut params = vec![("domain_id", domain_id.to_string())];

        if let Some(o) = offset {
            params.push(("offset", o.to_string()));
        }
        if let Some(l) = length {
            params.push(("length", l.to_string()));
        }

        let form = params.iter().fold(
            format!("login_token={}&format=json", self.login_token),
            |acc, (k, v)| format!("{}&{}={}", acc, k, v),
        );

        let response = self
            .http_client
            .post(format!("{}/Record.List", DNSPOD_API_URL))
            .body(form)
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: RecordListResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            // Empty result is code 10, which is not an error for listing
            if result.status.code == "10" {
                return Ok(RecordListResponse {
                    status: result.status,
                    domain: result.domain,
                    info: result.info,
                    records: Some(Vec::new()),
                });
            }
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }

    pub async fn get_record(
        &self,
        domain_id: &str,
        record_id: &str,
    ) -> Result<RecordInfoResponse, DnspodError> {
        let params = [("domain_id", domain_id), ("record_id", record_id)];

        let response = self
            .http_client
            .post(format!("{}/Record.Info", DNSPOD_API_URL))
            .body(self.build_form_params(&params))
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: RecordInfoResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }

    pub async fn create_record(
        &self,
        domain_id: &str,
        sub_domain: &str,
        record_type: &str,
        record_line: &str,
        value: &str,
        mx: Option<u16>,
        ttl: Option<u64>,
    ) -> Result<RecordCreateResponse, DnspodError> {
        let mut params = vec![
            ("domain_id", domain_id.to_string()),
            ("sub_domain", sub_domain.to_string()),
            ("record_type", record_type.to_string()),
            ("record_line", record_line.to_string()),
            ("value", value.to_string()),
        ];

        if let Some(mx_val) = mx {
            params.push(("mx", mx_val.to_string()));
        }
        if let Some(ttl_val) = ttl {
            params.push(("ttl", ttl_val.to_string()));
        }

        let form = params.iter().fold(
            format!("login_token={}&format=json", self.login_token),
            |acc, (k, v)| format!("{}&{}={}", acc, k, v),
        );

        let response = self
            .http_client
            .post(format!("{}/Record.Create", DNSPOD_API_URL))
            .body(form)
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: RecordCreateResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }

    pub async fn modify_record(
        &self,
        domain_id: &str,
        record_id: &str,
        sub_domain: &str,
        record_type: &str,
        record_line: &str,
        value: &str,
        mx: Option<u16>,
        ttl: Option<u64>,
    ) -> Result<RecordModifyResponse, DnspodError> {
        let mut params = vec![
            ("domain_id", domain_id.to_string()),
            ("record_id", record_id.to_string()),
            ("sub_domain", sub_domain.to_string()),
            ("record_type", record_type.to_string()),
            ("record_line", record_line.to_string()),
            ("value", value.to_string()),
        ];

        if let Some(mx_val) = mx {
            params.push(("mx", mx_val.to_string()));
        }
        if let Some(ttl_val) = ttl {
            params.push(("ttl", ttl_val.to_string()));
        }

        let form = params.iter().fold(
            format!("login_token={}&format=json", self.login_token),
            |acc, (k, v)| format!("{}&{}={}", acc, k, v),
        );

        let response = self
            .http_client
            .post(format!("{}/Record.Modify", DNSPOD_API_URL))
            .body(form)
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: RecordModifyResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }

    pub async fn delete_record(
        &self,
        domain_id: &str,
        record_id: &str,
    ) -> Result<StatusResponse, DnspodError> {
        let params = [("domain_id", domain_id), ("record_id", record_id)];

        let response = self
            .http_client
            .post(format!("{}/Record.Remove", DNSPOD_API_URL))
            .body(self.build_form_params(&params))
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: StatusResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }

    pub async fn set_record_status(
        &self,
        domain_id: &str,
        record_id: &str,
        status: &str,
    ) -> Result<RecordStatusResponse, DnspodError> {
        let params = [
            ("domain_id", domain_id),
            ("record_id", record_id),
            ("status", status),
        ];

        let response = self
            .http_client
            .post(format!("{}/Record.Status", DNSPOD_API_URL))
            .body(self.build_form_params(&params))
            .send()
            .await
            .map_err(DnspodError::Request)?;

        let result: RecordStatusResponse = response.json().await.map_err(DnspodError::Request)?;

        if result.status.code != "1" {
            return Err(DnspodError::Api(result.status));
        }

        Ok(result)
    }
}

// Error types

#[derive(Debug)]
pub enum DnspodError {
    Request(reqwest::Error),
    Api(Status),
}

impl std::fmt::Display for DnspodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnspodError::Request(e) => write!(f, "Request error: {}", e),
            DnspodError::Api(status) => {
                write!(f, "API error {}: {}", status.code, status.message)
            }
        }
    }
}

impl std::error::Error for DnspodError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DnspodError::Request(e) => Some(e),
            DnspodError::Api(_) => None,
        }
    }
}

// Response types

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Status {
    pub code: String,
    pub message: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusResponse {
    pub status: Status,
}

// Domain types

#[derive(Debug, Clone, Deserialize)]
pub struct Domain {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub punycode: Option<String>,
    #[serde(default)]
    pub grade: Option<String>,
    #[serde(default)]
    pub grade_title: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub ext_status: Option<String>,
    #[serde(default)]
    pub records: Option<String>,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub is_mark: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub is_vip: Option<String>,
    #[serde(default)]
    pub searchengine_push: Option<String>,
    #[serde(default)]
    pub beian: Option<String>,
    #[serde(default)]
    pub created_on: Option<String>,
    #[serde(default)]
    pub updated_on: Option<String>,
    #[serde(default)]
    pub ttl: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
}

impl Domain {
    pub fn get_ttl(&self) -> u64 {
        self.ttl
            .as_ref()
            .and_then(|t| t.parse().ok())
            .unwrap_or(600)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomainListInfo {
    pub domain_total: Option<u32>,
    pub all_total: Option<u32>,
    pub mine_total: Option<u32>,
    pub share_total: Option<u32>,
    pub vip_total: Option<u32>,
    pub ismark_total: Option<u32>,
    pub pause_total: Option<u32>,
    pub error_total: Option<u32>,
    pub lock_total: Option<u32>,
    pub spam_total: Option<u32>,
    pub vip_expire: Option<u32>,
    pub share_out_total: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomainListResponse {
    pub status: Status,
    #[serde(default)]
    pub info: Option<DomainListInfo>,
    #[serde(default)]
    pub domains: Option<Vec<Domain>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomainInfoResponse {
    pub status: Status,
    pub domain: Domain,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomainCreateDomain {
    pub id: String,
    #[serde(default)]
    pub punycode: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomainCreateResponse {
    pub status: Status,
    pub domain: DomainCreateDomain,
}

// Record types

#[derive(Debug, Clone, Deserialize)]
pub struct Record {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub line: Option<String>,
    #[serde(rename = "type", default)]
    pub record_type: Option<String>,
    #[serde(default)]
    pub ttl: Option<String>,
    pub value: String,
    #[serde(default)]
    pub mx: Option<String>,
    #[serde(default)]
    pub enabled: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub monitor_status: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub updated_on: Option<String>,
    #[serde(default)]
    pub hold: Option<String>,
}

impl Record {
    pub fn get_ttl(&self, default_ttl: u64) -> u64 {
        self.ttl
            .as_ref()
            .and_then(|t| t.parse().ok())
            .unwrap_or(default_ttl)
    }

    pub fn get_type(&self) -> &str {
        self.record_type.as_deref().unwrap_or("A")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordInfo {
    pub id: String,
    pub sub_domain: String,
    pub record_type: String,
    pub record_line: String,
    pub value: String,
    #[serde(default)]
    pub mx: Option<String>,
    #[serde(default)]
    pub ttl: Option<String>,
    #[serde(default)]
    pub enabled: Option<String>,
    #[serde(default)]
    pub monitor_status: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub updated_on: Option<String>,
    #[serde(default)]
    pub domain_id: Option<String>,
}

impl RecordInfo {
    pub fn get_ttl(&self, default_ttl: u64) -> u64 {
        self.ttl
            .as_ref()
            .and_then(|t| t.parse().ok())
            .unwrap_or(default_ttl)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordListInfo {
    #[serde(default)]
    pub sub_domains: Option<String>,
    #[serde(default)]
    pub record_total: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordListDomain {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub punycode: Option<String>,
    #[serde(default)]
    pub grade: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordListResponse {
    pub status: Status,
    #[serde(default)]
    pub domain: Option<RecordListDomain>,
    #[serde(default)]
    pub info: Option<RecordListInfo>,
    #[serde(default)]
    pub records: Option<Vec<Record>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordInfoDomain {
    pub id: String,
    pub domain: String,
    #[serde(default)]
    pub domain_grade: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordInfoResponse {
    pub status: Status,
    pub domain: RecordInfoDomain,
    pub record: RecordInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordCreateRecord {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordCreateResponse {
    pub status: Status,
    pub record: RecordCreateRecord,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordModifyRecord {
    pub id: serde_json::Value, // Can be string or number
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordModifyResponse {
    pub status: Status,
    pub record: RecordModifyRecord,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordStatusRecord {
    pub id: serde_json::Value, // Can be string or number
    pub name: String,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordStatusResponse {
    pub status: Status,
    pub record: RecordStatusRecord,
}
