use stq_types::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogStocksResponse {
    pub id: StockId,
    pub warehouse_id: WarehouseId,
    pub product_id: ProductId,
    pub quantity: Quantity,
}
