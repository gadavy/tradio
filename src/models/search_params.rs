pub enum OrderBy {
    CreatedAt,
}

#[derive(Default)]
pub struct StationsFilter {
    pub order_by: Option<OrderBy>,
}
