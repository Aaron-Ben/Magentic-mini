use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tldextract::{ExtractOptions, TldExtractor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UrlStatus {
    Allowed,
    Rejected,
}

pub type UrlStatusMap = HashMap<String, UrlStatus>;

pub struct UrlStatusManager {
    url_statuses: Option<UrlStatusMap>,
    url_block_list: Option<Vec<String>>,
    extractor: TldExtractor<'static>,
}

impl UrlStatusManager {
    pub fn new(
        url_statuses: Option<UrlStatusMap>,
        url_block_list: Option<Vec<String>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut opts = ExtractOptions::default();
        opts.cache = false; // 可选：禁用缓存（小项目无所谓）

        let extractor = TldExtractor::new(opts)?;

        // 清理 url_statuses 中的尾部斜杠
        let cleaned_statuses = url_statuses.map(|map| {
            map.into_iter()
                .map(|(k, v)| (k.trim().trim_end_matches('/').to_string(), v))
                .collect()
        });

        Ok(Self {
            url_statuses: cleaned_statuses,
            url_block_list,
            extractor,
        })
    }

    /// 设置某个 URL 的状态（仅当 url_statuses 被启用时生效）
    pub fn set_url_status(&mut self, url: &str, status: UrlStatus) {
        if let Some(ref mut statuses) = self.url_statuses {
            let cleaned = url.trim().trim_end_matches('/');
            statuses.insert(cleaned.to_string(), status);
        }
    }

    /// 检查 proposed_url 是否匹配 registered_url 模式
    fn is_url_match(&self, registered_url: &str, proposed_url: &str) -> bool {
        // 补全 scheme
        let reg_url = if registered_url.starts_with("http://") || registered_url.starts_with("https://") {
            registered_url.to_string()
        } else {
            format!("http://{}", registered_url)
        };

        let prop_url = if proposed_url.starts_with("http://") || proposed_url.starts_with("https://") {
            proposed_url.to_string()
        } else {
            format!("http://{}", proposed_url)
        };

        let reg_parsed = match url::Url::parse(&reg_url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let prop_parsed = match url::Url::parse(&prop_url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        // 处理 scheme：http 和 https 视为等价
        let http_equiv = |s: &str| s == "http" || s == "https";
        if http_equiv(reg_parsed.scheme()) && http_equiv(prop_parsed.scheme()) {
            // 允许
        } else if reg_parsed.scheme() != prop_parsed.scheme() {
            return false;
        }

        // 解析 TLD
        let reg_tld = match self.extractor.extract(reg_parsed.host_str().unwrap_or("")) {
            Ok(t) => t,
            Err(_) => return false,
        };
        let prop_tld = match self.extractor.extract(prop_parsed.host_str().unwrap_or("")) {
            Ok(t) => t,
            Err(_) => return false,
        };

        // 子域匹配
        if let Some(sub) = reg_tld.subdomain {
            if sub != prop_tld.subdomain.unwrap_or("") {
                return false;
            }
        }

        // 主域和后缀匹配
        if reg_tld.domain != prop_tld.domain {
            return false;
        }
        if let (Some(suf), Some(prop_suf)) = (reg_tld.suffix, prop_tld.suffix) {
            if suf != prop_suf {
                return false;
            }
        }

        // 路径前缀匹配
        if !reg_parsed.path().is_empty() {
            if !prop_parsed.path().starts_with(reg_parsed.path()) {
                return false;
            }
        }

        true
    }

    /// 检查 URL 是否在 block 列表中
    pub fn is_url_blocked(&self, url: &str) -> bool {
        if let Some(ref block_list) = self.url_block_list {
            return block_list.iter().any(|site| self.is_url_match(site, url));
        }
        false
    }

    /// 检查 URL 是否被显式拒绝（包括 block list）
    pub fn is_url_rejected(&self, url: &str) -> bool {
        if self.is_url_blocked(url) {
            return true;
        }

        if let Some(ref statuses) = self.url_statuses {
            return statuses.iter().any(|(site, &status)| {
                status == UrlStatus::Rejected && self.is_url_match(site, url)
            });
        }

        false
    }

    /// 检查 URL 是否被允许（不检查 rejected，但 block 优先）
    pub fn is_url_allowed(&self, url: &str) -> bool {
        if self.is_url_blocked(url) {
            return false;
        }

        match &self.url_statuses {
            None => true, // 未设置列表 → 全部允许
            Some(statuses) => {
                statuses.iter().any(|(site, &status)| {
                    status == UrlStatus::Allowed && self.is_url_match(site, url)
                })
            }
        }
    }

    /// 获取所有允许的站点
    pub fn get_allowed_sites(&self) -> Option<Vec<String>> {
        self.url_statuses.as_ref().map(|statuses| {
            statuses
                .iter()
                .filter(|(_, &status)| status == UrlStatus::Allowed)
                .map(|(url, _)| url.clone())
                .collect()
        })
    }

    /// 获取所有拒绝的站点
    pub fn get_rejected_sites(&self) -> Option<Vec<String>> {
        self.url_statuses.as_ref().map(|statuses| {
            statuses
                .iter()
                .filter(|(_, &status)| status == UrlStatus::Rejected)
                .map(|(url, _)| url.clone())
                .collect()
        })
    }

    /// 获取所有阻断的站点
    pub fn get_blocked_sites(&self) -> Option<Vec<String>> {
        self.url_block_list.clone()
    }
}