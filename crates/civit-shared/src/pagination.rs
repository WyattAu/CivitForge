//! Pagination types for list endpoints.

use serde::{Deserialize, Serialize};

/// Wrapper for paginated list responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    /// Items in the current page.
    pub data: Vec<T>,
    /// Pagination metadata.
    pub pagination: Pagination,
}

impl<T: Serialize> ListResponse<T> {
    /// Create a paginated response from items, total count, and params.
    pub fn from_total(data: Vec<T>, total: u64, params: &PaginationParams) -> Self {
        Self {
            data,
            pagination: Pagination::from_total(total, params),
        }
    }
}

/// Query parameters for paginated list endpoints.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PaginationParams {
    /// Maximum number of items to return (1-100, default 20).
    pub per_page: Option<u32>,
    /// Offset for pagination (0-indexed). Use `page` for cursor-based.
    pub offset: Option<u32>,
    /// 1-indexed page number. Mutually exclusive with `offset`.
    pub page: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            per_page: Some(20),
            offset: None,
            page: None,
        }
    }
}

impl PaginationParams {
    /// Returns the effective per_page, clamped to [1, 100].
    pub fn effective_per_page(&self) -> u32 {
        self.per_page.map(|p| p.clamp(1, 100)).unwrap_or(20)
    }

    /// Returns the effective offset, calculated from page or offset.
    pub fn effective_offset(&self) -> u32 {
        if let Some(offset) = self.offset {
            offset
        } else if let Some(page) = self.page {
            (page.saturating_sub(1)) * self.effective_per_page()
        } else {
            0
        }
    }
}

/// Pagination metadata returned with list responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Current page number (1-indexed).
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Total number of items across all pages.
    pub total: u64,
    /// Total number of pages.
    pub total_pages: u32,
}

impl Pagination {
    /// Create pagination metadata from total count and params.
    pub fn from_total(total: u64, params: &PaginationParams) -> Self {
        let per_page = params.effective_per_page() as u64;
        let total_pages = if total == 0 {
            1
        } else {
            total.div_ceil(per_page) as u32
        };
        let page = (params.effective_offset() / params.effective_per_page()) + 1;

        Self {
            page: page.min(total_pages),
            per_page: per_page as u32,
            total,
            total_pages,
        }
    }

    /// Whether there is a next page.
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    /// Whether there is a previous page.
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params() {
        let p = PaginationParams::default();
        assert_eq!(p.effective_per_page(), 20);
        assert_eq!(p.effective_offset(), 0);
    }

    #[test]
    fn page_based_offset() {
        let p = PaginationParams {
            per_page: Some(10),
            page: Some(3),
            offset: None,
        };
        assert_eq!(p.effective_offset(), 20); // (3-1) * 10
    }

    #[test]
    fn clamp_per_page() {
        let p = PaginationParams {
            per_page: Some(200),
            page: None,
            offset: None,
        };
        assert_eq!(p.effective_per_page(), 100);
    }

    #[test]
    fn pagination_from_total() {
        let params = PaginationParams {
            per_page: Some(10),
            page: Some(2),
            offset: None,
        };
        let pag = Pagination::from_total(25, &params);
        assert_eq!(pag.page, 2);
        assert_eq!(pag.per_page, 10);
        assert_eq!(pag.total, 25);
        assert_eq!(pag.total_pages, 3);
        assert!(pag.has_next());
        assert!(pag.has_prev());
    }
}
