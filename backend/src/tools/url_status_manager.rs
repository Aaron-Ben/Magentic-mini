use std::collections::HashMap;
use url::Url;
use tldextract::{TldExtractor, TldOption};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UrlStatus {
    Allowed,
    Rejected,
}

#[derive(Debug)]
pub struct UrlStatusManager {
    url_statuses: Option<HashMap<String, UrlStatus>>,
    url_block_list: Option<Vec<String>>,
    tld_extractor: TldExtractor,        // 缓存 TLD 解析器，避免重复初始化
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

        let tld_extractor = TldExtractor::new(TldOption::default());

        Self {
            url_statuses,
            url_block_list,
            tld_extractor,
        }
    }

    pub fn set_url_status(&mut self, url: &str, status: UrlStatus) {
        if let Some(statuses) = &mut self.url_statuses {
            let cleaned_url = url.trim().trim_end_matches('/').to_string();
            statuses.insert(cleaned_url, status);
        }
    }

    fn is_url_match(&self, registered_url: &str, proposed_url: &str) -> bool {
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


        fn extract_or_fallback(extractor: &TldExtractor, host: &str) -> (
            Option<String>, Option<String>, Option<String>,
        ) {
            match extractor.extract(host) {
                Ok(result) => (result.suffix, result.domain, result.subdomain),
                Err(_) => (None, Some(host.to_string()), None),
            }
        }

        let (subdomain_reg, domain_reg, suffix_reg) = extract_or_fallback(&self.tld_extractor, host_reg);
        let (subdomain_prop, domain_prop, suffix_prop) = extract_or_fallback(&self.tld_extractor, host_prop);

        // reg 的顶级域，主域名，子域名
        let suffix_reg_str = suffix_reg.as_deref().unwrap_or("");
        let domain_reg_str = domain_reg.as_deref().unwrap_or(host_reg);
        let subdomain_reg_str = subdomain_reg.as_deref().unwrap_or("");

        // prop 的顶级域，主域名，子域名
        let suffix_prop_str = suffix_prop.as_deref().unwrap_or("");
        let domain_prop_str = domain_prop.as_deref().unwrap_or(host_prop);
        let subdomain_prop_str = subdomain_prop.as_deref().unwrap_or("");

        if !subdomain_reg_str.is_empty() && subdomain_reg_str != subdomain_prop_str {
            return false;
        }

        if domain_reg_str != domain_prop_str {
            return false;
        }
        if !suffix_reg_str.is_empty() && suffix_reg_str != suffix_prop_str {
            return false;
        }

        let path_reg = parsed_reg.path();
        let path_prop = parsed_prop.path();
        
        if !path_reg.is_empty() {
            if !path_prop.starts_with(path_reg) {
                return false;
            }

            if path_reg != "/" && !path_reg.ends_with('/') && path_prop.len() > path_reg.len() {
                if !path_prop[path_reg.len()..].starts_with('/') {
                    return false;
                }
            }
        }

        true
    }

    pub fn is_url_blocked(&self, url: &str) -> bool {
        self.url_block_list
            .as_ref()
            .map_or(false, |list|list.iter().any(|site|self.is_url_match(site,url)))
    }

    pub fn is_url_rejected(&self, url: &str) -> bool {

        self.url_statuses.as_ref().map_or(false, |statuses| {
            statuses
                .iter()
                .any(|(site, status)| self.is_url_match(site, url) && *status == UrlStatus::Rejected)
        })
    }

    pub fn is_url_allowed(&self, url: &str) -> bool {
        if self.is_url_blocked(url) {
            return false;
        }

        if self.url_statuses.is_none() {
            return true;
        }

        self.url_statuses.as_ref().map_or(false, |statuses| {
            statuses
                .iter()
                .any(|(site, status)| self.is_url_match(site, url) && *status == UrlStatus::Allowed)
        })
    }

    pub fn get_allowed_sites(&self) -> Option<Vec<String>> {
        self.url_statuses.as_ref().map(|statuses| {
            statuses
                .iter()
                .filter(|(_, status)| **status == UrlStatus::Allowed)
                .map(|(site, _)| site.clone())
                .collect()
        })
    }

    pub fn get_rejected_sites(&self) -> Option<Vec<String>> {
        self.url_statuses.as_ref().map(|statuses| {
            statuses
                .iter()
                .filter(|(_, status)| **status == UrlStatus::Rejected)
                .map(|(site, _)| site.clone())
                .collect()
        })
    }

    pub fn get_blocked_sites(&self) -> Option<&Vec<String>> {
        self.url_block_list.as_ref()
    }
}