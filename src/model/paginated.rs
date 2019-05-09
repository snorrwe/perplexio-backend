#[derive(Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total_pages: i64,
    pub page: i64,
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, total_pages: i64, page: i64) -> Self {
        Self {
            items: items,
            total_pages: total_pages,
            page: page,
        }
    }
}
