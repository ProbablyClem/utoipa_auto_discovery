use utoipa::ToSchema;

#[derive(Debug, ToSchema)]
pub struct Schema1(pub u8);

#[derive(Debug, ToSchema)]
pub struct Schema2(pub u16);

#[derive(Debug, ToSchema)]
pub struct Schema3(pub u32);

#[derive(Debug, ToSchema)]
pub struct Schema4(pub u64);

#[derive(Debug, ToSchema)]
pub struct Schema5(pub u128);
