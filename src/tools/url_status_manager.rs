use std::collections::HashMap;
use url::Url;
use tldextract::{TldExtractor, TldOption};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UrlStatus {
    Allowed,
    Rejected,
}

pub struct UrlStatusManager {
    url_statuses: Option<HashMap<String, UrlStatus>>,
    url_block_list: Option<Vec<String>>,
}

impl UrlStatusManager {
    pub fn new(
        url_statuses: Option<HashMap<String, UrlStatus>>,
        url_block_list: Option<Vec<String>>,
    ) -> Self {
        let url_statuses = url_statuses.map(|statuses| {
            let cleaned: HashMap<_, _> = statuses
                .into_iter()
                .map(|(k, v)| (k.trim_end_matches('/').to_string(), v))
                .collect();
            cleaned
        });

        Self {
            url_statuses,
            url_block_list,
        }
    }

    pub fn _set_url_status(&mut self, url: &str, status: UrlStatus) {
        if let Some(statuses) = &mut self.url_statuses {
            let cleaned_url = url.trim().trim_end_matches('/').to_string();
            statuses.insert(cleaned_url, status);
        }
    }

    fn _is_url_match(&self, registered_url: &str, proposed_url: &str) -> bool {
        let reg_url = if registered_url.contains("://") {
        registered_url.to_string()
        } else {
            format!("http://{}", registered_url)
        };
        let prop_url = if proposed_url.contains("://") {
            proposed_url.to_string()
        } else {
            format!("http://{}", proposed_url)
        };

        let parsed_reg = match Url::parse(&reg_url) {
            Ok(u) => u,
            Err(_) => return false,
        };
        let parsed_prop = match Url::parse(&prop_url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        // HTTP/HTTPS 兼容
        let scheme_reg = parsed_reg.scheme();
        let scheme_prop = parsed_prop.scheme();
        let is_httpish = |s: &str| s == "http" || s == "https";
        if !(is_httpish(scheme_reg) && is_httpish(scheme_prop)) && scheme_reg != scheme_prop {
            return false;
        }

        let host_reg = match parsed_reg.host_str() {
            Some(h) => h,
            None => return false,
        };
        let host_prop = match parsed_prop.host_str() {
            Some(h) => h,
            None => return false,
        };

        // 使用 tldextract 提取域名组件
        let extractor = TldExtractor::new(TldOption::default());
        
        let extracted_reg = match extractor.extract(host_reg) {
            Ok(result) => result,
            Err(_) => return false,
        };
        let extracted_prop = match extractor.extract(host_prop) {
            Ok(result) => result,
            Err(_) => return false,
        };

        // reg 的顶级域，主域名，子域名
        let suffix_reg = extracted_reg.suffix.unwrap_or_default();
        let domain_reg = extracted_reg.domain.unwrap_or_default();
        let subdomain_reg = extracted_reg.subdomain.unwrap_or_default();

        // prop 的顶级域，主域名，子域名
        let suffix_prop = extracted_prop.suffix.unwrap_or_default();
        let domain_prop = extracted_prop.domain.unwrap_or_default();
        let subdomain_prop = extracted_prop.subdomain.unwrap_or_default();

        if !subdomain_reg.is_empty() && subdomain_reg != subdomain_prop {
            return false;
        }

        if domain_reg != domain_prop {
            return false;
        }
        if !suffix_reg.is_empty() && suffix_reg != suffix_prop {
            return false;
        }

        let path_reg = parsed_reg.path();
        let path_prop = parsed_prop.path();
        if !path_reg.is_empty() && !path_prop.starts_with(path_reg) {
            return false;
        }
        true
    }

    pub fn _is_url_blocked(&self, url: &str) -> bool {
        self.url_block_list
            .as_ref()
            .map_or(false, |list|list.iter().any(|site|self._is_url_match(site,url)))
    }

    pub fn _is_url_rejected(&self, url: &str) -> bool {
        if self._is_url_blocked(url) {
            return true;
        }

        self.url_statuses.as_ref().map_or(false, |statuses| {
            statuses
                .iter()
                .any(|(site, status)| self._is_url_match(site, url) && *status == UrlStatus::Rejected)
        })
    }

    pub fn _is_url_allowed(&self, url: &str) -> bool {
        if self._is_url_blocked(url) {
            return false;
        }

        if self.url_statuses.is_none() {
            return true;
        }

        self.url_statuses.as_ref().map_or(false, |statuses| {
            statuses
                .iter()
                .any(|(site, status)| self._is_url_match(site, url) && *status == UrlStatus::Allowed)
        })
    }

    pub fn _get_allowed_sites(&self) -> Option<Vec<String>> {
        self.url_statuses.as_ref().map(|statuses| {
            statuses
                .iter()
                .filter(|(_, status)| **status == UrlStatus::Allowed)
                .map(|(site, _)| site.clone())
                .collect()
        })
    }

    pub fn _get_rejected_sites(&self) -> Option<Vec<String>> {
        self.url_statuses.as_ref().map(|statuses| {
            statuses
                .iter()
                .filter(|(_, status)| **status == UrlStatus::Rejected)
                .map(|(site, _)| site.clone())
                .collect()
        })
    }

    pub fn _get_blocked_sites(&self) -> Option<&Vec<String>> {
        self.url_block_list.as_ref()
    }
}